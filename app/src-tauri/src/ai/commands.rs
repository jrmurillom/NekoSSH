//! Comandos IPC Tauri, catálogo multi-shell y gestión de ventana independiente `neko-ai`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

use super::driver::{AiModelDescriptor, AiProviderDriver, DriverHealthStatus};
use super::drivers::AgyCliDriver;
use super::fsm::{AgentHarnessFsm, ExecutionMode, HarnessPhase};
use super::orchestrator::{AiTurnCancelRegistry, HarnessOrchestrator, TurnExecutionResultDto};
use super::preferences::{
    clamp_context_max_lines, get_ai_preferences, reset_ai_preferences, set_ai_preferences,
    AiPreferencesDto, DEFAULT_CONTEXT_MAX_LINES,
};
use super::pty_tap::{wrap_command_with_sentinel, PtyTapRegistry, SentinelPurpose};
use super::redactor::SecretRedactor;
use super::risk_guard::{CommandRiskGuard, ReadOnlyEnforcer};
use super::schema::RiskLevel;

pub const AI_COPILOT_WINDOW_LABEL: &str = "neko-ai";
pub const MAX_FEED_ENTRIES_PER_SESSION: usize = 150;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiFeedEntryDto {
    pub kind: String,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub expanded: Option<bool>,
    #[serde(default)]
    pub phase_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiPendingActionDto {
    pub purpose: String,
    pub original_command: String,
    pub draft_command: String,
    pub is_editing: bool,
    pub risk_level: String,
    #[serde(default)]
    pub audit_warning: Option<String>,
    #[serde(default)]
    pub backup_strategy: Option<String>,
    #[serde(default)]
    pub rollback_cmd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiFailedContextDto {
    pub command: String,
    pub exit_code: i32,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiRollbackDto {
    pub source_command: String,
    pub rollback_cmd: String,
    pub label: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiUiSessionStateDto {
    #[serde(default)]
    pub pending_action: Option<AiPendingActionDto>,
    #[serde(default)]
    pub last_failed: Option<AiFailedContextDto>,
    #[serde(default)]
    pub last_rollback: Option<AiRollbackDto>,
}

#[derive(Clone, Default)]
struct SessionUiSlot {
    feed: Vec<AiFeedEntryDto>,
    ui_state: AiUiSessionStateDto,
}

/// Memoria volátil (solo RAM de sesión) del feed visual y tarjetas HITL pendientes por terminal
/// para preservar estado entre cambios de pestaña SSH y reapertura de `neko-ai`.
#[derive(Clone, Default)]
pub struct AiUiFeedRegistry(Arc<Mutex<HashMap<String, SessionUiSlot>>>);

impl AiUiFeedRegistry {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    pub fn get(&self, session_id: &str) -> Vec<AiFeedEntryDto> {
        let map = self.0.lock().unwrap();
        map.get(session_id)
            .map(|slot| slot.feed.clone())
            .unwrap_or_default()
    }

    pub fn append(&self, session_id: &str, entry: AiFeedEntryDto) {
        let mut map = self.0.lock().unwrap();
        let slot = map.entry(session_id.to_string()).or_default();
        slot.feed.push(entry);
        if slot.feed.len() > MAX_FEED_ENTRIES_PER_SESSION {
            let drain_count = slot.feed.len() - MAX_FEED_ENTRIES_PER_SESSION;
            slot.feed.drain(0..drain_count);
        }
    }

    pub fn get_session_state(&self, session_id: &str) -> AiUiSessionStateDto {
        let map = self.0.lock().unwrap();
        map.get(session_id)
            .map(|slot| slot.ui_state.clone())
            .unwrap_or_default()
    }

    pub fn set_session_state(&self, session_id: &str, ui_state: AiUiSessionStateDto) {
        let mut map = self.0.lock().unwrap();
        let slot = map.entry(session_id.to_string()).or_default();
        slot.ui_state = ui_state;
    }

    pub fn clear(&self, session_id: &str) {
        let mut map = self.0.lock().unwrap();
        map.remove(session_id);
    }
}

/// Ejecuta la limpieza pura de cierre aislado de `neko-ai` (Opción A):
/// cancela turnos en vuelo y apaga la escucha PTY conservando memoria de sesión.
pub fn handle_copilot_window_close_state(
    ai_cancels: &AiTurnCancelRegistry,
    pty_taps: &PtyTapRegistry,
    catalog: &ActiveShellsCatalog,
) -> Option<ActiveShellsSnapshotDto> {
    ai_cancels.cancel_all();
    if pty_taps.disable_all_listening() {
        Some(catalog.get_snapshot(pty_taps))
    } else {
        None
    }
}

pub fn cleanup_on_copilot_window_closed(app: &AppHandle) {
    if let (Some(ai_cancels), Some(pty_taps), Some(catalog)) = (
        app.try_state::<AiTurnCancelRegistry>(),
        app.try_state::<PtyTapRegistry>(),
        app.try_state::<ActiveShellsCatalog>(),
    ) {
        if let Some(snap) = handle_copilot_window_close_state(&ai_cancels, &pty_taps, &catalog) {
            let _ = app.emit("ai-shells-updated", snap);
        }
    } else if let Some(ai_cancels) = app.try_state::<AiTurnCancelRegistry>() {
        ai_cancels.cancel_all();
    }
}

/// Registro en memoria de arneses de IA indexados por identificador de terminal/sesión.
#[derive(Clone, Default)]
pub struct HarnessRegistry(pub Arc<Mutex<HashMap<String, AgentHarnessFsm>>>);

impl HarnessRegistry {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    pub fn with_or_create<F, R>(&self, session_id: &str, default_mode: ExecutionMode, f: F) -> R
    where
        F: FnOnce(&mut AgentHarnessFsm) -> R,
    {
        let mut map = self.0.lock().unwrap();
        let fsm = map
            .entry(session_id.to_string())
            .or_insert_with(|| AgentHarnessFsm::new(session_id, default_mode));
        f(fsm)
    }

    pub fn remove(&self, session_id: &str) -> Option<AgentHarnessFsm> {
        let mut map = self.0.lock().unwrap();
        map.remove(session_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingAiSelectionDto {
    pub terminal_id: String,
    pub selection_text: String,
}

/// Cola efímera anti-race para selecciones enviadas con `Ctrl+Shift+E` antes o durante la apertura de `neko-ai`.
#[derive(Clone, Default)]
pub struct AiSelectionQueue(Arc<Mutex<Option<PendingAiSelectionDto>>>);

impl AiSelectionQueue {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub fn set_pending(&self, terminal_id: &str, selection_text: &str) {
        let bounded = truncate_selection_safe(selection_text, 12_000);
        let redacted = SecretRedactor::redact(&bounded);
        let mut guard = self.0.lock().unwrap();
        *guard = Some(PendingAiSelectionDto {
            terminal_id: terminal_id.to_string(),
            selection_text: redacted,
        });
    }

    pub fn take_pending(&self) -> Option<PendingAiSelectionDto> {
        self.0.lock().unwrap().take()
    }
}

fn truncate_selection_safe(input: &str, max_bytes: usize) -> String {
    if input.len() <= max_bytes {
        return input.to_string();
    }
    let mut idx = max_bytes;
    while idx > 0 && !input.is_char_boundary(idx) {
        idx -= 1;
    }
    input[..idx].to_string()
}

/// Normaliza un comando para inserción pasiva en PTY eliminando saltos finales sin ejecutar `Enter`.
pub fn normalize_command_for_passive_insert(command: &str) -> Result<String, String> {
    let cleaned = command.trim_matches(|c| c == '\r' || c == '\n').trim();
    if cleaned.is_empty() {
        return Err("El comando a insertar no puede estar vacío.".to_string());
    }
    Ok(cleaned.to_string())
}

/// Construye el contexto efectivo del turno combinando telemetría ambiental no sensible (`profile_name` + `cwd`)
/// con el contexto explícito o el snapshot opt-in, sanitizado por `SecretRedactor`.
pub fn build_effective_turn_context(
    explicit_context: Option<String>,
    snapshot_context: Option<String>,
    shell_info: Option<&ActiveShellInfoDto>,
) -> Option<String> {
    let body = match explicit_context {
        Some(c) if !c.trim().is_empty() => Some(c),
        _ => snapshot_context.filter(|s| !s.trim().is_empty()),
    };

    let env_header = shell_info.map(|sh| match sh.current_cwd.as_deref().map(str::trim) {
        Some(cwd) if !cwd.is_empty() => {
            format!("[Sesión SSH: perfil=\"{}\" | cwd=\"{}\"]", sh.profile_name, cwd)
        }
        _ => format!("[Sesión SSH: perfil=\"{}\"]", sh.profile_name),
    });

    let combined = match (env_header, body) {
        (Some(hdr), Some(ctx)) => format!("{}\n{}", hdr, ctx),
        (Some(hdr), None) => hdr,
        (None, Some(ctx)) => ctx,
        (None, None) => return None,
    };

    Some(SecretRedactor::redact(&combined))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveShellInfoDto {
    pub terminal_id: String,
    pub context_id: String,
    pub profile_name: String,
    pub label: String,
    pub is_connected: bool,
    #[serde(default)]
    pub is_listening: bool,
    #[serde(default)]
    pub captured_lines: usize,
    #[serde(default)]
    pub current_cwd: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveShellsSnapshotDto {
    pub shells: Vec<ActiveShellInfoDto>,
    pub focused_terminal_id: Option<String>,
}

/// Catálogo en memoria de shells abiertos en `"main"` sincronizado hacia `"neko-ai"`.
#[derive(Clone, Default)]
pub struct ActiveShellsCatalog(pub Arc<Mutex<ActiveShellsSnapshotDto>>);

impl ActiveShellsCatalog {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(ActiveShellsSnapshotDto::default())))
    }

    fn hydrate_shells(shells: &mut [ActiveShellInfoDto], pty_taps: &PtyTapRegistry) {
        for sh in shells {
            sh.is_listening = pty_taps.is_listening(&sh.terminal_id);
            sh.captured_lines = pty_taps.captured_line_count(&sh.terminal_id);
            sh.current_cwd = pty_taps.current_cwd(&sh.terminal_id);
        }
    }

    pub fn update_snapshot(
        &self,
        mut shells: Vec<ActiveShellInfoDto>,
        focused_terminal_id: Option<String>,
        pty_taps: &PtyTapRegistry,
    ) -> ActiveShellsSnapshotDto {
        Self::hydrate_shells(&mut shells, pty_taps);
        let snapshot = ActiveShellsSnapshotDto {
            shells,
            focused_terminal_id,
        };
        let mut guard = self.0.lock().unwrap();
        *guard = snapshot.clone();
        snapshot
    }

    pub fn get_snapshot(&self, pty_taps: &PtyTapRegistry) -> ActiveShellsSnapshotDto {
        let mut snap = self.0.lock().unwrap().clone();
        Self::hydrate_shells(&mut snap.shells, pty_taps);
        snap
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAuditResultDto {
    pub computed_risk: RiskLevel,
    pub is_mutating: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStepDispatchDto {
    pub nonce: String,
    pub step_index: usize,
    pub command: String,
    pub phase: HarnessPhase,
}

fn generate_sentinel_nonce() -> String {
    let raw_uuid = Uuid::new_v4().simple().to_string();
    raw_uuid[..12].to_string()
}

/// Prepara y valida de forma pura el disparo del paso actual en Fase 3 (`GuidedExecution`).
pub fn prepare_plan_step_dispatch(
    fsm: &mut AgentHarnessFsm,
    edited_command: Option<String>,
) -> Result<(usize, String, String, String, HarnessPhase), String> {
    let (step_idx, effective_cmd) = match fsm.phase() {
        HarnessPhase::GuidedExecution {
            tasks,
            current_step_index,
            ..
        } => {
            let idx = *current_step_index;
            let current = tasks
                .get(idx)
                .ok_or_else(|| "No hay tarea pendiente en el índice actual del plan".to_string())?;
            (idx, current.command.clone())
        }
        other => {
            return Err(format!(
                "El arnés no está en Fase 3 (GuidedExecution). Estado actual: {:?}",
                other
            ));
        }
    };

    let final_cmd = if let Some(edited) = edited_command {
        let trimmed = edited.trim();
        if trimmed.is_empty() {
            return Err("El comando del paso no puede estar vacío.".to_string());
        }
        let risk = CommandRiskGuard::audit_risk(trimmed);
        let (_, updated_task) = fsm
            .update_current_step_command(trimmed, risk)
            .map_err(|e| e.to_string())?;
        updated_task.command
    } else {
        let risk = CommandRiskGuard::audit_risk(&effective_cmd);
        let (_, updated_task) = fsm
            .update_current_step_command(effective_cmd, risk)
            .map_err(|e| e.to_string())?;
        updated_task.command
    };

    let nonce = generate_sentinel_nonce();
    let wrapped = wrap_command_with_sentinel(&final_cmd, &nonce);
    Ok((step_idx, final_cmd, nonce, wrapped, fsm.phase().clone()))
}

/// Prepara y valida de forma pura el disparo de un `rollback_cmd` quirúrgico para `step_index`.
pub fn prepare_rollback_step_dispatch(
    fsm: &AgentHarnessFsm,
    step_index: usize,
    custom_rollback_cmd: Option<&str>,
) -> Result<(String, String, String, HarnessPhase), String> {
    let effective_cmd = fsm
        .validate_rollback_request(step_index, custom_rollback_cmd)
        .map_err(|e| e.to_string())?;
    let nonce = generate_sentinel_nonce();
    let wrapped = wrap_command_with_sentinel(&effective_cmd, &nonce);
    Ok((effective_cmd, nonce, wrapped, fsm.phase().clone()))
}

/// Aplica el efecto determinista de una interrupción manual `Ctrl+C` cuando había centinela activo.
pub fn apply_manual_interrupt_to_fsm(
    fsm: &mut AgentHarnessFsm,
    taken_sentinel: Option<(String, SentinelPurpose)>,
) -> HarnessPhase {
    if let Some((_, SentinelPurpose::PlanStep { command, .. })) = taken_sentinel {
        let _ = fsm.halt_on_error(
            command,
            130,
            "Ejecución interrumpida manualmente por el operador (Ctrl+C).",
        );
    }
    fsm.phase().clone()
}

#[derive(Debug, Clone, Serialize)]
struct AiTurnProgressPayload {
    session_id: String,
    chunk: String,
}

/// Fábrica desacoplada del driver activo a partir de preferencias universales.
pub fn resolve_active_driver(
    prefs: &AiPreferencesDto,
    sandbox_dir: PathBuf,
) -> Result<Box<dyn AiProviderDriver>, String> {
    match prefs.active_driver.trim().to_ascii_lowercase().as_str() {
        "" | "agy" => {
            let custom_bin = prefs.driver_options.get("agy_binary_path").cloned();
            Ok(Box::new(AgyCliDriver::new(custom_bin, sandbox_dir)))
        }
        other => Err(format!(
            "Proveedor de IA '{}' no soportado todavía en esta versión.",
            other
        )),
    }
}

fn resolve_sandbox_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("No se pudo resolver app_data_dir para ai-sandbox: {}", e))?;
    Ok(data_dir.join("ai-sandbox"))
}

/// Lógica pura de verificación e inyección segura de lectura en Fase 1 (testeable con `Mock` / sin estado global).
pub fn validate_and_prepare_readonly_command(command: &str) -> Result<(String, String), String> {
    ReadOnlyEnforcer::validate_recon_command(command)?;
    let risk = CommandRiskGuard::audit_risk(command);
    if risk.is_mutating() {
        return Err(format!(
            "Bloqueado por guardia de Fase 1: el comando fue clasificado como '{}'",
            risk.as_str()
        ));
    }
    let raw_uuid = Uuid::new_v4().simple().to_string();
    let nonce = raw_uuid[..12].to_string();
    let wrapped = wrap_command_with_sentinel(command, &nonce);
    Ok((nonce, wrapped))
}

#[tauri::command]
pub async fn open_or_focus_ai_copilot_window_cmd(app: AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window(AI_COPILOT_WINDOW_LABEL) {
        let _ = existing.show();
        let _ = existing.unminimize();
        existing
            .set_focus()
            .map_err(|e| format!("No se pudo enfocar ventana NekoAI: {}", e))?;
        return Ok(());
    }

    let win = WebviewWindowBuilder::new(
        &app,
        AI_COPILOT_WINDOW_LABEL,
        WebviewUrl::App("ai-copilot.html".into()),
    )
    .title("NekoAI — Copiloto SSH")
    .inner_size(500.0, 760.0)
    .min_inner_size(390.0, 560.0)
    .resizable(true)
    .build()
    .map_err(|e| format!("No se pudo abrir ventana NekoAI: {}", e))?;

    let _ = win.set_focus();
    Ok(())
}

#[tauri::command]
pub async fn set_ai_window_always_on_top_cmd(
    app: AppHandle,
    always_on_top: bool,
) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(AI_COPILOT_WINDOW_LABEL) {
        win.set_always_on_top(always_on_top)
            .map_err(|e| format!("No se pudo cambiar Always on Top: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn notify_ai_theme_changed_cmd(app: AppHandle, theme_id: String) -> Result<(), String> {
    let _ = app.emit("nekossh-theme-changed", theme_id);
    Ok(())
}

#[tauri::command]
pub async fn sync_active_shells_cmd(
    app: AppHandle,
    shells: Vec<ActiveShellInfoDto>,
    focused_terminal_id: Option<String>,
    catalog: State<'_, ActiveShellsCatalog>,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<(), String> {
    let snap = catalog.update_snapshot(shells, focused_terminal_id, &pty_taps);
    let _ = app.emit("ai-shells-updated", snap);
    Ok(())
}

#[tauri::command]
pub async fn get_active_shells_cmd(
    catalog: State<'_, ActiveShellsCatalog>,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<ActiveShellsSnapshotDto, String> {
    Ok(catalog.get_snapshot(&pty_taps))
}

#[tauri::command]
pub async fn set_pty_listening_cmd(
    app: AppHandle,
    terminal_id: String,
    enabled: bool,
    pty_taps: State<'_, PtyTapRegistry>,
    catalog: State<'_, ActiveShellsCatalog>,
) -> Result<(), String> {
    pty_taps.set_listening(&terminal_id, enabled);
    let snap = catalog.get_snapshot(&pty_taps);
    let _ = app.emit("ai-shells-updated", snap);
    if enabled {
        let _ = app.emit("ai-request-visible-buffer", terminal_id);
    }
    Ok(())
}

#[tauri::command]
pub async fn seed_pty_visible_screen_cmd(
    app: AppHandle,
    terminal_id: String,
    visible_text: String,
    pty_taps: State<'_, PtyTapRegistry>,
    catalog: State<'_, ActiveShellsCatalog>,
) -> Result<(), String> {
    pty_taps.seed_visible_screen(&terminal_id, &visible_text);
    let snap = catalog.get_snapshot(&pty_taps);
    let _ = app.emit("ai-shells-updated", snap);
    Ok(())
}

/// Inserta un comando limpio en el prompt del PTY activo **sin** agregar salto de línea ni centinela.
#[tauri::command]
pub async fn insert_ai_command_to_pty_cmd(
    terminal_id: String,
    command: String,
    ssh_state: State<'_, crate::SshConnections>,
) -> Result<(), String> {
    let cleaned = normalize_command_for_passive_insert(&command)?;
    crate::write_ssh_raw(&ssh_state, &terminal_id, &cleaned)
}

/// Envía una selección de terminal hacia `#copilot-prompt-input` de `neko-ai` incluso con `Escucha OFF`.
#[tauri::command]
pub async fn send_selection_to_ai_cmd(
    app: AppHandle,
    terminal_id: String,
    selection_text: String,
    sel_queue: State<'_, AiSelectionQueue>,
) -> Result<(), String> {
    if selection_text.trim().is_empty() {
        return Ok(());
    }
    sel_queue.set_pending(&terminal_id, &selection_text);
    let payload = sel_queue.0.lock().unwrap().clone();
    open_or_focus_ai_copilot_window_cmd(app.clone()).await?;
    if let Some(dto) = payload {
        let _ = app.emit("ai-composer-selection", dto);
    }
    Ok(())
}

/// Extrae y consume la selección pendiente en cola (anti-race al abrir `neko-ai` por primera vez).
#[tauri::command]
pub async fn take_pending_ai_selection_cmd(
    sel_queue: State<'_, AiSelectionQueue>,
) -> Result<Option<PendingAiSelectionDto>, String> {
    Ok(sel_queue.take_pending())
}

/// Ejecución soberana unificada de un comando autorizado por el operador (o lectura `🟢` en Modo Flexible).
/// No rechaza por listas blancas: envuelve con centinela POSIX inmune a eco TTY e inyecta en el PTY.
#[tauri::command]
pub async fn execute_ai_command_cmd(
    terminal_id: String,
    command: String,
    purpose: Option<String>,
    ssh_state: State<'_, crate::SshConnections>,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<String, String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err("El comando a ejecutar no puede estar vacío.".to_string());
    }
    let nonce = generate_sentinel_nonce();
    let wrapped = wrap_command_with_sentinel(trimmed, &nonce);

    pty_taps.try_arm_sentinel(
        &terminal_id,
        nonce.clone(),
        SentinelPurpose::AuthorizedCommand {
            command: trimmed.to_string(),
            purpose: purpose.unwrap_or_default(),
        },
    )?;

    if let Err(e) = crate::write_ssh_raw(&ssh_state, &terminal_id, &wrapped) {
        pty_taps.clear_sentinel(&terminal_id);
        return Err(e);
    }
    Ok(nonce)
}

#[tauri::command]
pub async fn get_ai_feed_cmd(
    session_id: String,
    ai_feeds: State<'_, AiUiFeedRegistry>,
) -> Result<Vec<AiFeedEntryDto>, String> {
    Ok(ai_feeds.get(&session_id))
}

#[tauri::command]
pub async fn append_ai_feed_cmd(
    session_id: String,
    entry: AiFeedEntryDto,
    ai_feeds: State<'_, AiUiFeedRegistry>,
) -> Result<(), String> {
    ai_feeds.append(&session_id, entry);
    Ok(())
}

#[tauri::command]
pub async fn get_ai_session_ui_state_cmd(
    session_id: String,
    ai_feeds: State<'_, AiUiFeedRegistry>,
) -> Result<AiUiSessionStateDto, String> {
    Ok(ai_feeds.get_session_state(&session_id))
}

#[tauri::command]
pub async fn save_ai_session_ui_state_cmd(
    session_id: String,
    ui_state: AiUiSessionStateDto,
    ai_feeds: State<'_, AiUiFeedRegistry>,
) -> Result<(), String> {
    ai_feeds.set_session_state(&session_id, ui_state);
    Ok(())
}

#[tauri::command]
pub async fn clear_ai_conversation_cmd(
    session_id: String,
    state: State<'_, HarnessRegistry>,
    cancel_state: State<'_, AiTurnCancelRegistry>,
    pty_taps: State<'_, PtyTapRegistry>,
    ai_feeds: State<'_, AiUiFeedRegistry>,
) -> Result<HarnessPhase, String> {
    cancel_state.cancel_session(&session_id);
    pty_taps.clear_sentinel(&session_id);
    ai_feeds.clear(&session_id);
    Ok(state.with_or_create(&session_id, ExecutionMode::Strict, |fsm| {
        fsm.cancel_mission();
        fsm.phase().clone()
    }))
}

#[tauri::command]
pub async fn get_pty_context_snapshot_cmd(
    app: AppHandle,
    terminal_id: String,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<String, String> {
    let max_lines = crate::get_db_conn(&app)
        .and_then(|conn| get_ai_preferences(&conn))
        .map(|p| p.context_max_lines)
        .unwrap_or(DEFAULT_CONTEXT_MAX_LINES);
    let raw = pty_taps.get_context_snapshot(&terminal_id, max_lines);
    Ok(SecretRedactor::redact(&raw))
}

#[tauri::command]
pub async fn execute_ai_readonly_step_cmd(
    terminal_id: String,
    command: String,
    ssh_state: State<'_, crate::SshConnections>,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<String, String> {
    let (nonce, wrapped) = validate_and_prepare_readonly_command(&command)?;

    pty_taps.try_arm_sentinel(
        &terminal_id,
        nonce.clone(),
        SentinelPurpose::ReadOnlyRecon {
            command: command.trim().to_string(),
        },
    )?;
    if let Err(e) = crate::write_ssh_raw(&ssh_state, &terminal_id, &wrapped) {
        pty_taps.clear_sentinel(&terminal_id);
        return Err(e);
    }
    Ok(nonce)
}

#[tauri::command]
pub async fn execute_ai_plan_step_cmd(
    terminal_id: String,
    edited_command: Option<String>,
    state: State<'_, HarnessRegistry>,
    ssh_state: State<'_, crate::SshConnections>,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<PlanStepDispatchDto, String> {
    if pty_taps.has_active_sentinel(&terminal_id) {
        return Err(
            "Ya existe un comando supervisado en ejecución esperando confirmación en esta terminal."
                .to_string(),
        );
    }

    let (step_index, final_cmd, nonce, wrapped, phase) =
        state.with_or_create(&terminal_id, ExecutionMode::Strict, |fsm| {
            prepare_plan_step_dispatch(fsm, edited_command)
        })?;

    pty_taps.try_arm_sentinel(
        &terminal_id,
        nonce.clone(),
        SentinelPurpose::PlanStep {
            step_index,
            command: final_cmd.clone(),
        },
    )?;

    if let Err(e) = crate::write_ssh_raw(&ssh_state, &terminal_id, &wrapped) {
        pty_taps.clear_sentinel(&terminal_id);
        return Err(e);
    }

    Ok(PlanStepDispatchDto {
        nonce,
        step_index,
        command: final_cmd,
        phase,
    })
}

#[tauri::command]
pub async fn interrupt_ai_pty_step_cmd(
    terminal_id: String,
    state: State<'_, HarnessRegistry>,
    ssh_state: State<'_, crate::SshConnections>,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<HarnessPhase, String> {
    let write_res = crate::write_ssh_raw(&ssh_state, &terminal_id, "\x03");
    let taken = pty_taps.take_sentinel(&terminal_id);
    let phase = state.with_or_create(&terminal_id, ExecutionMode::Strict, |fsm| {
        apply_manual_interrupt_to_fsm(fsm, taken)
    });
    if let Err(e) = write_res {
        return Err(format!("No se pudo enviar Ctrl+C al PTY: {}", e));
    }
    Ok(phase)
}

#[tauri::command]
pub async fn retry_halted_step_cmd(
    session_id: String,
    fixed_command: Option<String>,
    state: State<'_, HarnessRegistry>,
) -> Result<HarnessPhase, String> {
    state.with_or_create(&session_id, ExecutionMode::Strict, |fsm| {
        fsm.resume_from_error(fixed_command)
            .map_err(|e| e.to_string())?;
        Ok(fsm.phase().clone())
    })
}

#[tauri::command]
pub async fn execute_ai_rollback_step_cmd(
    terminal_id: String,
    step_index: usize,
    custom_rollback_cmd: Option<String>,
    state: State<'_, HarnessRegistry>,
    ssh_state: State<'_, crate::SshConnections>,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<PlanStepDispatchDto, String> {
    if pty_taps.has_active_sentinel(&terminal_id) {
        return Err(
            "Ya existe un comando supervisado en ejecución esperando confirmación en esta terminal."
                .to_string(),
        );
    }

    let (effective_cmd, nonce, wrapped, phase) =
        state.with_or_create(&terminal_id, ExecutionMode::Strict, |fsm| {
            prepare_rollback_step_dispatch(fsm, step_index, custom_rollback_cmd.as_deref())
        })?;

    pty_taps.try_arm_sentinel(
        &terminal_id,
        nonce.clone(),
        SentinelPurpose::StepRollback {
            step_index,
            rollback_cmd: effective_cmd.clone(),
        },
    )?;

    if let Err(e) = crate::write_ssh_raw(&ssh_state, &terminal_id, &wrapped) {
        pty_taps.clear_sentinel(&terminal_id);
        return Err(e);
    }

    Ok(PlanStepDispatchDto {
        nonce,
        step_index,
        command: effective_cmd,
        phase,
    })
}

#[tauri::command]
pub async fn get_harness_phase_cmd(
    session_id: String,
    state: State<'_, HarnessRegistry>,
) -> Result<HarnessPhase, String> {
    Ok(state.with_or_create(&session_id, ExecutionMode::Strict, |fsm| fsm.phase().clone()))
}

#[tauri::command]
pub async fn start_harness_recon_cmd(
    session_id: String,
    objective: String,
    state: State<'_, HarnessRegistry>,
    app: AppHandle,
) -> Result<(), String> {
    let mode = {
        let conn = crate::get_db_conn(&app)?;
        get_ai_preferences(&conn)?.execution_mode
    };

    state.with_or_create(&session_id, mode, |fsm| {
        fsm.set_mode(mode);
        fsm.start_recon(objective).map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn approve_harness_plan_cmd(
    session_id: String,
    state: State<'_, HarnessRegistry>,
) -> Result<HarnessPhase, String> {
    state.with_or_create(&session_id, ExecutionMode::Strict, |fsm| {
        fsm.approve_plan_and_start_execution()
            .map_err(|e| e.to_string())?;
        Ok(fsm.phase().clone())
    })
}

#[tauri::command]
pub async fn cancel_harness_mission_cmd(
    session_id: String,
    state: State<'_, HarnessRegistry>,
    cancel_state: State<'_, AiTurnCancelRegistry>,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<HarnessPhase, String> {
    cancel_state.cancel_session(&session_id);
    pty_taps.clear_sentinel(&session_id);
    Ok(state.with_or_create(&session_id, ExecutionMode::Strict, |fsm| {
        fsm.cancel_mission();
        fsm.phase().clone()
    }))
}

#[tauri::command]
pub async fn validate_recon_command_cmd(command: String) -> Result<(), String> {
    ReadOnlyEnforcer::validate_recon_command(&command)
}

#[tauri::command]
pub async fn audit_command_risk_cmd(command: String) -> Result<RiskAuditResultDto, String> {
    let computed_risk = CommandRiskGuard::audit_risk(&command);
    Ok(RiskAuditResultDto {
        computed_risk,
        is_mutating: computed_risk.is_mutating(),
    })
}

#[tauri::command]
pub async fn redact_secrets_cmd(text: String) -> Result<String, String> {
    Ok(SecretRedactor::redact(&text))
}

#[tauri::command]
pub async fn get_ai_preferences_cmd(
    app: AppHandle,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<AiPreferencesDto, String> {
    let conn = crate::get_db_conn(&app)?;
    let prefs = get_ai_preferences(&conn)?;
    pty_taps.set_max_capture_lines(prefs.context_max_lines);
    Ok(prefs)
}

#[tauri::command]
pub async fn set_ai_preferences_cmd(
    app: AppHandle,
    prefs: AiPreferencesDto,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<(), String> {
    let conn = crate::get_db_conn(&app)?;
    set_ai_preferences(&conn, &prefs)?;
    pty_taps.set_max_capture_lines(clamp_context_max_lines(prefs.context_max_lines));
    Ok(())
}

#[tauri::command]
pub async fn reset_ai_preferences_cmd(
    app: AppHandle,
    pty_taps: State<'_, PtyTapRegistry>,
) -> Result<AiPreferencesDto, String> {
    let conn = crate::get_db_conn(&app)?;
    let defaults = reset_ai_preferences(&conn)?;
    pty_taps.set_max_capture_lines(defaults.context_max_lines);
    Ok(defaults)
}

#[tauri::command]
pub async fn pause_harness_on_disconnect_cmd(
    session_id: String,
    state: State<'_, HarnessRegistry>,
) -> Result<(), String> {
    state.with_or_create(&session_id, ExecutionMode::Strict, |fsm| {
        fsm.pause_on_disconnect().map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn resume_harness_on_reconnect_cmd(
    session_id: String,
    state: State<'_, HarnessRegistry>,
) -> Result<HarnessPhase, String> {
    state.with_or_create(&session_id, ExecutionMode::Strict, |fsm| {
        fsm.resume_after_reconnect().map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub async fn probe_ai_driver_health_cmd(app: AppHandle) -> Result<DriverHealthStatus, String> {
    let prefs = {
        let conn = crate::get_db_conn(&app)?;
        get_ai_preferences(&conn)?
    };
    let sandbox_dir = resolve_sandbox_dir(&app)?;
    let driver = resolve_active_driver(&prefs, sandbox_dir)?;

    tauri::async_runtime::spawn_blocking(move || Ok(driver.probe_health()))
        .await
        .map_err(|e| format!("Fallo en hilo de diagnóstico AI: {}", e))?
}

#[tauri::command]
pub async fn list_ai_models_cmd(app: AppHandle) -> Result<Vec<AiModelDescriptor>, String> {
    let prefs = {
        let conn = crate::get_db_conn(&app)?;
        get_ai_preferences(&conn)?
    };
    let sandbox_dir = resolve_sandbox_dir(&app)?;
    let driver = resolve_active_driver(&prefs, sandbox_dir)?;

    tauri::async_runtime::spawn_blocking(move || driver.list_models().map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("Fallo listando modelos AI: {}", e))?
}

#[tauri::command]
pub async fn run_ai_harness_turn_cmd(
    app: AppHandle,
    session_id: String,
    user_prompt: String,
    terminal_context: Option<String>,
    state: State<'_, HarnessRegistry>,
    cancel_state: State<'_, AiTurnCancelRegistry>,
    pty_taps: State<'_, PtyTapRegistry>,
    catalog: State<'_, ActiveShellsCatalog>,
) -> Result<TurnExecutionResultDto, String> {
    let prefs = {
        let conn = crate::get_db_conn(&app)?;
        get_ai_preferences(&conn)?
    };
    let sandbox_dir = resolve_sandbox_dir(&app)?;
    let driver = resolve_active_driver(&prefs, sandbox_dir)?;

    let snap = catalog.get_snapshot(&pty_taps);
    let target_shell = snap.shells.iter().find(|s| s.terminal_id == session_id);
    let tap_snapshot = if terminal_context.as_deref().map(str::trim).is_some_and(|s| !s.is_empty()) {
        None
    } else {
        Some(pty_taps.get_context_snapshot(&session_id, prefs.context_max_lines))
    };
    let effective_context = build_effective_turn_context(terminal_context, tap_snapshot, target_shell);

    let registry = state.inner().clone();
    let cancel_reg = cancel_state.inner().clone();
    let app_clone = app.clone();
    let sid_clone = session_id.clone();

    tauri::async_runtime::spawn_blocking(move || {
        HarnessOrchestrator::execute_turn(
            &sid_clone,
            &user_prompt,
            effective_context.as_deref(),
            &prefs,
            &registry,
            &cancel_reg,
            driver.as_ref(),
            &|chunk| {
                let _ = app_clone.emit(
                    "ai-turn-progress",
                    AiTurnProgressPayload {
                        session_id: sid_clone.clone(),
                        chunk: chunk.to_string(),
                    },
                );
            },
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Fallo ejecutando turno del arnés AI: {}", e))?
}

#[tauri::command]
pub async fn cancel_ai_turn_cmd(
    session_id: String,
    cancel_state: State<'_, AiTurnCancelRegistry>,
) -> Result<bool, String> {
    Ok(cancel_state.cancel_session(&session_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::schema::AgentTaskProposal;

    #[test]
    fn registry_crea_fsm_independiente_por_sesion() {
        let registry = HarnessRegistry::new();
        let phase1 = registry.with_or_create("term-1", ExecutionMode::Strict, |fsm| {
            fsm.start_recon("Meta 1").unwrap();
            fsm.phase().clone()
        });
        assert!(matches!(phase1, HarnessPhase::Reconnaissance { .. }));

        let phase2 = registry.with_or_create("term-2", ExecutionMode::Flexible, |fsm| {
            fsm.phase().clone()
        });
        assert_eq!(phase2, HarnessPhase::Idle);

        let phase1_again = registry.with_or_create("term-1", ExecutionMode::Strict, |fsm| {
            fsm.phase().clone()
        });
        assert_eq!(phase1, phase1_again);
    }

    #[test]
    fn comandos_ipc_validacion_y_auditoria() {
        assert!(tauri::async_runtime::block_on(validate_recon_command_cmd("cat /proc/cpuinfo".to_string())).is_ok());
        assert!(tauri::async_runtime::block_on(validate_recon_command_cmd("rm -rf /".to_string())).is_err());

        let audit = tauri::async_runtime::block_on(audit_command_risk_cmd("curl -fsSL https://pkg.com | bash".to_string())).unwrap();
        assert_eq!(audit.computed_risk, RiskLevel::Destructive);
        assert!(audit.is_mutating);

        let redacted = tauri::async_runtime::block_on(redact_secrets_cmd("API_KEY=secret_key_12345".to_string())).unwrap();
        assert!(redacted.contains("[REDACTED_SECRET]"));
        assert!(!redacted.contains("secret_key_12345"));
    }

    #[test]
    fn resolve_active_driver_resuelve_agy_y_rechaza_desconocido() {
        let prefs = AiPreferencesDto::default();
        let driver = resolve_active_driver(&prefs, std::env::temp_dir()).expect("agy ok");
        assert_eq!(driver.driver_id(), "agy");

        let bad_prefs = AiPreferencesDto {
            active_driver: "desconocido".to_string(),
            ..Default::default()
        };
        let err = match resolve_active_driver(&bad_prefs, std::env::temp_dir()) {
            Ok(_) => panic!("debe fallar con driver no soportado"),
            Err(e) => e,
        };
        assert!(err.contains("no soportado"));
    }

    #[test]
    fn validate_and_prepare_readonly_step_bloquea_mutaciones_y_envuelve_lecturas() {
        let (nonce, wrapped) = validate_and_prepare_readonly_command("free -h && ss -tulpn").expect("ok");
        assert_eq!(nonce.len(), 12);
        assert!(!wrapped.contains(&format!("__NEKO_END_{}:", nonce)));
        assert!(wrapped.contains("'__NEKO_END'") && wrapped.contains(&nonce));

        let err = validate_and_prepare_readonly_command("apt-get install -y nginx").unwrap_err();
        assert!(err.contains("Fase 1") || err.contains("Reconocimiento"));
    }

    #[test]
    fn fase_3_dispatch_ctrl_c_reintento_y_rollback_quirurgico() {
        let mut fsm = AgentHarnessFsm::new("t-p3", ExecutionMode::Flexible);
        fsm.start_recon("Configurar Nginx").unwrap();
        fsm.advance_to_plan(vec![
            AgentTaskProposal::new(
                1,
                "Respaldo conf",
                "Seguridad",
                "cp /etc/nginx/nginx.conf /etc/nginx/nginx.conf.bak",
                RiskLevel::ConfigChange,
                "ls /etc/nginx",
                "rm -f /etc/nginx/nginx.conf.bak",
            ),
            AgentTaskProposal::new(
                2,
                "Recargar servicio",
                "Aplicar cambios",
                "systemctl reload nginx",
                RiskLevel::ServiceImpact,
                "nginx -t",
                "cp /etc/nginx/nginx.conf.bak /etc/nginx/nginx.conf && systemctl reload nginx",
            ),
        ])
        .unwrap();
        fsm.approve_plan_and_start_execution().unwrap();

        // 1. Preparar paso 0 con edición inline que eleva riesgo a Destructive
        let (idx, cmd, nonce, wrapped, _) = prepare_plan_step_dispatch(
            &mut fsm,
            Some("cp /etc/nginx/nginx.conf /etc/nginx/nginx.conf.bak ; echo ok | sh".to_string()),
        )
        .expect("dispatch 0");
        assert_eq!(idx, 0);
        assert_eq!(nonce.len(), 12);
        assert!(wrapped.contains(&cmd));
        match fsm.phase() {
            HarnessPhase::GuidedExecution { tasks, .. } => {
                assert_eq!(tasks[0].risk_level, RiskLevel::Destructive);
            }
            _ => panic!("debe seguir en GuidedExecution"),
        }

        // Completar paso 0 y simular interrupción Ctrl+C en paso 1
        fsm.complete_current_step().unwrap();
        let phase_interrupted = apply_manual_interrupt_to_fsm(
            &mut fsm,
            Some((
                "n999".to_string(),
                SentinelPurpose::PlanStep {
                    step_index: 1,
                    command: "systemctl reload nginx".to_string(),
                },
            )),
        );
        match phase_interrupted {
            HarnessPhase::HaltedOnError {
                step_index,
                exit_code,
                ..
            } => {
                assert_eq!(step_index, 1);
                assert_eq!(exit_code, 130);
            }
            other => panic!("Esperaba HaltedOnError 130, llegó {:?}", other),
        }

        // Preparar rollback del paso 1 y luego del paso 0 completado
        let (rb_cmd1, _, _, _) = prepare_rollback_step_dispatch(&fsm, 1, None).expect("rb 1");
        assert!(rb_cmd1.contains("nginx.conf.bak"));
        fsm.mark_step_rolled_back(1).unwrap();

        let (rb_cmd0, _, _, _) = prepare_rollback_step_dispatch(&fsm, 0, None).expect("rb 0");
        assert_eq!(rb_cmd0, "rm -f /etc/nginx/nginx.conf.bak");

        // Reintentar paso 1 con comando corregido
        fsm.resume_from_error(Some("nginx -t && systemctl reload nginx".to_string()))
            .unwrap();
        assert!(matches!(
            fsm.phase(),
            HarnessPhase::GuidedExecution {
                current_step_index: 1,
                ..
            }
        ));
    }

    #[test]
    fn active_shells_catalog_sincroniza_escucha_y_foco() {
        let pty_taps = PtyTapRegistry::new();
        pty_taps.set_listening("term-1-s1", true);

        let catalog = ActiveShellsCatalog::new();
        let snap = catalog.update_snapshot(
            vec![
                ActiveShellInfoDto {
                    terminal_id: "term-1".to_string(),
                    context_id: "term-1".to_string(),
                    profile_name: "Prod".to_string(),
                    label: "Principal".to_string(),
                    is_connected: true,
                    is_listening: false,
                    captured_lines: 0,
                    current_cwd: None,
                },
                ActiveShellInfoDto {
                    terminal_id: "term-1-s1".to_string(),
                    context_id: "term-1".to_string(),
                    profile_name: "Prod".to_string(),
                    label: "Shell 2".to_string(),
                    is_connected: true,
                    is_listening: false,
                    captured_lines: 0,
                    current_cwd: None,
                },
            ],
            Some("term-1-s1".to_string()),
            &pty_taps,
        );

        assert!(!snap.shells[0].is_listening);
        assert!(snap.shells[1].is_listening);
        assert_eq!(snap.focused_terminal_id.as_deref(), Some("term-1-s1"));
    }

    #[test]
    fn approve_y_cancel_mission_actualizan_fsm() {
        let registry = HarnessRegistry::new();
        registry.with_or_create("t1", ExecutionMode::Strict, |fsm| {
            fsm.start_recon("Tomcat").unwrap();
            fsm.advance_to_plan(vec![AgentTaskProposal::new(
                1,
                "Check",
                "Why",
                "free -h",
                RiskLevel::ReadOnly,
                "",
                "",
            )])
            .unwrap();
            fsm.approve_plan_and_start_execution().unwrap();
            assert!(matches!(fsm.phase(), HarnessPhase::GuidedExecution { .. }));
            fsm.cancel_mission();
            assert_eq!(*fsm.phase(), HarnessPhase::Idle);
        });
    }

    #[test]
    fn ai_ui_feed_registry_guarda_acota_y_limpia_por_sesion() {
        let feeds = AiUiFeedRegistry::new();
        for idx in 0..(MAX_FEED_ENTRIES_PER_SESSION + 10) {
            feeds.append(
                "term-a",
                AiFeedEntryDto {
                    kind: "assistant".to_string(),
                    title: format!("NekoAI #{}", idx),
                    body: "ok".to_string(),
                    code: None,
                    exit_code: None,
                    expanded: None,
                    phase_label: Some("Diagnóstico".to_string()),
                },
            );
        }

        let got = feeds.get("term-a");
        assert_eq!(got.len(), MAX_FEED_ENTRIES_PER_SESSION);
        assert_eq!(got.first().unwrap().title, "NekoAI #10");

        feeds.clear("term-a");
        assert!(feeds.get("term-a").is_empty());
    }

    #[test]
    fn cierre_aislado_de_neko_ai_cancela_turnos_y_apaga_escucha() {
        let ai_cancels = AiTurnCancelRegistry::new();
        let token = ai_cancels.register("term-1");
        assert!(!token.is_cancelled());

        let pty_taps = PtyTapRegistry::new();
        pty_taps.set_listening("term-1", true);
        pty_taps.seed_visible_screen("term-1", "root@vps:~# ls\n");

        let catalog = ActiveShellsCatalog::new();
        catalog.update_snapshot(
            vec![ActiveShellInfoDto {
                terminal_id: "term-1".to_string(),
                context_id: "term-1".to_string(),
                profile_name: "Prod".to_string(),
                label: "Principal".to_string(),
                is_connected: true,
                is_listening: true,
                captured_lines: 0,
                current_cwd: None,
            }],
            Some("term-1".to_string()),
            &pty_taps,
        );

        let snap = handle_copilot_window_close_state(&ai_cancels, &pty_taps, &catalog)
            .expect("debe emitir snapshot actualizado porque había escucha ON");

        assert!(token.is_cancelled());
        assert!(!pty_taps.is_listening("term-1"));
        assert_eq!(pty_taps.get_context_snapshot("term-1", 50), "");
        assert!(!snap.shells[0].is_listening);
    }

    #[test]
    fn ai_ui_feed_registry_preserva_estado_hitl_por_sesion() {
        let feeds = AiUiFeedRegistry::new();
        let state_a = AiUiSessionStateDto {
            pending_action: Some(AiPendingActionDto {
                purpose: "reiniciar nginx".to_string(),
                original_command: "systemctl restart nginx".to_string(),
                draft_command: "systemctl restart nginx && nginx -t".to_string(),
                is_editing: true,
                risk_level: "service_impact".to_string(),
                audit_warning: Some("Afecta servicio en vivo".to_string()),
                backup_strategy: Some("cp conf".to_string()),
                rollback_cmd: Some("cp conf.bak conf".to_string()),
            }),
            last_failed: None,
            last_rollback: Some(AiRollbackDto {
                source_command: "cp a b".to_string(),
                rollback_cmd: "rm -f b".to_string(),
                label: "backup".to_string(),
            }),
        };

        feeds.set_session_state("term-a", state_a.clone());
        assert_eq!(feeds.get_session_state("term-a"), state_a);
        assert_eq!(feeds.get_session_state("term-b"), AiUiSessionStateDto::default());

        feeds.clear("term-a");
        assert_eq!(feeds.get_session_state("term-a"), AiUiSessionStateDto::default());
    }

    #[test]
    fn telemetria_ambiental_y_canales_rapidos_de_insercion_y_seleccion() {
        let pty_taps = PtyTapRegistry::new();
        pty_taps.set_listening("term-ux", true);
        pty_taps.process_chunk("term-ux", "\x1b]7;file://prod/etc/nginx\x07line1\nline2\n");

        let catalog = ActiveShellsCatalog::new();
        let snap = catalog.update_snapshot(
            vec![ActiveShellInfoDto {
                terminal_id: "term-ux".to_string(),
                context_id: "term-ux".to_string(),
                profile_name: "Prod Nginx".to_string(),
                label: "Principal".to_string(),
                is_connected: true,
                is_listening: false,
                captured_lines: 0,
                current_cwd: None,
            }],
            Some("term-ux".to_string()),
            &pty_taps,
        );

        assert_eq!(snap.shells[0].captured_lines, 2);
        assert_eq!(snap.shells[0].current_cwd.as_deref(), Some("/etc/nginx"));

        let ctx = build_effective_turn_context(
            None,
            Some("line1\nline2".to_string()),
            snap.shells.first(),
        )
        .expect("contexto construido");
        assert!(ctx.contains("Prod Nginx"));
        assert!(ctx.contains("/etc/nginx"));
        assert!(ctx.contains("line1\nline2"));

        assert_eq!(
            normalize_command_for_passive_insert("  nginx -t  \r\n").unwrap(),
            "nginx -t"
        );
        assert!(normalize_command_for_passive_insert("  \r\n ").is_err());

        let sel_queue = AiSelectionQueue::new();
        sel_queue.set_pending("term-ux", "error en puerto 443");
        let pulled = sel_queue.take_pending().expect("selección disponible");
        assert_eq!(pulled.terminal_id, "term-ux");
        assert!(pulled.selection_text.contains("error en puerto 443"));
        assert!(sel_queue.take_pending().is_none());
    }
}
