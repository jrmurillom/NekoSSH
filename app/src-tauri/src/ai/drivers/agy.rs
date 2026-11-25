//! Adaptador aislado para `agy` CLI (`AgyCliDriver`).
//! Encierra toda la interacción con `agy.EXE` / `agy` en modo headless JSON puro dentro de sandbox.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::Value;
use uuid::Uuid;

use crate::ai::driver::{
    AgentTurnOutcome, AgentTurnRequest, AiDriverError, AiModelDescriptor, AiProviderDriver,
    DriverHealthStatus, TurnCancellationToken,
};
use crate::ai::schema::{AgentResponseKind, AgentTurnWireEnvelope, ConversationRole};

/// Tope seguro para argumento `--print` en Windows `CreateProcessW` (límite 32,767 chars).
const MAX_CLI_PROMPT_CHARS: usize = 20_000;

#[cfg(windows)]
fn apply_no_window(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn apply_no_window(_cmd: &mut Command) {}

/// Termina recursivamente el árbol de procesos hijo (vital en Windows cuando `cmd /C` envuelve `.cmd`/`.bat`)
/// sin dejar subprocesos huérfanos reteniendo pipes abiertos.
fn terminate_process_tree(child: &mut std::process::Child) {
    #[cfg(windows)]
    {
        let pid = child.id().to_string();
        let mut kill_cmd = Command::new("taskkill");
        kill_cmd.args(["/F", "/T", "/PID", &pid]);
        kill_cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        apply_no_window(&mut kill_cmd);
        let _ = kill_cmd.status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

/// Guardia RAII para archivo temporal de `--json-schema` dentro del sandbox.
struct TempSchemaFile {
    path: PathBuf,
}

impl TempSchemaFile {
    fn create(sandbox_dir: &Path, schema: &Value) -> Result<Self, AiDriverError> {
        std::fs::create_dir_all(sandbox_dir).map_err(|e| {
            AiDriverError::ExecutionFailed(format!(
                "No se pudo crear el directorio sandbox '{}': {}",
                sandbox_dir.display(),
                e
            ))
        })?;
        let path = sandbox_dir.join(format!("turn-schema-{}.json", Uuid::new_v4()));
        let content = serde_json::to_string_pretty(schema).map_err(|e| {
            AiDriverError::ExecutionFailed(format!("Error al serializar JSON Schema: {}", e))
        })?;
        std::fs::write(&path, content).map_err(|e| {
            AiDriverError::ExecutionFailed(format!(
                "No se pudo escribir esquema temporal '{}': {}",
                path.display(),
                e
            ))
        })?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempSchemaFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Adaptador productivo para Google Antigravity CLI (`agy`).
#[derive(Debug, Clone)]
pub struct AgyCliDriver {
    binary_override: Option<String>,
    sandbox_dir: PathBuf,
}

impl AgyCliDriver {
    pub fn new(binary_override: Option<String>, sandbox_dir: PathBuf) -> Self {
        let cleaned = binary_override
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Self {
            binary_override: cleaned,
            sandbox_dir,
        }
    }

    pub fn sandbox_dir(&self) -> &Path {
        &self.sandbox_dir
    }

    /// Resuelve la ruta del ejecutable `agy` priorizando la ruta configurada, luego `PATH` y fallback local en Windows.
    pub fn resolve_binary(&self) -> Option<PathBuf> {
        if let Some(ref custom) = self.binary_override {
            let candidate = PathBuf::from(custom);
            if candidate.is_file() {
                return Some(candidate);
            }
            return None;
        }

        if let Some(path_var) = std::env::var_os("PATH") {
            let candidates: &[&str] = if cfg!(windows) {
                &["agy.EXE", "agy.exe", "agy.cmd", "agy.bat", "agy"]
            } else {
                &["agy"]
            };

            for dir in std::env::split_paths(&path_var) {
                for name in candidates {
                    let full = dir.join(name);
                    if full.is_file() {
                        return Some(full);
                    }
                }
            }
        }

        #[cfg(windows)]
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            let fallback = PathBuf::from(local_app_data)
                .join("agy")
                .join("bin")
                .join("agy.EXE");
            if fallback.is_file() {
                return Some(fallback);
            }
        }

        None
    }

    fn build_base_command(&self, bin: &Path, args: &[String]) -> Result<Command, AiDriverError> {
        std::fs::create_dir_all(&self.sandbox_dir).map_err(|e| {
            AiDriverError::ExecutionFailed(format!(
                "No se pudo preparar sandbox '{}': {}",
                self.sandbox_dir.display(),
                e
            ))
        })?;

        let is_batch_shim = bin
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "cmd" | "bat"))
            .unwrap_or(false);

        let mut cmd = if is_batch_shim {
            let mut c = Command::new("cmd");
            c.arg("/C").arg(bin);
            c.args(args);
            c
        } else {
            let mut c = Command::new(bin);
            c.args(args);
            c
        };

        cmd.current_dir(&self.sandbox_dir);
        apply_no_window(&mut cmd);
        Ok(cmd)
    }
}

/// Construye los argumentos headless confinados para `agy` (sin `--dangerously-skip-permissions`).
pub fn build_agy_turn_args(
    request: &AgentTurnRequest,
    schema_path: &Path,
    prompt_payload: &str,
) -> Vec<String> {
    let mut args = vec![
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--sandbox".to_string(),
        "--disable-slash-commands".to_string(),
        "--json-schema".to_string(),
        schema_path.to_string_lossy().into_owned(),
    ];

    if request.timeout_secs > 0 {
        args.push("--print-timeout".to_string());
        args.push(format!("{}s", request.timeout_secs));
    }

    if let Some(ref model) = request.selected_model {
        let trimmed = model.trim();
        if !trimmed.is_empty() {
            args.push("--model".to_string());
            args.push(trimmed.to_string());
        }
    }

    if let Some(ref conv_id) = request.provider_conversation_id {
        let trimmed = conv_id.trim();
        if !trimmed.is_empty() {
            args.push("--conversation".to_string());
            args.push(trimmed.to_string());
        }
    }

    args.push("--print".to_string());
    args.push(prompt_payload.to_string());

    args
}

/// Ensambla el payload del turno acotando de forma segura su tamaño para CLI en Windows.
pub fn build_agy_prompt_payload(request: &AgentTurnRequest) -> String {
    let mut out = String::with_capacity(4096);
    out.push_str(&request.system_prompt);
    out.push_str("\n\n");

    if request.provider_conversation_id.is_none() && !request.turns.is_empty() {
        out.push_str("# HISTORIAL RECIENTE DE LA MISIÓN\n");
        for turn in &request.turns {
            let tag = match turn.role {
                ConversationRole::User => "OPERADOR",
                ConversationRole::Assistant => "ASISTENTE",
                ConversationRole::SystemEvent => "EVENTO_SISTEMA",
            };
            out.push_str(&format!("[{}]: {}\n", tag, turn.content));
        }
        out.push('\n');
    }

    let footer = format!(
        "# SOLICITUD ACTUAL DEL OPERADOR\n{}\n\nRESPONDE DIRECTAMENTE SIN LLAMAR HERRAMIENTAS LOCALES Y CUMPLE EL --json-schema CONFIGURADO.\n",
        request.user_prompt.trim()
    );

    if let Some(ref ctx) = request.terminal_context {
        let trimmed = ctx.trim();
        if !trimmed.is_empty() {
            let header_ctx = "# CONTEXTO SANITIZADO DE LA TERMINAL SSH\n```text\n";
            let close_ctx = "\n```\n\n";
            let fixed_len = out.chars().count() + footer.chars().count() + header_ctx.len() + close_ctx.len();
            let budget = MAX_CLI_PROMPT_CHARS.saturating_sub(fixed_len);
            if budget > 200 {
                out.push_str(header_ctx);
                let ctx_chars = trimmed.chars().count();
                if ctx_chars <= budget {
                    out.push_str(trimmed);
                } else {
                    // Preservar los últimos caracteres más recientes de consola
                    let tail: String = trimmed
                        .chars()
                        .skip(ctx_chars.saturating_sub(budget.saturating_sub(32)))
                        .collect();
                    out.push_str("[...truncado...]\n");
                    out.push_str(&tail);
                }
                out.push_str(close_ctx);
            }
        }
    }

    out.push_str(&footer);
    out
}

/// Parsea la salida de `agy models` ignorando encabezados y extrayendo pares `<id>\t<label>`.
pub fn parse_agy_models_output(stdout: &str) -> Vec<AiModelDescriptor> {
    let mut models = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("Fetching ") {
            continue;
        }
        if let Some((id, label)) = trimmed.split_once('\t') {
            let clean_id = id.trim();
            let clean_label = label.trim();
            if !clean_id.is_empty() && !clean_label.is_empty() {
                models.push(AiModelDescriptor {
                    id: clean_id.to_string(),
                    label: clean_label.to_string(),
                });
            }
        }
    }
    models
}

#[derive(Debug, Default)]
struct StreamParseState {
    conversation_id: Option<String>,
    raw_model_used: Option<String>,
    final_envelope: Option<AgentTurnWireEnvelope>,
    fallback_text: String,
    error_message: Option<String>,
}

/// Procesa una línea NDJSON de `agy --output-format stream-json`.
fn process_stream_json_line(
    line: &str,
    state: &mut StreamParseState,
    on_progress: &(dyn Fn(&str) + Send + Sync),
) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return;
    }

    let Ok(val) = serde_json::from_str::<Value>(trimmed) else {
        return;
    };

    // Capturar conversation_id en raíz, init, step_update o result
    if let Some(cid) = val
        .get("conversation_id")
        .or_else(|| val.get("conversationId"))
        .or_else(|| val.get("result").and_then(|r| r.get("conversation_id")))
        .or_else(|| val.get("step_update").and_then(|s| s.get("conversation_id")))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        state.conversation_id = Some(cid.to_string());
    }

    if let Some(model) = val
        .get("model")
        .or_else(|| val.get("init").and_then(|i| i.get("model")))
        .and_then(|v| v.as_str())
    {
        state.raw_model_used = Some(model.to_string());
    }

    if let Some(err) = val
        .get("error")
        .or_else(|| val.get("result").and_then(|r| r.get("error")))
        .and_then(|v| v.as_str().or_else(|| v.get("message").and_then(|m| m.as_str())))
    {
        state.error_message = Some(err.to_string());
    }

    // Progreso en vivo desde eventos `step_update.text_delta` o campos legacy
    if let Some(delta) = val
        .get("step_update")
        .and_then(|s| s.get("text_delta"))
        .and_then(|v| v.as_str())
    {
        if !delta.is_empty() {
            state.fallback_text.push_str(delta);
            on_progress(delta);
        }
    } else {
        for key in ["thought", "delta", "status", "text"] {
            if let Some(chunk) = val.get(key).and_then(|v| v.as_str()) {
                let clean = chunk.trim();
                if !clean.is_empty() && !clean.starts_with('{') {
                    if key == "delta" || key == "text" {
                        state.fallback_text.push_str(clean);
                    }
                    on_progress(clean);
                }
            }
        }
    }

    // 1. Formato oficial agy `{"event": "result", "result": { "structured_output": ... }}`
    if let Some(res_obj) = val.get("result") {
        if let Some(structured) = res_obj.get("structured_output") {
            if let Some(env) = try_parse_wire_envelope(structured) {
                state.final_envelope = Some(env);
                return;
            }
        }
        if let Some(resp_field) = res_obj.get("response") {
            if let Some(env) = try_parse_wire_envelope(resp_field) {
                state.final_envelope = Some(env);
                return;
            }
            if let Some(resp_str) = resp_field.as_str() {
                if !resp_str.trim().is_empty() {
                    state.fallback_text = resp_str.trim().to_string();
                }
            }
        }
    }

    // 2. Variantes directas o planas
    for candidate_key in ["structured_output", "result", "response", "output"] {
        if let Some(candidate) = val.get(candidate_key) {
            if let Some(env) = try_parse_wire_envelope(candidate) {
                state.final_envelope = Some(env);
                return;
            }
        }
    }

    if val.get("kind").is_some() || (val.get("phase").is_some() && val.get("message").is_some()) {
        if let Some(env) = try_parse_wire_envelope(&val) {
            state.final_envelope = Some(env);
        }
    }
}

fn try_parse_wire_envelope(val: &Value) -> Option<AgentTurnWireEnvelope> {
    if let Some(s) = val.as_str() {
        let trimmed = s.trim();
        let json_slice = strip_markdown_json_fence(trimmed);
        return serde_json::from_str::<AgentTurnWireEnvelope>(json_slice).ok();
    }
    serde_json::from_value::<AgentTurnWireEnvelope>(val.clone()).ok()
}

fn strip_markdown_json_fence(input: &str) -> &str {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix("```json") {
        if let Some(inner) = rest.strip_suffix("```") {
            return inner.trim();
        }
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        if let Some(inner) = rest.strip_suffix("```") {
            return inner.trim();
        }
    }
    trimmed
}

fn map_agy_failure_error(stderr: &str, stream_err: Option<&str>) -> AiDriverError {
    let combined = format!(
        "{}\n{}",
        stderr.trim(),
        stream_err.unwrap_or("").trim()
    );
    let lower = combined.to_ascii_lowercase();
    if lower.contains("not authenticated")
        || lower.contains("not logged in")
        || lower.contains("unauthorized")
        || lower.contains("please login")
        || lower.contains("authentication")
        || lower.contains("credentials")
    {
        return AiDriverError::AuthenticationRequired(
            "La sesión de 'agy' no está autenticada o expiró. Ejecuta 'agy' en una terminal para iniciar sesión.".to_string(),
        );
    }

    let detail = if combined.trim().is_empty() {
        "El proceso 'agy' terminó con error sin detalle adicional.".to_string()
    } else {
        combined.trim().to_string()
    };
    AiDriverError::ExecutionFailed(detail)
}

impl AiProviderDriver for AgyCliDriver {
    fn driver_id(&self) -> &str {
        "agy"
    }

    fn probe_health(&self) -> DriverHealthStatus {
        let Some(bin) = self.resolve_binary() else {
            return DriverHealthStatus {
                driver_id: "agy".to_string(),
                available: false,
                version: None,
                message: "No se encontró el binario 'agy' en PATH ni en la ruta configurada. Instala Google Antigravity CLI o especifica su ruta en Ajustes.".to_string(),
            };
        };

        let args = vec!["--version".to_string()];
        let mut cmd = match self.build_base_command(&bin, &args) {
            Ok(c) => c,
            Err(e) => {
                return DriverHealthStatus {
                    driver_id: "agy".to_string(),
                    available: false,
                    version: None,
                    message: e.to_string(),
                };
            }
        };

        match cmd.output() {
            Ok(out) if out.status.success() => {
                let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
                DriverHealthStatus {
                    driver_id: "agy".to_string(),
                    available: true,
                    version: if ver.is_empty() { None } else { Some(ver.clone()) },
                    message: format!("agy CLI disponible ({})", if ver.is_empty() { "versión detectada" } else { &ver }),
                }
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
                DriverHealthStatus {
                    driver_id: "agy".to_string(),
                    available: false,
                    version: None,
                    message: format!("'agy --version' falló: {}", err),
                }
            }
            Err(e) => DriverHealthStatus {
                driver_id: "agy".to_string(),
                available: false,
                version: None,
                message: format!("No se pudo ejecutar '{}': {}", bin.display(), e),
            },
        }
    }

    fn list_models(&self) -> Result<Vec<AiModelDescriptor>, AiDriverError> {
        let bin = self.resolve_binary().ok_or_else(|| {
            AiDriverError::BinaryNotFound(
                "No se encontró el binario 'agy' para listar modelos disponibles.".to_string(),
            )
        })?;

        let args = vec!["models".to_string()];
        let mut cmd = self.build_base_command(&bin, &args)?;
        let out = cmd.output().map_err(|e| {
            AiDriverError::ExecutionFailed(format!("Error al ejecutar 'agy models': {}", e))
        })?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(map_agy_failure_error(&stderr, None));
        }

        let stdout = String::from_utf8_lossy(&out.stdout);
        Ok(parse_agy_models_output(&stdout))
    }

    fn execute_turn(
        &self,
        request: &AgentTurnRequest,
        cancel: &TurnCancellationToken,
        on_progress: &(dyn Fn(&str) + Send + Sync),
    ) -> Result<AgentTurnOutcome, AiDriverError> {
        if cancel.is_cancelled() {
            return Err(AiDriverError::Cancelled);
        }

        let bin = self.resolve_binary().ok_or_else(|| {
            AiDriverError::BinaryNotFound(
                "No se encontró el binario 'agy'. Configura su ruta en Ajustes o agrégalo al PATH.".to_string(),
            )
        })?;

        let schema_file = TempSchemaFile::create(&self.sandbox_dir, &request.json_schema)?;
        let prompt_payload = build_agy_prompt_payload(request);
        let args = build_agy_turn_args(request, schema_file.path(), &prompt_payload);

        let mut cmd = self.build_base_command(&bin, &args)?;
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            AiDriverError::ExecutionFailed(format!("Error al iniciar proceso 'agy': {}", e))
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            AiDriverError::ExecutionFailed("No se pudo capturar stdout de 'agy'".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            AiDriverError::ExecutionFailed("No se pudo capturar stderr de 'agy'".to_string())
        })?;

        let (tx_line, rx_line) = mpsc::channel::<String>();
        let stdout_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if tx_line.send(line).is_err() {
                    break;
                }
            }
        });

        let stderr_handle = std::thread::spawn(move || {
            let mut buf = String::new();
            let mut reader = BufReader::new(stderr);
            let _ = std::io::Read::read_to_string(&mut reader, &mut buf);
            buf
        });

        let timeout = Duration::from_secs(request.timeout_secs.max(10));
        let started = Instant::now();
        let mut parse_state = StreamParseState::default();

        loop {
            if cancel.is_cancelled() {
                terminate_process_tree(&mut child);
                return Err(AiDriverError::Cancelled);
            }

            if started.elapsed() >= timeout {
                terminate_process_tree(&mut child);
                return Err(AiDriverError::Timeout(format!(
                    "El proveedor 'agy' excedió el tiempo límite de {}s.",
                    timeout.as_secs()
                )));
            }

            match rx_line.recv_timeout(Duration::from_millis(50)) {
                Ok(line) => {
                    process_stream_json_line(&line, &mut parse_state, on_progress);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        let status = child.wait().map_err(|e| {
            AiDriverError::ExecutionFailed(format!("Error esperando término de 'agy': {}", e))
        })?;
        let _ = stdout_handle.join();
        let stderr_text = stderr_handle.join().unwrap_or_default();

        if let Some(env) = parse_state.final_envelope {
            let response = AgentResponseKind::try_from(env).map_err(AiDriverError::InvalidResponse)?;
            return Ok(AgentTurnOutcome {
                response,
                provider_conversation_id: parse_state.conversation_id,
                raw_model_used: parse_state.raw_model_used,
            });
        }

        // Degradación elegante: si agy terminó ok con texto útil fuera de JSON, entregar como ReplyOnly
        if status.success() && !parse_state.fallback_text.trim().is_empty() {
            return Ok(AgentTurnOutcome {
                response: AgentResponseKind::ReplyOnly {
                    message: parse_state.fallback_text.trim().to_string(),
                    phase_tag: Some(crate::ai::schema::ConversationalPhaseTag::DirectAnswer),
                },
                provider_conversation_id: parse_state.conversation_id,
                raw_model_used: parse_state.raw_model_used,
            });
        }

        if !status.success() || parse_state.error_message.is_some() {
            return Err(map_agy_failure_error(
                &stderr_text,
                parse_state.error_message.as_deref(),
            ));
        }

        Err(AiDriverError::InvalidResponse(
            "El flujo stream-json de 'agy' finalizó sin entregar un sobre JSON válido del esquema."
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::schema::{agent_turn_wire_json_schema, ConversationTurn, RiskLevel};
    use std::sync::Mutex as StdMutex;

    #[test]
    fn build_args_incluye_blindaje_sandbox_y_excluye_skip_permissions() {
        let req = AgentTurnRequest {
            system_prompt: "sys".to_string(),
            user_prompt: "Revisa puertos".to_string(),
            terminal_context: None,
            turns: Vec::new(),
            provider_conversation_id: Some("conv-xyz".to_string()),
            selected_model: Some("gemini-4-carbon-high".to_string()),
            json_schema: agent_turn_wire_json_schema(),
            timeout_secs: 45,
        };
        let args = build_agy_turn_args(&req, Path::new("/tmp/sandbox/schema.json"), "mi prompt");

        assert!(args.contains(&"stream-json".to_string()));
        assert!(args.contains(&"--sandbox".to_string()));
        assert!(args.contains(&"--disable-slash-commands".to_string()));
        assert!(args.contains(&"--json-schema".to_string()));
        assert!(args.contains(&"--model".to_string()));
        assert!(args.contains(&"gemini-4-carbon-high".to_string()));
        assert!(args.contains(&"--conversation".to_string()));
        assert!(args.contains(&"conv-xyz".to_string()));
        assert!(args.contains(&"--print".to_string()));
        assert!(args.contains(&"mi prompt".to_string()));
        assert!(!args.iter().any(|a| a.contains("dangerously-skip-permissions")));
    }

    #[test]
    fn parsea_catalogo_real_de_agy_models() {
        let sample = "Fetching available models...\n\
            gemini-4-neon-high\tGemini 4 Neon (High)\n\
            claude-sonnet-5-5-high\tClaude Sonnet 5.5 (High)\n\
            linea sin tabulador ignorada\n";
        let models = parse_agy_models_output(sample);
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "gemini-4-neon-high");
        assert_eq!(models[0].label, "Gemini 4 Neon (High)");
        assert_eq!(models[1].id, "claude-sonnet-5-5-high");
    }

    #[test]
    fn stream_json_parser_soporta_formato_real_de_agy_1_5() {
        let mut state = StreamParseState::default();
        let progress = StdMutex::new(Vec::new());

        let line_init = r#"{"event":"init","conversation_id":"conv-real-1","init":{"model":"gemini-3.8-flash-low"}}"#;
        let line_delta = r#"{"event":"step_update","step_update":{"conversation_id":"conv-real-1","step_index":1,"state":"ACTIVE","step_type":"agent_response","text_delta":"Analizando..."}}"#;
        let line_result = r#"{"event":"result","result":{"conversation_id":"conv-real-1","status":"SUCCESS","structured_output":{"kind":"propose_step","step":{"reasoning":"Revisar memoria","command":"free -h","risk_level":"read_only"}}}}"#;

        for l in [line_init, line_delta, line_result] {
            process_stream_json_line(l, &mut state, &|p| progress.lock().unwrap().push(p.to_string()));
        }

        assert_eq!(progress.lock().unwrap().as_slice(), &["Analizando..."]);
        assert_eq!(state.conversation_id.as_deref(), Some("conv-real-1"));
        assert_eq!(state.raw_model_used.as_deref(), Some("gemini-3.8-flash-low"));

        let env = state.final_envelope.expect("envelope");
        let parsed = AgentResponseKind::try_from(env).expect("valid");
        match parsed {
            AgentResponseKind::ProposeStep { command, risk_level, .. } => {
                assert_eq!(command, "free -h");
                assert_eq!(risk_level, RiskLevel::ReadOnly);
            }
            other => panic!("Esperaba ProposeStep, llegó {:?}", other),
        }
    }

    #[test]
    fn mapea_error_de_autenticacion_a_spanish_accionable() {
        let err = map_agy_failure_error("Error: user not authenticated, please login", None);
        assert!(matches!(err, AiDriverError::AuthenticationRequired(_)));
        assert!(err.to_string().contains("autenticada"));
    }

    #[test]
    fn prompt_payload_incluye_turns_cuando_no_hay_conversation_id_y_acota_contexto() {
        let req = AgentTurnRequest {
            system_prompt: "CONSTITUCION".to_string(),
            user_prompt: "Cambia el puerto a 8085".to_string(),
            terminal_context: Some("Debian 12".to_string()),
            turns: vec![ConversationTurn {
                role: ConversationRole::User,
                content: "Instalar Tomcat".to_string(),
            }],
            provider_conversation_id: None,
            selected_model: None,
            json_schema: serde_json::json!({}),
            timeout_secs: 30,
        };
        let payload = build_agy_prompt_payload(&req);
        assert!(payload.contains("HISTORIAL RECIENTE"));
        assert!(payload.contains("Instalar Tomcat"));
        assert!(payload.contains("Debian 12"));
        assert!(payload.contains("Cambia el puerto a 8085"));
    }

    #[test]
    fn binario_inexistente_retorna_unavailable_en_probe_health() {
        let temp_sandbox = std::env::temp_dir().join(format!("nekossh-sb-test-{}", Uuid::new_v4()));
        let driver = AgyCliDriver::new(
            Some(r"C:\ruta\que\no\existe\agy-fake.exe".to_string()),
            temp_sandbox,
        );
        let status = driver.probe_health();
        assert!(!status.available);
        assert!(status.message.contains("No se encontró"));
    }

    #[test]
    #[ignore = "requiere agy CLI instalado y autenticado en entorno local"]
    fn smoke_real_agy_probe_models_y_stream_json() {
        let temp_sandbox = std::env::temp_dir().join(format!("nekossh-agy-smoke-{}", Uuid::new_v4()));
        let driver = AgyCliDriver::new(None, temp_sandbox.clone());

        let health = driver.probe_health();
        println!("HEALTH: {:?}", health);
        assert!(health.available, "agy debe estar disponible: {}", health.message);

        let models = driver.list_models().expect("list_models");
        println!("MODELS COUNT: {}", models.len());
        assert!(!models.is_empty());

        let req = AgentTurnRequest {
            system_prompt: crate::ai::prompt::PromptAssembler::assemble_system_prompt("", ""),
            user_prompt: "puedes revisar que OS tenemos?".to_string(),
            terminal_context: None,
            turns: Vec::new(),
            provider_conversation_id: None,
            selected_model: Some("gemini-3.8-flash-low".to_string()),
            json_schema: agent_turn_wire_json_schema(),
            timeout_secs: 45,
        };
        let cancel = TurnCancellationToken::new();
        let outcome = driver
            .execute_turn(&req, &cancel, &|chunk| {
                print!("{}", chunk);
            })
            .expect("execute_turn turno 1 contra agy real");
        println!("\nTURN 1 OUTCOME: {:?}", outcome);

        // Validar que cualquier comando propuesto pase por evaluate_proposal sin bloquearse
        if let AgentResponseKind::ProposeStep { command, risk_level, .. } = &outcome.response {
            let report = crate::ai::risk_guard::CommandRiskGuard::evaluate_proposal(*risk_level, command);
            println!("SAFETY REPORT TURN 1: {:?}", report);
        }

        // Turno 2 continuando misma conversación simulando salida real del servidor
        let req2 = AgentTurnRequest {
            system_prompt: crate::ai::prompt::PromptAssembler::assemble_system_prompt("", ""),
            user_prompt: "Resultado de inspección (exit=0): PRETTY_NAME=\"Debian GNU/Linux 12 (bookworm)\"\nLinux prod-web 6.1.0-26-amd64".to_string(),
            terminal_context: None,
            turns: Vec::new(),
            provider_conversation_id: outcome.provider_conversation_id.clone(),
            selected_model: Some("gemini-3.8-flash-low".to_string()),
            json_schema: agent_turn_wire_json_schema(),
            timeout_secs: 45,
        };
        let outcome2 = driver
            .execute_turn(&req2, &cancel, &|_| {})
            .expect("execute_turn turno 2 respuesta directa");
        println!("\nTURN 2 OUTCOME: {:?}", outcome2);

        let _ = std::fs::remove_dir_all(&temp_sandbox);
    }

    #[test]
    fn cancelacion_mata_arbol_de_procesos_y_retorna_rapido_sin_colgar_pipes() {
        let temp_dir = std::env::temp_dir().join(format!("nekossh-agy-killtree-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        #[cfg(windows)]
        let shim_path = {
            let p = temp_dir.join("agy-hang.cmd");
            std::fs::write(&p, "@echo off\r\nping -n 20 127.0.0.1 >nul\r\n").unwrap();
            p
        };
        #[cfg(not(windows))]
        let shim_path = {
            use std::os::unix::fs::PermissionsExt;
            let p = temp_dir.join("agy-hang.sh");
            std::fs::write(&p, "#!/bin/sh\nsleep 20\n").unwrap();
            let mut perms = std::fs::metadata(&p).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&p, perms).unwrap();
            p
        };

        let driver = AgyCliDriver::new(
            Some(shim_path.to_string_lossy().into_owned()),
            temp_dir.clone(),
        );
        let req = AgentTurnRequest {
            system_prompt: "sys".to_string(),
            user_prompt: "hola".to_string(),
            terminal_context: None,
            turns: Vec::new(),
            provider_conversation_id: None,
            selected_model: None,
            json_schema: agent_turn_wire_json_schema(),
            timeout_secs: 30,
        };

        let cancel = TurnCancellationToken::new();
        let cancel_trigger = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            cancel_trigger.cancel();
        });

        let started = Instant::now();
        let res = driver.execute_turn(&req, &cancel, &|_| {});
        let elapsed = started.elapsed();

        let _ = std::fs::remove_dir_all(&temp_dir);
        assert_eq!(res.unwrap_err(), AiDriverError::Cancelled);
        assert!(
            elapsed < Duration::from_secs(3),
            "cancelar árbol shim no debe quedar bloqueado en pipes; tardó {:?}",
            elapsed
        );
    }
}
