//! Colector opt-in de contexto PTY (`PtyContextTap`) y corredor de centinela POSIX silencioso.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::fsm::{AgentHarnessFsm, HarnessPhase};
use super::preferences::{clamp_context_max_lines, DEFAULT_CONTEXT_MAX_LINES};

const MAX_BUFFER_LINES: usize = 2000;
const SENTINEL_PREFIX: &str = "__NEKO_END_";
const MAX_PENDING_TAIL_BYTES: usize = 128;
const MAX_ECHO_TAIL_BYTES: usize = 220;
const ECHO_ANCHOR: &str = ";__neko_rc=$?;printf";
const ECHO_END_TOKEN: &str = "\"$__neko_rc\"";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SentinelPurpose {
    ReadOnlyRecon {
        command: String,
    },
    AuthorizedCommand {
        command: String,
        #[serde(default)]
        purpose: String,
    },
    PlanStep {
        step_index: usize,
        command: String,
    },
    StepRollback {
        step_index: usize,
        rollback_cmd: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SentinelCompletion {
    pub terminal_id: String,
    pub nonce: String,
    pub exit_code: i32,
    pub captured_output: String,
    pub purpose: SentinelPurpose,
    #[serde(default)]
    pub updated_phase: Option<HarnessPhase>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessChunkResult {
    pub visible_output: String,
    pub sentinel_completed: Option<SentinelCompletion>,
    pub waiting_input_hint: Option<String>,
    pub cwd_changed: Option<String>,
}

#[derive(Debug)]
struct ActiveSentinel {
    nonce: String,
    purpose: SentinelPurpose,
    raw_capture: String,
    pending_tail: String,
    echo_stripped: bool,
    waiting_hint_emitted: bool,
}

#[derive(Debug, Default)]
pub struct PtySessionTap {
    listening: bool,
    in_alternate_screen: bool,
    lines: VecDeque<String>,
    current_line: String,
    active_sentinel: Option<ActiveSentinel>,
    current_cwd: Option<String>,
    osc7_leftover: String,
}

impl PtySessionTap {
    fn clear_buffers(&mut self) {
        self.lines.clear();
        self.current_line.clear();
        self.in_alternate_screen = false;
    }

    fn update_osc7_cwd(&mut self, raw: &str) -> Option<String> {
        const MAX_OSC7_LEFTOVER_BYTES: usize = 512;
        let combined = if self.osc7_leftover.is_empty() {
            raw.to_string()
        } else {
            format!("{}{}", self.osc7_leftover, raw)
        };
        let (paths, leftover) = crate::osc7::extract_cwd_paths(&combined);
        self.osc7_leftover = if leftover.len() <= MAX_OSC7_LEFTOVER_BYTES {
            leftover
        } else {
            String::new()
        };
        if let Some(latest) = paths.into_iter().last() {
            if self.current_cwd.as_deref() != Some(latest.as_str()) {
                self.current_cwd = Some(latest.clone());
                return Some(latest);
            }
        }
        None
    }

    fn captured_line_count(&self) -> usize {
        self.lines.len()
            + usize::from(!self.current_line.trim().is_empty())
    }

    fn ingest_clean_chunk(&mut self, raw: &str) {
        let mut chars = raw.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                match chars.peek().copied() {
                    Some('[') => {
                        chars.next();
                        let mut csi = String::new();
                        while let Some(c) = chars.next() {
                            csi.push(c);
                            if ('@'..='~').contains(&c) {
                                break;
                            }
                        }
                        if csi == "?1049h" || csi == "?47h" || csi == "?1047h" {
                            self.in_alternate_screen = true;
                        } else if csi == "?1049l" || csi == "?47l" || csi == "?1047l" {
                            self.in_alternate_screen = false;
                        }
                    }
                    Some(']') => {
                        chars.next();
                        // OSC termina en BEL (\x07) o ST (\x1b\\)
                        while let Some(c) = chars.next() {
                            if c == '\x07' {
                                break;
                            }
                            if c == '\x1b' && chars.peek() == Some(&'\\') {
                                chars.next();
                                break;
                            }
                        }
                    }
                    Some(_) => {
                        chars.next();
                    }
                    None => {}
                }
                continue;
            }

            if self.in_alternate_screen {
                continue;
            }

            match ch {
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                        self.push_completed_line();
                    } else {
                        // Sobrescritura en la misma línea (barras de progreso)
                        self.current_line.clear();
                    }
                }
                '\n' => {
                    self.push_completed_line();
                }
                '\x08' => {
                    self.current_line.pop();
                }
                c if c.is_control() && c != '\t' => {}
                c => {
                    self.current_line.push(c);
                }
            }
        }
    }

    fn push_completed_line(&mut self) {
        let completed = std::mem::take(&mut self.current_line);
        self.lines.push_back(completed);
        while self.lines.len() > MAX_BUFFER_LINES {
            self.lines.pop_front();
        }
    }

    fn snapshot(&self, max_lines: usize) -> String {
        let mut all: Vec<String> = self.lines.iter().cloned().collect();
        if !self.current_line.trim().is_empty() {
            all.push(self.current_line.clone());
        }
        format_head_tail_lines(&all, max_lines)
    }
}

/// Limpia secuencias ANSI/OSC/CR de un texto completo en una sola pasada determinista.
pub fn clean_ansi_output(raw: &str) -> String {
    let mut tap = PtySessionTap {
        listening: true,
        ..Default::default()
    };
    tap.ingest_clean_chunk(raw);
    let mut lines: Vec<String> = tap.lines.into_iter().collect();
    if !tap.current_line.is_empty() {
        lines.push(tap.current_line);
    }
    lines.join("\n")
}

/// Recorta deterministamente mediante estrategia Head + Tail respetando el presupuesto.
pub fn format_head_tail_lines(lines: &[String], max_lines: usize) -> String {
    if lines.is_empty() || max_lines == 0 {
        return String::new();
    }
    if lines.len() <= max_lines {
        return lines.join("\n");
    }

    let head_count = (max_lines / 3).max(1);
    let tail_count = max_lines.saturating_sub(head_count).max(1);
    let omitted = lines.len().saturating_sub(head_count + tail_count);

    let mut out = Vec::with_capacity(head_count + tail_count + 1);
    out.extend_from_slice(&lines[..head_count]);
    out.push(format!("[... {} líneas omitidas ...]", omitted));
    out.extend_from_slice(&lines[lines.len() - tail_count..]);
    out.join("\n")
}

/// Determina si el comando contiene saltos de línea o comentarios `#` sin comillas
/// (único caso que requiere bloque agrupado `{ ... \n}` antes de `;`).
fn needs_block_grouping(cmd: &str) -> bool {
    if cmd.contains('\n') {
        return true;
    }
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;
    let mut prev = ' ';
    for ch in cmd.chars() {
        if escaped {
            escaped = false;
            prev = ch;
            continue;
        }
        if ch == '\\' && !in_single {
            escaped = true;
            prev = ch;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            prev = ch;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            prev = ch;
            continue;
        }
        if !in_single && !in_double && ch == '#' && prev.is_whitespace() {
            return true;
        }
        prev = ch;
    }
    false
}

/// Construye el comando POSIX con marcador centinela único:
/// - En comandos de una línea evita `{ ... \n}` para impedir el prompt `PS2` (`> `) en consola.
/// - Mantiene `%s_%s` separado para inmunidad ante eco TTY.
pub fn wrap_command_with_sentinel(command: &str, nonce: &str) -> String {
    let clean_cmd = command.trim().trim_end_matches(';').trim();
    if needs_block_grouping(clean_cmd) {
        format!(
            " {{\n{}\n}};__neko_rc=$?;printf '\\n%s_%s:%d__\\n' '__NEKO_END' '{}' \"$__neko_rc\"\n",
            clean_cmd, nonce
        )
    } else {
        format!(
            " {};__neko_rc=$?;printf '\\n%s_%s:%d__\\n' '__NEKO_END' '{}' \"$__neko_rc\"\n",
            clean_cmd, nonce
        )
    }
}

/// Aplica de forma determinista el efecto de un centinela completado sobre la FSM de esa sesión.
pub fn apply_sentinel_completion_to_fsm(fsm: &mut AgentHarnessFsm, done: &SentinelCompletion) {
    match &done.purpose {
        SentinelPurpose::ReadOnlyRecon { .. } | SentinelPurpose::AuthorizedCommand { .. } => {}
        SentinelPurpose::PlanStep { command, .. } => {
            if done.exit_code == 0 {
                let _ = fsm.complete_current_step();
            } else {
                let _ = fsm.halt_on_error(
                    command.clone(),
                    done.exit_code,
                    done.captured_output.clone(),
                );
            }
        }
        SentinelPurpose::StepRollback { step_index, .. } => {
            if done.exit_code == 0 {
                let _ = fsm.mark_step_rolled_back(*step_index);
            }
        }
    }
}

enum MarkerScan {
    Complete {
        start_idx: usize,
        marker_end: usize,
        exit_code: i32,
    },
    IncompleteFrom(usize),
    NotFound,
}

fn scan_sentinel_marker(combined: &str, marker_head: &str) -> MarkerScan {
    let mut offset = 0;
    while offset < combined.len() {
        let Some(rel_start) = combined[offset..].find(marker_head) else {
            break;
        };
        let start_idx = offset + rel_start;
        let code_start = start_idx + marker_head.len();
        let Some(end_rel) = combined[code_start..].find("__") else {
            return MarkerScan::IncompleteFrom(start_idx);
        };
        let code_str = &combined[code_start..code_start + end_rel];
        if let Ok(exit_code) = code_str.trim().parse::<i32>() {
            return MarkerScan::Complete {
                start_idx,
                marker_end: code_start + end_rel + 2,
                exit_code,
            };
        }
        // Coincidencia falsa (ej. literal `%d` en logs/eco antiguo): continuar buscando en el mismo chunk
        offset = code_start;
    }
    MarkerScan::NotFound
}

/// Registro global thread-safe de colectores PTY por `terminal_id`.
#[derive(Clone)]
pub struct PtyTapRegistry {
    sessions: Arc<Mutex<HashMap<String, PtySessionTap>>>,
    max_capture_lines: Arc<AtomicUsize>,
}

impl Default for PtyTapRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PtyTapRegistry {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            max_capture_lines: Arc::new(AtomicUsize::new(DEFAULT_CONTEXT_MAX_LINES)),
        }
    }

    pub fn set_max_capture_lines(&self, lines: usize) {
        self.max_capture_lines
            .store(clamp_context_max_lines(lines), Ordering::SeqCst);
    }

    pub fn max_capture_lines(&self) -> usize {
        self.max_capture_lines.load(Ordering::SeqCst)
    }

    pub fn set_listening(&self, terminal_id: &str, enabled: bool) {
        let mut map = self.sessions.lock().unwrap();
        if !enabled {
            if let Some(tap) = map.get_mut(terminal_id) {
                tap.listening = false;
                tap.clear_buffers();
            }
            return;
        }
        let tap = map.entry(terminal_id.to_string()).or_default();
        tap.listening = true;
    }

    pub fn is_listening(&self, terminal_id: &str) -> bool {
        let map = self.sessions.lock().unwrap();
        map.get(terminal_id).map(|t| t.listening).unwrap_or(false)
    }

    /// Apaga la escucha PTY en todas las sesiones activas y purga sus buffers acumulados.
    /// Retorna `true` si al menos una sesión tenía escucha activa previa.
    pub fn disable_all_listening(&self) -> bool {
        let mut map = self.sessions.lock().unwrap();
        let mut any_disabled = false;
        for tap in map.values_mut() {
            if tap.listening || !tap.lines.is_empty() || !tap.current_line.is_empty() {
                if tap.listening {
                    any_disabled = true;
                }
                tap.listening = false;
                tap.clear_buffers();
            }
        }
        any_disabled
    }

    pub fn has_active_sentinel(&self, terminal_id: &str) -> bool {
        let map = self.sessions.lock().unwrap();
        map.get(terminal_id)
            .map(|t| t.active_sentinel.is_some())
            .unwrap_or(false)
    }

    /// Intenta armar un centinela tipado, rechazando colisiones si ya existe otro en vuelo.
    pub fn try_arm_sentinel(
        &self,
        terminal_id: &str,
        nonce: impl Into<String>,
        purpose: SentinelPurpose,
    ) -> Result<(), String> {
        let mut map = self.sessions.lock().unwrap();
        let tap = map.entry(terminal_id.to_string()).or_default();
        if tap.active_sentinel.is_some() {
            return Err(
                "Ya existe un comando supervisado en ejecución esperando confirmación en esta terminal."
                    .to_string(),
            );
        }
        tap.active_sentinel = Some(ActiveSentinel {
            nonce: nonce.into(),
            purpose,
            raw_capture: String::new(),
            pending_tail: String::new(),
            echo_stripped: false,
            waiting_hint_emitted: false,
        });
        Ok(())
    }

    /// Atajo de compatibilidad para pruebas de lectura simple.
    pub fn arm_sentinel(&self, terminal_id: &str, nonce: impl Into<String>) {
        let mut map = self.sessions.lock().unwrap();
        let tap = map.entry(terminal_id.to_string()).or_default();
        tap.active_sentinel = Some(ActiveSentinel {
            nonce: nonce.into(),
            purpose: SentinelPurpose::ReadOnlyRecon {
                command: String::new(),
            },
            raw_capture: String::new(),
            pending_tail: String::new(),
            echo_stripped: false,
            waiting_hint_emitted: false,
        });
    }

    /// Extrae y limpia el centinela en vuelo (para `Ctrl+C` o caída abrupta de socket SSH).
    pub fn take_sentinel(&self, terminal_id: &str) -> Option<(String, SentinelPurpose)> {
        let mut map = self.sessions.lock().unwrap();
        map.get_mut(terminal_id)
            .and_then(|tap| tap.active_sentinel.take())
            .map(|s| (s.nonce, s.purpose))
    }

    pub fn clear_sentinel(&self, terminal_id: &str) {
        let _ = self.take_sentinel(terminal_id);
    }

    pub fn remove_session(&self, terminal_id: &str) {
        let mut map = self.sessions.lock().unwrap();
        map.remove(terminal_id);
    }

    pub fn get_context_snapshot(&self, terminal_id: &str, max_lines: usize) -> String {
        let map = self.sessions.lock().unwrap();
        match map.get(terminal_id) {
            Some(tap) if tap.listening => tap.snapshot(max_lines),
            _ => String::new(),
        }
    }

    pub fn current_cwd(&self, terminal_id: &str) -> Option<String> {
        let map = self.sessions.lock().unwrap();
        map.get(terminal_id).and_then(|t| t.current_cwd.clone())
    }

    pub fn captured_line_count(&self, terminal_id: &str) -> usize {
        let map = self.sessions.lock().unwrap();
        match map.get(terminal_id) {
            Some(tap) if tap.listening => tap.captured_line_count(),
            _ => 0,
        }
    }

    /// Siembra el contenido visible actual del buffer xterm.js al activar `Escucha ON`.
    /// Solo opera si `listening == true` y no hay líneas previas capturadas; al apagar la escucha todo se purga.
    pub fn seed_visible_screen(&self, terminal_id: &str, visible_text: &str) {
        let mut map = self.sessions.lock().unwrap();
        let Some(tap) = map.get_mut(terminal_id) else {
            return;
        };
        if !tap.listening {
            return;
        }
        if !tap.lines.is_empty() || !tap.current_line.is_empty() {
            return;
        }
        for line in visible_text.lines() {
            let trimmed_end = line.trim_end_matches('\r');
            tap.lines.push_back(trimmed_end.to_string());
            while tap.lines.len() > MAX_BUFFER_LINES {
                tap.lines.pop_front();
            }
        }
    }

    /// Intercepta un chunk proveniente de `ssh-stdout`:
    /// - Si `listening == false` y no hay centinela activo: no retiene bytes y pasa intacto.
    /// - Si hay centinela activo: detecta y oculta `__NEKO_END_<nonce>:<code>__` incluso si viene partido o precedido por eco TTY.
    pub fn process_chunk(&self, terminal_id: &str, raw_chunk: &str) -> ProcessChunkResult {
        let max_lines = self.max_capture_lines();
        let mut map = self.sessions.lock().unwrap();
        let tap = map.entry(terminal_id.to_string()).or_default();
        let cwd_changed = tap.update_osc7_cwd(raw_chunk);

        if !tap.listening && tap.active_sentinel.is_none() {
            return ProcessChunkResult {
                visible_output: raw_chunk.to_string(),
                sentinel_completed: None,
                waiting_input_hint: None,
                cwd_changed,
            };
        }

        let mut visible_out = raw_chunk.to_string();
        let mut completed: Option<SentinelCompletion> = None;
        let mut waiting_hint: Option<String> = None;

        if let Some(mut sentinel) = tap.active_sentinel.take() {
            let mut combined = String::with_capacity(sentinel.pending_tail.len() + raw_chunk.len());
            combined.push_str(&sentinel.pending_tail);
            combined.push_str(raw_chunk);
            sentinel.pending_tail.clear();

            // 1. Ocultar el eco del wrapper `;__neko_rc=$?;printf ... "$__neko_rc"` antes de emitir a xterm.js
            let mut deferred_echo_tail = String::new();
            if !sentinel.echo_stripped {
                match strip_echo_plumbing(&combined, &sentinel.nonce) {
                    EchoStripOutcome::Stripped(cleaned) => {
                        combined = cleaned;
                        sentinel.echo_stripped = true;
                    }
                    EchoStripOutcome::HoldTail { emit_now, hold_tail } => {
                        combined = emit_now;
                        deferred_echo_tail = hold_tail;
                    }
                    EchoStripOutcome::NotPresent => {}
                }
            }

            let marker_head = format!("{}{}:", SENTINEL_PREFIX, sentinel.nonce);
            match scan_sentinel_marker(&combined, &marker_head) {
                MarkerScan::Complete {
                    start_idx,
                    marker_end,
                    exit_code,
                } => {
                    let after_start = if combined[marker_end..].starts_with("\r\n") {
                        marker_end + 2
                    } else if combined[marker_end..].starts_with('\n')
                        || combined[marker_end..].starts_with('\r')
                    {
                        marker_end + 1
                    } else {
                        marker_end
                    };

                    let before_end = if combined[..start_idx].ends_with("\r\n\r\n") {
                        start_idx - 2
                    } else if combined[..start_idx].ends_with("\n\n") {
                        start_idx - 1
                    } else {
                        start_idx
                    };

                    let before = &combined[..before_end];
                    let after = &combined[after_start..];

                    sentinel.raw_capture.push_str(before);
                    let cleaned_capture =
                        clean_captured_command_output(&sentinel.raw_capture, &sentinel.nonce, max_lines);

                    visible_out = format!("{}{}", before, after);
                    completed = Some(SentinelCompletion {
                        terminal_id: terminal_id.to_string(),
                        nonce: sentinel.nonce.clone(),
                        exit_code,
                        captured_output: cleaned_capture,
                        purpose: sentinel.purpose,
                        updated_phase: None,
                    });
                }
                MarkerScan::IncompleteFrom(start_idx) => {
                    let hold_from =
                        start_idx.max(combined.len().saturating_sub(MAX_PENDING_TAIL_BYTES));
                    let safe_boundary = floor_char_boundary(&combined, hold_from);
                    visible_out = combined[..safe_boundary].to_string();
                    sentinel.raw_capture.push_str(&visible_out);
                    sentinel.pending_tail = format!("{}{}", &combined[safe_boundary..], deferred_echo_tail);
                    tap.active_sentinel = Some(sentinel);
                }
                MarkerScan::NotFound => {
                    if let Some(prefix_idx) = find_partial_marker_tail(&combined, &marker_head) {
                        visible_out = combined[..prefix_idx].to_string();
                        sentinel.raw_capture.push_str(&visible_out);
                        sentinel.pending_tail = format!("{}{}", &combined[prefix_idx..], deferred_echo_tail);
                        tap.active_sentinel = Some(sentinel);
                    } else {
                        sentinel.raw_capture.push_str(&combined);
                        if sentinel.raw_capture.len() > 512 * 1024 {
                            let keep_from = floor_char_boundary(
                                &sentinel.raw_capture,
                                sentinel.raw_capture.len() - 256 * 1024,
                            );
                            sentinel.raw_capture = sentinel.raw_capture[keep_from..].to_string();
                        }
                        visible_out = combined;
                        sentinel.pending_tail = deferred_echo_tail;
                        tap.active_sentinel = Some(sentinel);
                    }
                }
            }
        }

        if completed.is_none() {
            if let Some(sentinel) = tap.active_sentinel.as_mut() {
                if !sentinel.waiting_hint_emitted {
                    if let Some(hint) = detect_waiting_input_hint(&visible_out) {
                        sentinel.waiting_hint_emitted = true;
                        waiting_hint = Some(hint.to_string());
                    }
                }
            }
        }

        if tap.listening && !visible_out.is_empty() {
            tap.ingest_clean_chunk(&visible_out);
        }

        ProcessChunkResult {
            visible_output: visible_out,
            sentinel_completed: completed,
            waiting_input_hint: waiting_hint,
            cwd_changed,
        }
    }
}

/// Detecta prompts bloqueantes (`sudo password`, `[y/N]`) o errores de shell no-POSIX (`fish`)
/// sobre texto visible mientras existe un centinela activo.
fn detect_waiting_input_hint(visible_chunk: &str) -> Option<&'static str> {
    if visible_chunk.trim().is_empty() {
        return None;
    }
    let clean = clean_ansi_output(visible_chunk).to_ascii_lowercase();
    if clean.contains("fish: unsupported use of '='")
        || clean.contains("fish: unknown command '__neko_rc'")
        || clean.contains("in fish, please use 'set")
    {
        return Some(
            "Tu shell remoto parece ser 'fish' (no POSIX). El marcador automático puede quedar esperando; cambia a bash/sh o pulsa Ctrl+C.",
        );
    }
    if clean.contains("[sudo] password")
        || clean.contains("password for ")
        || clean.contains("enter passphrase")
        || clean.contains("passphrase for ")
        || clean.contains("[y/n]")
        || clean.contains("(y/n)")
        || clean.contains("are you sure you want to continue")
    {
        return Some(
            "La terminal espera entrada interactiva (contraseña sudo o confirmación [y/N]). Responde en la terminal o pulsa Ctrl+C.",
        );
    }
    None
}

enum EchoStripOutcome {
    Stripped(String),
    HoldTail { emit_now: String, hold_tail: String },
    NotPresent,
}

/// Detecta y recorta el sufijo de eco `;__neko_rc=$?;printf ... "$__neko_rc"` incluso si viene
/// envuelto por ancho de columnas (`\r\n`, `> `), coloreado con ANSI (`zsh-syntax-highlighting`)
/// o fragmentado entre paquetes TCP.
fn strip_echo_plumbing(text: &str, nonce: &str) -> EchoStripOutcome {
    // Compactar ignorando '\r', '\n' y secuencias ANSI CSI/OSC para mapear offsets reales en `text`
    let mut compact = String::with_capacity(text.len());
    let mut byte_map: Vec<usize> = Vec::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        if bytes[idx] == 0x1b {
            if idx + 1 < bytes.len() && bytes[idx + 1] == b'[' {
                idx += 2;
                while idx < bytes.len() {
                    let b = bytes[idx];
                    idx += 1;
                    if (0x40..=0x7e).contains(&b) {
                        break;
                    }
                }
                continue;
            }
            if idx + 1 < bytes.len() && bytes[idx + 1] == b']' {
                idx += 2;
                while idx < bytes.len() {
                    if bytes[idx] == 0x07 {
                        idx += 1;
                        break;
                    }
                    if bytes[idx] == 0x1b && idx + 1 < bytes.len() && bytes[idx + 1] == b'\\' {
                        idx += 2;
                        break;
                    }
                    idx += 1;
                }
                continue;
            }
        }
        let ch_str = &text[idx..];
        let Some(ch) = ch_str.chars().next() else {
            break;
        };
        let ch_len = ch.len_utf8();
        if ch == '\r' || ch == '\n' {
            idx += ch_len;
            continue;
        }
        compact.push(ch);
        byte_map.push(idx);
        idx += ch_len;
    }

    if let Some(c_start) = compact.find(ECHO_ANCHOR) {
        let orig_start = byte_map[c_start];
        // Retroceder si venía precedido por cierre de bloque `}` o `> ` (bloque multilínea)
        let mut adjusted_start = orig_start;
        let prefix_slice = &text[..orig_start];
        if let Some(before_brace) = prefix_slice.strip_suffix('}') {
            let trimmed_ws = before_brace.trim_end_matches([' ', '>']);
            if trimmed_ws.ends_with('\n') || trimmed_ws.ends_with('\r') {
                adjusted_start = trimmed_ws.len();
            }
        }

        let after_anchor = &compact[c_start..];
        if let Some(rel_end) = after_anchor.find(ECHO_END_TOKEN) {
            let span_compact = &after_anchor[..rel_end + ECHO_END_TOKEN.len()];
            if span_compact.contains(nonce) {
                let last_compact_idx = c_start + rel_end + ECHO_END_TOKEN.len() - 1;
                let orig_end = byte_map[last_compact_idx] + 1;
                let cleaned = format!("{}{}", &text[..adjusted_start], &text[orig_end..]);
                return EchoStripOutcome::Stripped(cleaned);
            }
        }

        // Ancla detectada pero falta cerrar `"$__neko_rc"` en el siguiente chunk (acotado)
        if text.len().saturating_sub(adjusted_start) < MAX_ECHO_TAIL_BYTES {
            return EchoStripOutcome::HoldTail {
                emit_now: text[..adjusted_start].to_string(),
                hold_tail: text[adjusted_start..].to_string(),
            };
        }
        return EchoStripOutcome::NotPresent;
    }

    // Buscar prefijo parcial de `;__neko_rc=$?;printf` al final del paquete
    let max_k = (ECHO_ANCHOR.len() - 1).min(compact.len());
    for k in (2..=max_k).rev() {
        let candidate = &compact[compact.len() - k..];
        if ECHO_ANCHOR.starts_with(candidate) {
            let orig_idx = byte_map[compact.len() - k];
            return EchoStripOutcome::HoldTail {
                emit_now: text[..orig_idx].to_string(),
                hold_tail: text[orig_idx..].to_string(),
            };
        }
    }

    EchoStripOutcome::NotPresent
}

fn floor_char_boundary(s: &str, idx: usize) -> usize {
    let mut i = idx.min(s.len());
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn find_partial_marker_tail(text: &str, marker_head: &str) -> Option<usize> {
    let tail_start = floor_char_boundary(text, text.len().saturating_sub(MAX_PENDING_TAIL_BYTES));
    let tail = &text[tail_start..];

    if let Some(rel) = tail.rfind(SENTINEL_PREFIX) {
        let abs = tail_start + rel;
        if marker_head.starts_with(&text[abs..]) {
            return Some(abs);
        }
    }

    let max_k = marker_head.len().min(tail.len());
    for k in (1..max_k).rev() {
        let candidate_idx = text.len() - k;
        if text.is_char_boundary(candidate_idx) && marker_head.starts_with(&text[candidate_idx..]) {
            return Some(candidate_idx);
        }
    }
    None
}

fn clean_captured_command_output(raw: &str, nonce: &str, max_lines: usize) -> String {
    let cleaned = clean_ansi_output(raw);
    let cont_marker = format!("{}{}", SENTINEL_PREFIX, nonce);
    let filtered: Vec<String> = cleaned
        .lines()
        .filter(|line| {
            !line.contains(&cont_marker)
                && !(line.contains("'__NEKO_END'") && line.contains(nonce))
        })
        .map(|s| s.to_string())
        .collect();
    format_head_tail_lines(&filtered, max_lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::fsm::ExecutionMode;
    use crate::ai::schema::{AgentTaskProposal, RiskLevel};

    #[test]
    fn modo_privacidad_retiene_cero_bytes_y_purga_al_apagar() {
        let reg = PtyTapRegistry::new();

        let res = reg.process_chunk("term-1", "SECRET_LINE\n");
        assert_eq!(res.visible_output, "SECRET_LINE\n");
        assert_eq!(reg.get_context_snapshot("term-1", 100), "");

        reg.set_listening("term-1", true);
        reg.process_chunk("term-1", "line-1\nline-2\n");
        assert_eq!(reg.get_context_snapshot("term-1", 100), "line-1\nline-2");

        reg.set_listening("term-1", false);
        assert_eq!(reg.get_context_snapshot("term-1", 100), "");
    }

    #[test]
    fn limpia_ansi_osc_retornos_cr_y_descarta_pantalla_alterna() {
        let reg = PtyTapRegistry::new();
        reg.set_listening("term-1", true);

        let stream = "\x1b]0;root@vps\x07\x1b[32mOK\x1b[0m\r\n\
            Descargando 10%\rDescargando 100%\r\n\
            \x1b[?1049hCONTENIDO_VIM_SECRETO\nMAS_VIM\x1b[?1049l\
            prompt final\n";

        reg.process_chunk("term-1", stream);
        let snap = reg.get_context_snapshot("term-1", 50);
        assert!(snap.contains("OK"));
        assert!(snap.contains("Descargando 100%"));
        assert!(!snap.contains("Descargando 10%"));
        assert!(!snap.contains("CONTENIDO_VIM_SECRETO"));
        assert!(snap.contains("prompt final"));
    }

    #[test]
    fn presupuesto_head_tail_recorta_salidas_extensas() {
        let lines: Vec<String> = (1..=30).map(|i| format!("linea-{}", i)).collect();
        let out = format_head_tail_lines(&lines, 9);
        assert!(out.contains("linea-1"));
        assert!(out.contains("linea-3"));
        assert!(out.contains("[... 21 líneas omitidas ...]"));
        assert!(out.contains("linea-30"));
        assert!(!out.contains("linea-15\n"));
    }

    #[test]
    fn centinela_posix_detecta_y_oculta_marcador_incluso_fragmentado_entre_paquetes() {
        let reg = PtyTapRegistry::new();
        reg.set_listening("term-1", true);
        reg.arm_sentinel("term-1", "n123");

        let p1 = reg.process_chunk("term-1", "Linux vps 6.1\r\n__NEKO_");
        assert_eq!(p1.visible_output, "Linux vps 6.1\r\n");
        assert!(p1.sentinel_completed.is_none());

        let p2 = reg.process_chunk("term-1", "END_n123:0__\r\nroot@vps:~# ");
        assert_eq!(p2.visible_output, "root@vps:~# ");
        let done = p2.sentinel_completed.expect("debe detectar fin de centinela");
        assert_eq!(done.nonce, "n123");
        assert_eq!(done.exit_code, 0);
        assert!(done.captured_output.contains("Linux vps 6.1"));
        assert!(!done.captured_output.contains("__NEKO_END_"));
    }

    #[test]
    fn wrapper_inmune_a_eco_tty_y_comentarios_en_mismo_paquete_tcp() {
        let wrapped_simple = wrap_command_with_sentinel("mkdir -p /opt/demo", "abc999");
        assert!(!wrapped_simple.contains('\n') || wrapped_simple.trim_end().find('\n').is_none());
        assert!(!wrapped_simple.contains("__NEKO_END_abc999:"));

        let wrapped = wrap_command_with_sentinel("mkdir -p /opt/demo # crear carpeta", "abc999");
        // En el comando escrito NUNCA debe aparecer `__NEKO_END_abc999:` contiguo
        assert!(!wrapped.contains("__NEKO_END_abc999:"));
        assert!(wrapped.contains("\n};__neko_rc=$?"));

        let reg = PtyTapRegistry::new();
        reg.try_arm_sentinel(
            "term-eco",
            "abc999",
            SentinelPurpose::PlanStep {
                step_index: 0,
                command: "mkdir -p /opt/demo".to_string(),
            },
        )
        .expect("arm ok");

        // Colisión mientras sigue en vuelo debe fallar
        assert!(reg
            .try_arm_sentinel(
                "term-eco",
                "otro",
                SentinelPurpose::ReadOnlyRecon {
                    command: "ls".to_string()
                }
            )
            .is_err());

        // Simular eco real fragmentado con wrap 80 cols + resultado + centinela real
        let p1 = reg.process_chunk("term-eco", "root@vps:~#  mkdir -p /opt/demo;__neko_rc=");
        assert_eq!(p1.visible_output, "root@vps:~#  mkdir -p /opt/demo");
        assert!(p1.sentinel_completed.is_none());

        let p2 = reg.process_chunk(
            "term-eco",
            "$?;printf '\\n%s_%s:%d__\\n' '__NEKO_END'\r\n 'abc999' \"$__neko_rc\"\r\nCarpeta creada\r\n__NEKO_END_abc999:0__\r\nroot@vps:~# ",
        );
        assert_eq!(p2.visible_output, "\r\nCarpeta creada\r\nroot@vps:~# ");
        assert!(!p2.visible_output.contains("__neko_rc"));
        assert!(!p2.visible_output.contains("__NEKO_END"));
        let done = p2.sentinel_completed.expect("debe resolver centinela real en mismo paquete");
        assert_eq!(done.exit_code, 0);
        assert!(done.captured_output.contains("Carpeta creada"));

        // Verificar commit atómico sobre FSM (PlanStep último paso -> Completed)
        let mut fsm = AgentHarnessFsm::new("term-eco", ExecutionMode::Strict);
        fsm.start_recon("Demo").unwrap();
        fsm.advance_to_plan(vec![AgentTaskProposal::new(
            1,
            "Crear dir",
            "Preparar ruta",
            "mkdir -p /opt/demo",
            RiskLevel::ConfigChange,
            "ls -ld /opt",
            "rmdir /opt/demo",
        )])
        .unwrap();
        fsm.approve_plan_and_start_execution().unwrap();

        apply_sentinel_completion_to_fsm(&mut fsm, &done);
        assert!(matches!(
            fsm.phase(),
            HarnessPhase::Completed {
                completed_tasks_count: 1,
                ..
            }
        ));
    }

    #[test]
    fn seed_visible_screen_solo_siembra_cuando_escucha_esta_on_y_se_purga_al_apagar() {
        let reg = PtyTapRegistry::new();
        // Con escucha OFF no debe retener absolutamente nada
        reg.seed_visible_screen("term-seed", "linea previa 1\nError: nginx fail\n");
        assert_eq!(reg.get_context_snapshot("term-seed", 50), "");

        // Al encender ON síembra el estado visible actual de pantalla
        reg.set_listening("term-seed", true);
        reg.seed_visible_screen("term-seed", "linea previa 1\nError: nginx fail\n");
        let snap = reg.get_context_snapshot("term-seed", 50);
        assert!(snap.contains("Error: nginx fail"));

        // Al apagar escucha se purga por completo a 0 bytes
        reg.set_listening("term-seed", false);
        assert_eq!(reg.get_context_snapshot("term-seed", 50), "");
    }

    #[test]
    fn disable_all_listening_apaga_escucha_y_purga_multiples_sesiones() {
        let reg = PtyTapRegistry::new();
        reg.set_listening("term-1", true);
        reg.set_listening("term-2", true);
        reg.seed_visible_screen("term-1", "root@a:~# uptime\n");
        reg.seed_visible_screen("term-2", "root@b:~# df -h\n");

        assert!(reg.disable_all_listening());
        assert!(!reg.is_listening("term-1"));
        assert!(!reg.is_listening("term-2"));
        assert_eq!(reg.get_context_snapshot("term-1", 50), "");
        assert_eq!(reg.get_context_snapshot("term-2", 50), "");

        // Idempotente cuando ya no hay ninguna escucha encendida
        assert!(!reg.disable_all_listening());
    }

    #[test]
    fn strip_echo_zsh_con_colores_ansi_y_detecta_espera_interactiva() {
        let reg = PtyTapRegistry::new();
        reg.try_arm_sentinel(
            "term-zsh",
            "z123",
            SentinelPurpose::AuthorizedCommand {
                command: "command -v docker".to_string(),
                purpose: "verificar docker".to_string(),
            },
        )
        .unwrap();

        // Eco coloreado estilo zsh-syntax-highlighting + salida real + centinela
        let colored_chunk = "root@zsh:~# command -v docker\x1b[32m;\x1b[0m__neko_rc=$?\x1b[32m;\x1b[0mprintf '\\n%s_%s:%d__\\n' '__NEKO_END' 'z123' \"$__neko_rc\"\r\n/usr/bin/docker\r\n__NEKO_END_z123:0__\r\nroot@zsh:~# ";
        let res = reg.process_chunk("term-zsh", colored_chunk);
        assert!(res.visible_output.contains("/usr/bin/docker"));
        assert!(!res.visible_output.contains("__neko_rc"));
        assert!(!res.visible_output.contains("__NEKO_END"));
        assert!(res.sentinel_completed.is_some());

        // Detección única por centinela ante prompt de contraseña sudo
        reg.try_arm_sentinel(
            "term-sudo",
            "s999",
            SentinelPurpose::AuthorizedCommand {
                command: "sudo nginx -t".to_string(),
                purpose: "validar nginx".to_string(),
            },
        )
        .unwrap();
        let p_sudo1 = reg.process_chunk("term-sudo", "[sudo] password for deploy: ");
        assert!(p_sudo1.waiting_input_hint.is_some());
        let p_sudo2 = reg.process_chunk("term-sudo", "otra salida");
        assert!(p_sudo2.waiting_input_hint.is_none());

        // Detección temprana ante shell no-POSIX (fish)
        reg.try_arm_sentinel(
            "term-fish",
            "f777",
            SentinelPurpose::AuthorizedCommand {
                command: "uname -a".to_string(),
                purpose: "ver kernel".to_string(),
            },
        )
        .unwrap();
        let p_fish = reg.process_chunk(
            "term-fish",
            "fish: Unsupported use of '='. In fish, please use 'set __neko_rc $status'.\r\n",
        );
        let hint_fish = p_fish.waiting_input_hint.expect("debe detectar error fish");
        assert!(hint_fish.to_lowercase().contains("fish"));
    }

    #[test]
    fn salida_capturada_respeta_limite_dinamico_max_capture_lines() {
        let reg = PtyTapRegistry::new();
        assert_eq!(reg.max_capture_lines(), 200);

        // Configurar 400 líneas debe conservar 300 líneas intactas sin truncar a 200
        reg.set_max_capture_lines(400);
        assert_eq!(reg.max_capture_lines(), 400);

        let lines_300: String = (1..=300)
            .map(|i| format!("row-{}\r\n", i))
            .collect();
        reg.arm_sentinel("term-cap", "c400");
        let chunk_big = format!("{lines_300}__NEKO_END_c400:0__\r\n");
        let res_big = reg.process_chunk("term-cap", &chunk_big);
        let done_big = res_big.sentinel_completed.expect("completado");
        assert!(!done_big.captured_output.contains("líneas omitidas"));
        assert!(done_big.captured_output.contains("row-150"));

        // Configurar 50 líneas debe recortar a 50 usando Head + Tail (también acotado al mínimo seguro 50)
        reg.set_max_capture_lines(10);
        assert_eq!(reg.max_capture_lines(), 50);
        reg.arm_sentinel("term-cap", "c50");
        let res_small = reg.process_chunk("term-cap", &format!("{lines_300}__NEKO_END_c50:0__\r\n"));
        let done_small = res_small.sentinel_completed.expect("completado");
        assert!(done_small.captured_output.contains("líneas omitidas"));
    }

    #[test]
    fn osc7_actualiza_cwd_incluso_con_escucha_off_y_reporta_telemetria() {
        let reg = PtyTapRegistry::new();

        // Con Escucha OFF no debe retener bytes de stdout pero sí extraer cwd OSC 7 (incluso fragmentado)
        let r1 = reg.process_chunk("term-osc", "SECRET_PASSWORD=12345\r\n\x1b]7;file://vps/var/log/");
        assert!(r1.cwd_changed.is_none());
        let r2 = reg.process_chunk("term-osc", "nginx\x07root@vps:/var/log/nginx# ");
        assert_eq!(r2.cwd_changed.as_deref(), Some("/var/log/nginx"));
        assert_eq!(reg.current_cwd("term-osc").as_deref(), Some("/var/log/nginx"));
        assert_eq!(reg.captured_line_count("term-osc"), 0);
        assert_eq!(reg.get_context_snapshot("term-osc", 100), "");

        // Al activar Escucha ON cuenta las líneas reales y al apagar conserva current_cwd pero purga líneas
        reg.set_listening("term-osc", true);
        reg.process_chunk("term-osc", "linea 1\nlinea 2\nlinea 3\n");
        assert_eq!(reg.captured_line_count("term-osc"), 3);

        reg.set_listening("term-osc", false);
        assert_eq!(reg.captured_line_count("term-osc"), 0);
        assert_eq!(reg.current_cwd("term-osc").as_deref(), Some("/var/log/nginx"));
    }
}
