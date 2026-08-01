//! Registro de sesiones de edición externa + debounce/coalesce.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::edit_util::{content_fingerprint, EDIT_WATCH_DEBOUNCE_MS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditSessionPhase {
    Watching,
    ConfirmPending,
    Uploading,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditSessionInfo {
    pub edit_id: String,
    pub terminal_id: String,
    pub remote_path: String,
    pub local_path: String,
    pub reused: bool,
    pub phase: EditSessionPhase,
}

#[derive(Debug, Clone)]
pub struct EditSessionRecord {
    pub edit_id: String,
    pub terminal_id: String,
    pub remote_path: String,
    pub local_path: PathBuf,
    pub baseline_fingerprint: String,
    pub phase: EditSessionPhase,
    pub preserve_temp_on_close: bool,
    /// Último evento de FS observado (para debounce).
    pub last_fs_event: Option<Instant>,
    /// Cambio real pendiente de emitir al frontend.
    pub pending_change_emit: bool,
}

/// Estado compartido de edit sessions.
pub struct EditSessionRegistry {
    by_id: HashMap<String, EditSessionRecord>,
    /// key = terminal_id\0remote_path
    index: HashMap<String, String>,
}

impl Default for EditSessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EditSessionRegistry {
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            index: HashMap::new(),
        }
    }

    fn index_key(terminal_id: &str, remote_path: &str) -> String {
        format!("{}\0{}", terminal_id, remote_path)
    }

    pub fn find(
        &self,
        terminal_id: &str,
        remote_path: &str,
    ) -> Option<&EditSessionRecord> {
        let id = self.index.get(&Self::index_key(terminal_id, remote_path))?;
        self.by_id.get(id)
    }

    pub fn get(&self, edit_id: &str) -> Option<&EditSessionRecord> {
        self.by_id.get(edit_id)
    }

    pub fn get_mut(&mut self, edit_id: &str) -> Option<&mut EditSessionRecord> {
        self.by_id.get_mut(edit_id)
    }

    /// Registra o reutiliza sesión. Si reutiliza, no cambia paths/baseline.
    pub fn register_or_reuse(
        &mut self,
        edit_id: String,
        terminal_id: String,
        remote_path: String,
        local_path: PathBuf,
        baseline_fingerprint: String,
    ) -> EditSessionInfo {
        let key = Self::index_key(&terminal_id, &remote_path);
        if let Some(existing_id) = self.index.get(&key).cloned() {
            if let Some(rec) = self.by_id.get(&existing_id) {
                if rec.phase != EditSessionPhase::Closed {
                    return EditSessionInfo {
                        edit_id: rec.edit_id.clone(),
                        terminal_id: rec.terminal_id.clone(),
                        remote_path: rec.remote_path.clone(),
                        local_path: rec.local_path.to_string_lossy().into_owned(),
                        reused: true,
                        phase: rec.phase,
                    };
                }
            }
        }

        let info = EditSessionInfo {
            edit_id: edit_id.clone(),
            terminal_id: terminal_id.clone(),
            remote_path: remote_path.clone(),
            local_path: local_path.to_string_lossy().into_owned(),
            reused: false,
            phase: EditSessionPhase::Watching,
        };

        self.index.insert(key, edit_id.clone());
        self.by_id.insert(
            edit_id.clone(),
            EditSessionRecord {
                edit_id,
                terminal_id,
                remote_path,
                local_path,
                baseline_fingerprint,
                phase: EditSessionPhase::Watching,
                preserve_temp_on_close: false,
                last_fs_event: None,
                pending_change_emit: false,
            },
        );
        info
    }

    pub fn mark_confirm_pending(&mut self, edit_id: &str) -> bool {
        if let Some(rec) = self.by_id.get_mut(edit_id) {
            if rec.phase == EditSessionPhase::Watching {
                rec.phase = EditSessionPhase::ConfirmPending;
                rec.pending_change_emit = false;
                return true;
            }
            // Ya hay confirm pendiente: coalesce (no apilar)
            if rec.phase == EditSessionPhase::ConfirmPending {
                return false;
            }
        }
        false
    }

    pub fn dismiss_confirm(&mut self, edit_id: &str) {
        if let Some(rec) = self.by_id.get_mut(edit_id) {
            if rec.phase == EditSessionPhase::ConfirmPending {
                rec.phase = EditSessionPhase::Watching;
            }
        }
    }

    pub fn begin_upload(&mut self, edit_id: &str) -> Result<(), String> {
        let rec = self
            .by_id
            .get_mut(edit_id)
            .ok_or_else(|| "Sesión de edición no encontrada".to_string())?;
        if rec.phase != EditSessionPhase::ConfirmPending
            && rec.phase != EditSessionPhase::Watching
        {
            return Err("La sesión no puede subir en este estado".to_string());
        }
        rec.phase = EditSessionPhase::Uploading;
        Ok(())
    }

    pub fn finish_upload(&mut self, edit_id: &str, new_fingerprint: String) {
        if let Some(rec) = self.by_id.get_mut(edit_id) {
            rec.baseline_fingerprint = new_fingerprint;
            rec.phase = EditSessionPhase::Watching;
            rec.pending_change_emit = false;
        }
    }

    pub fn fail_upload(&mut self, edit_id: &str) {
        if let Some(rec) = self.by_id.get_mut(edit_id) {
            rec.phase = EditSessionPhase::Watching;
        }
    }

    /// Nota un evento FS; actualiza debounce. Devuelve true si hay que programar timer.
    pub fn note_fs_event(&mut self, edit_id: &str) {
        if let Some(rec) = self.by_id.get_mut(edit_id) {
            if rec.phase == EditSessionPhase::Uploading
                || rec.phase == EditSessionPhase::Closed
            {
                return;
            }
            rec.last_fs_event = Some(Instant::now());
            rec.pending_change_emit = true;
        }
    }

    /// Tras debounce: si el fingerprint cambió vs baseline y no hay confirm pendiente,
    /// marca confirm_pending y devuelve el record para emitir evento.
    pub fn evaluate_after_debounce(
        &mut self,
        edit_id: &str,
        current_fingerprint: &str,
        debounce: Duration,
        now: Instant,
    ) -> Option<EditSessionInfo> {
        let rec = self.by_id.get_mut(edit_id)?;
        if !rec.pending_change_emit {
            return None;
        }
        let Some(last) = rec.last_fs_event else {
            return None;
        };
        if now.duration_since(last) < debounce {
            return None;
        }
        if current_fingerprint == rec.baseline_fingerprint {
            rec.pending_change_emit = false;
            return None;
        }
        if rec.phase == EditSessionPhase::ConfirmPending {
            // Coalesce: no apilar otro dialog
            rec.pending_change_emit = false;
            return None;
        }
        if rec.phase != EditSessionPhase::Watching {
            return None;
        }
        rec.phase = EditSessionPhase::ConfirmPending;
        rec.pending_change_emit = false;
        Some(EditSessionInfo {
            edit_id: rec.edit_id.clone(),
            terminal_id: rec.terminal_id.clone(),
            remote_path: rec.remote_path.clone(),
            local_path: rec.local_path.to_string_lossy().into_owned(),
            reused: false,
            phase: rec.phase,
        })
    }

    /// Cierra sesiones de un terminal. Si `preserve_temps`, no marca borrado inmediato.
    pub fn take_for_terminal(
        &mut self,
        terminal_id: &str,
        preserve_temps: bool,
    ) -> Vec<EditSessionRecord> {
        let ids: Vec<String> = self
            .by_id
            .values()
            .filter(|r| r.terminal_id == terminal_id)
            .map(|r| r.edit_id.clone())
            .collect();
        let mut out = Vec::new();
        for id in ids {
            if let Some(mut rec) = self.by_id.remove(&id) {
                self.index
                    .remove(&Self::index_key(&rec.terminal_id, &rec.remote_path));
                rec.preserve_temp_on_close = preserve_temps
                    || rec.phase == EditSessionPhase::ConfirmPending
                    || rec.phase == EditSessionPhase::Uploading;
                // Durante upload/confirm no borrar (design)
                if rec.phase == EditSessionPhase::Uploading
                    || rec.phase == EditSessionPhase::ConfirmPending
                {
                    rec.preserve_temp_on_close = true;
                }
                rec.phase = EditSessionPhase::Closed;
                out.push(rec);
            }
        }
        out
    }

    pub fn remove(&mut self, edit_id: &str) -> Option<EditSessionRecord> {
        let rec = self.by_id.remove(edit_id)?;
        self.index
            .remove(&Self::index_key(&rec.terminal_id, &rec.remote_path));
        Some(rec)
    }

    pub fn debounce_duration() -> Duration {
        Duration::from_millis(EDIT_WATCH_DEBOUNCE_MS)
    }
}

pub type SharedEditSessions = Arc<Mutex<EditSessionRegistry>>;

pub fn new_shared_registry() -> SharedEditSessions {
    Arc::new(Mutex::new(EditSessionRegistry::new()))
}

/// Payload del evento `edit-session-changed`.
#[derive(Clone, Serialize)]
pub struct EditSessionChangedPayload {
    pub edit_id: String,
    pub terminal_id: String,
    pub remote_path: String,
    pub reason: String,
}

/// Payload disconnect mid-edit.
#[derive(Clone, Serialize)]
pub struct EditSessionDisconnectedPayload {
    pub terminal_id: String,
    pub edit_ids: Vec<String>,
    pub message: String,
}

pub fn fingerprint_from_bytes(data: &[u8]) -> String {
    content_fingerprint(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn reusa_misma_terminal_y_remote_path() {
        let mut reg = EditSessionRegistry::new();
        let a = reg.register_or_reuse(
            "id-1".into(),
            "term-a".into(),
            "/home/x/a.txt".into(),
            PathBuf::from("/tmp/a.txt"),
            "fp1".into(),
        );
        assert!(!a.reused);
        let b = reg.register_or_reuse(
            "id-2".into(),
            "term-a".into(),
            "/home/x/a.txt".into(),
            PathBuf::from("/tmp/other.txt"),
            "fp2".into(),
        );
        assert!(b.reused);
        assert_eq!(b.edit_id, "id-1");
        assert_eq!(b.local_path, "/tmp/a.txt");
    }

    #[test]
    fn coalesce_confirm_no_apila() {
        let mut reg = EditSessionRegistry::new();
        let info = reg.register_or_reuse(
            "e1".into(),
            "t".into(),
            "/f".into(),
            PathBuf::from("/tmp/f"),
            "base".into(),
        );
        assert!(reg.mark_confirm_pending(&info.edit_id));
        assert!(!reg.mark_confirm_pending(&info.edit_id));
    }

    #[test]
    fn debounce_emite_solo_si_fingerprint_cambio() {
        let mut reg = EditSessionRegistry::new();
        let info = reg.register_or_reuse(
            "e1".into(),
            "t".into(),
            "/f".into(),
            PathBuf::from("/tmp/f"),
            "base".into(),
        );
        reg.note_fs_event(&info.edit_id);
        let now = Instant::now() + Duration::from_secs(2);
        let none = reg.evaluate_after_debounce(
            &info.edit_id,
            "base",
            Duration::from_millis(10),
            now,
        );
        assert!(none.is_none());

        reg.note_fs_event(&info.edit_id);
        let changed = reg.evaluate_after_debounce(
            &info.edit_id,
            "dirty",
            Duration::from_millis(10),
            now + Duration::from_secs(1),
        );
        assert!(changed.is_some());
        assert_eq!(changed.unwrap().phase, EditSessionPhase::ConfirmPending);

        // Coalesce: segundo evaluate con confirm pendiente no re-emite
        reg.note_fs_event(&info.edit_id);
        let again = reg.evaluate_after_debounce(
            &info.edit_id,
            "dirty2",
            Duration::from_millis(10),
            now + Duration::from_secs(3),
        );
        assert!(again.is_none());
    }

    #[test]
    fn disconnect_preserva_temp_si_confirm() {
        let mut reg = EditSessionRegistry::new();
        let info = reg.register_or_reuse(
            "e1".into(),
            "term".into(),
            "/f".into(),
            PathBuf::from("/tmp/f"),
            "base".into(),
        );
        reg.mark_confirm_pending(&info.edit_id);
        let taken = reg.take_for_terminal("term", true);
        assert_eq!(taken.len(), 1);
        assert!(taken[0].preserve_temp_on_close);
        assert!(reg.get("e1").is_none());
    }

    #[test]
    fn no_borrar_logica_durante_upload() {
        let mut reg = EditSessionRegistry::new();
        let info = reg.register_or_reuse(
            "e1".into(),
            "term".into(),
            "/f".into(),
            PathBuf::from("/tmp/f"),
            "base".into(),
        );
        reg.begin_upload(&info.edit_id).unwrap();
        let taken = reg.take_for_terminal("term", false);
        assert!(taken[0].preserve_temp_on_close);
    }
}
