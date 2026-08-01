//! Clasificación de fallos de upload y path elevado (temp remoto + sudo -n cp).
//! Tests/harness: solo mock/fake — cero writes a hosts SSH de lab.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::edit_util::remote_basename;
use crate::path_util::shell_quote;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UploadErrorKind {
    PermissionDenied,
    SudoPasswordRequired,
    SudoFailed,
    Disconnected,
    NotFound,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditUploadError {
    pub kind: UploadErrorKind,
    pub message: String,
    pub elevatable: bool,
}

impl EditUploadError {
    pub fn new(kind: UploadErrorKind, message: impl Into<String>) -> Self {
        let elevatable = matches!(kind, UploadErrorKind::PermissionDenied);
        Self {
            kind,
            message: message.into(),
            elevatable,
        }
    }

    pub fn from_upload_message(message: impl Into<String>) -> Self {
        let message = message.into();
        let kind = classify_upload_error(&message);
        Self::new(kind, message)
    }
}

impl std::fmt::Display for EditUploadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Clasifica un mensaje de fallo SFTP/upload: permiso → elevable; resto no.
pub fn classify_upload_error(message: &str) -> UploadErrorKind {
    let lower = message.to_ascii_lowercase();
    if lower.contains('\0') {
        return UploadErrorKind::Other;
    }
    if lower.contains("permission denied")
        || lower.contains("operation not permitted")
        || lower.contains("eacces")
        || lower.contains("eperm")
        || lower.contains("access denied")
        || lower.contains("escritura denegada")
        || lower.contains("permiso denegado")
        || lower.contains("read-only file system")
        || (lower.contains("permission") && lower.contains("denied"))
    {
        return UploadErrorKind::PermissionDenied;
    }
    if lower.contains("no hay sesión ssh")
        || lower.contains("disconnected")
        || lower.contains("desconect")
        || lower.contains("connection reset")
        || lower.contains("connection refused")
        || lower.contains("broken pipe")
        || lower.contains("session closed")
        || lower.contains("sesión cerrada")
        || lower.contains("not connected")
        || lower.contains("transport")
    {
        return UploadErrorKind::Disconnected;
    }
    if lower.contains("no such file")
        || lower.contains("not found")
        || lower.contains("inexistente")
        || lower.contains("enoent")
    {
        return UploadErrorKind::NotFound;
    }
    UploadErrorKind::Other
}

/// Path temp remoto writable: `/tmp/nekossh-edit-<edit_id>-<basename>`.
pub fn remote_elevated_temp_path(edit_id: &str, remote_dest: &str) -> Result<String, String> {
    if edit_id.contains('\0') || remote_dest.contains('\0') {
        return Err("Path con NUL no permitido".to_string());
    }
    let base = remote_basename(remote_dest);
    let safe_id: String = edit_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let safe_base: String = base
        .chars()
        .map(|c| {
            if c == '/' || c == '\\' || c == '\0' {
                '_'
            } else {
                c
            }
        })
        .collect();
    Ok(format!("/tmp/nekossh-edit-{}-{}", safe_id, safe_base))
}

/// `sudo -n cp -- <temp> <dest>` con quoting seguro. Rechaza NUL.
pub fn build_sudo_cp_command(temp_remote: &str, dest_remote: &str) -> Result<String, String> {
    if temp_remote.contains('\0') || dest_remote.contains('\0') {
        return Err("Path con NUL no permitido".to_string());
    }
    if temp_remote.is_empty() || dest_remote.is_empty() {
        return Err("Path remoto vacío".to_string());
    }
    Ok(format!(
        "sudo -n cp -- {} {}",
        shell_quote(temp_remote),
        shell_quote(dest_remote)
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Clasifica salida de sudo/exec no interactivo.
pub fn classify_sudo_exec(outcome: &ExecOutcome) -> Result<(), EditUploadError> {
    if outcome.exit_code == 0 {
        return Ok(());
    }
    let combined = format!("{}\n{}", outcome.stdout, outcome.stderr).to_ascii_lowercase();
    if combined.contains("password is required")
        || combined.contains("a terminal is required")
        || combined.contains("no tty")
        || combined.contains("tty present")
        || combined.contains("must be run from a terminal")
        || combined.contains("sudo: a password")
        || (combined.contains("password") && combined.contains("sudo"))
    {
        return Err(EditUploadError::new(
            UploadErrorKind::SudoPasswordRequired,
            "sudo requiere contraseña o no está disponible de forma no interactiva. No se solicita contraseña en la app.",
        ));
    }
    Err(EditUploadError::new(
        UploadErrorKind::SudoFailed,
        format!(
            "El comando sudo falló (código {}). {}",
            outcome.exit_code,
            outcome.stderr.trim()
        ),
    ))
}

/// Orquesta upload a temp + sudo cp + cleanup. Inyectable para mock/fake.
pub fn run_elevated_upload<U, E, C>(
    local_path: &Path,
    remote_dest: &str,
    edit_id: &str,
    upload_temp: U,
    exec_cmd: E,
    cleanup_temp: C,
) -> Result<(), EditUploadError>
where
    U: FnOnce(&Path, &str) -> Result<(), String>,
    E: FnOnce(&str) -> Result<ExecOutcome, String>,
    C: FnOnce(&str),
{
    let temp = remote_elevated_temp_path(edit_id, remote_dest)
        .map_err(|m| EditUploadError::new(UploadErrorKind::Other, m))?;
    let cmd = build_sudo_cp_command(&temp, remote_dest)
        .map_err(|m| EditUploadError::new(UploadErrorKind::Other, m))?;

    if let Err(e) = upload_temp(local_path, &temp) {
        cleanup_temp(&temp);
        return Err(EditUploadError::from_upload_message(e));
    }

    let exec_res = exec_cmd(&cmd);
    cleanup_temp(&temp);

    match exec_res {
        Ok(outcome) => classify_sudo_exec(&outcome),
        Err(e) => {
            let kind = classify_upload_error(&e);
            let kind = if matches!(kind, UploadErrorKind::Disconnected) {
                UploadErrorKind::Disconnected
            } else {
                UploadErrorKind::SudoFailed
            };
            Err(EditUploadError::new(kind, e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn permission_denied_es_elevable() {
        let cases = [
            "Error al crear remoto /etc/hosts: Permission denied",
            "EACCES: permission denied",
            "Operation not permitted",
            "Escritura denegada",
            "permiso denegado al escribir",
        ];
        for msg in cases {
            let err = EditUploadError::from_upload_message(msg);
            assert_eq!(err.kind, UploadErrorKind::PermissionDenied, "{msg}");
            assert!(err.elevatable, "{msg}");
        }
    }

    #[test]
    fn disconnect_y_not_found_no_elevables() {
        let disconnect = EditUploadError::from_upload_message("No hay sesión SSH para esta terminal");
        assert_eq!(disconnect.kind, UploadErrorKind::Disconnected);
        assert!(!disconnect.elevatable);

        let reset = EditUploadError::from_upload_message("connection reset by peer");
        assert_eq!(reset.kind, UploadErrorKind::Disconnected);
        assert!(!reset.elevatable);

        let missing = EditUploadError::from_upload_message("no such file or directory");
        assert_eq!(missing.kind, UploadErrorKind::NotFound);
        assert!(!missing.elevatable);

        let other = EditUploadError::from_upload_message("disco lleno / ENOSPC");
        assert_eq!(other.kind, UploadErrorKind::Other);
        assert!(!other.elevatable);
    }

    #[test]
    fn build_sudo_cp_quoting_y_nul() {
        let cmd = build_sudo_cp_command("/tmp/a", "/etc/hosts").unwrap();
        assert_eq!(cmd, "sudo -n cp -- '/tmp/a' '/etc/hosts'");

        let quoted = build_sudo_cp_command("/tmp/it's", "/var/lib/x").unwrap();
        assert!(quoted.contains("'\"'\"'"));
        assert!(quoted.starts_with("sudo -n cp -- "));

        assert!(build_sudo_cp_command("/tmp/a\0b", "/etc/x").is_err());
        assert!(build_sudo_cp_command("/tmp/a", "/etc/\0x").is_err());
        assert!(remote_elevated_temp_path("id\0x", "/etc/hosts").is_err());
    }

    #[test]
    fn remote_temp_path_prefijo() {
        let p = remote_elevated_temp_path("edit-1", "/etc/nginx/nginx.conf").unwrap();
        assert_eq!(p, "/tmp/nekossh-edit-edit-1-nginx.conf");
    }

    #[test]
    fn sudo_password_required_desde_stderr() {
        let err = classify_sudo_exec(&ExecOutcome {
            exit_code: 1,
            stdout: String::new(),
            stderr: "sudo: a password is required".into(),
        })
        .unwrap_err();
        assert_eq!(err.kind, UploadErrorKind::SudoPasswordRequired);
        assert!(!err.elevatable);
    }

    #[test]
    fn sudo_failed_generico() {
        let err = classify_sudo_exec(&ExecOutcome {
            exit_code: 1,
            stdout: String::new(),
            stderr: "cp: cannot create".into(),
        })
        .unwrap_err();
        assert_eq!(err.kind, UploadErrorKind::SudoFailed);
    }

    #[test]
    fn orchestrate_exito_mock() {
        let uploaded = Arc::new(Mutex::new(None::<String>));
        let executed = Arc::new(Mutex::new(None::<String>));
        let cleaned = Arc::new(Mutex::new(None::<String>));
        let up = uploaded.clone();
        let ex = executed.clone();
        let cl = cleaned.clone();
        let tmp = std::env::temp_dir().join(format!("nekossh-elev-ok-{}", std::process::id()));
        std::fs::write(&tmp, b"hola").unwrap();
        run_elevated_upload(
            &tmp,
            "/etc/hosts",
            "e1",
            |local, remote| {
                assert!(local.exists());
                *up.lock().unwrap() = Some(remote.to_string());
                Ok(())
            },
            |cmd| {
                *ex.lock().unwrap() = Some(cmd.to_string());
                Ok(ExecOutcome {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                })
            },
            |t| *cl.lock().unwrap() = Some(t.to_string()),
        )
        .expect("elevated ok");
        assert!(uploaded.lock().unwrap().as_ref().unwrap().starts_with("/tmp/nekossh-edit-"));
        assert!(executed
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .starts_with("sudo -n cp -- "));
        assert!(cleaned.lock().unwrap().is_some());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn orchestrate_password_required_conserva_flujo() {
        let cleaned = Arc::new(Mutex::new(false));
        let cl = cleaned.clone();
        let tmp = std::env::temp_dir().join(format!("nekossh-elev-pw-{}", std::process::id()));
        std::fs::write(&tmp, b"x").unwrap();
        let err = run_elevated_upload(
            &tmp,
            "/etc/shadow",
            "e2",
            |_l, _r| Ok(()),
            |_cmd| {
                Ok(ExecOutcome {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: "sudo: a password is required".into(),
                })
            },
            |_t| *cl.lock().unwrap() = true,
        )
        .unwrap_err();
        assert_eq!(err.kind, UploadErrorKind::SudoPasswordRequired);
        assert!(*cleaned.lock().unwrap());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn orchestrate_upload_temp_permission_propaga() {
        let tmp = std::env::temp_dir().join(format!("nekossh-elev-perm-{}", std::process::id()));
        std::fs::write(&tmp, b"x").unwrap();
        let err = run_elevated_upload(
            &tmp,
            "/etc/hosts",
            "e3",
            |_l, _r| Err("Permission denied".into()),
            |_cmd| panic!("exec no debe llamarse"),
            |_t| {},
        )
        .unwrap_err();
        assert_eq!(err.kind, UploadErrorKind::PermissionDenied);
        assert!(err.elevatable);
        let _ = std::fs::remove_file(&tmp);
    }
}
