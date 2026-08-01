//! Harness local: clasificación + path elevado mock (fake SFTP/exec).
//! MUST NOT connect ni escribir a hosts SSH de lab.
//!
//! Ejecutar: `cargo run --manifest-path app/src-tauri/Cargo.toml --example smoke_elevated_upload_local`

use app_lib::elevated_upload::{
    build_sudo_cp_command, classify_upload_error, run_elevated_upload, EditUploadError,
    ExecOutcome, UploadErrorKind,
};
use app_lib::fake_sftp::FakeSftpStore;
use std::sync::{Arc, Mutex};

fn main() {
    println!("=== smoke_elevated_upload_local (mock/local only; cero writes al lab SSH) ===");

    assert_eq!(
        classify_upload_error("Error al crear remoto /etc/x: Permission denied"),
        UploadErrorKind::PermissionDenied
    );
    assert_eq!(
        classify_upload_error("No hay sesión SSH para esta terminal"),
        UploadErrorKind::Disconnected
    );
    let elev = EditUploadError::from_upload_message("Permission denied");
    assert!(elev.elevatable);
    let no = EditUploadError::from_upload_message("connection reset");
    assert!(!no.elevatable);
    println!("OK clasificación elevable vs no elevable");

    let cmd = build_sudo_cp_command("/tmp/nekossh-edit-e1-hosts", "/etc/hosts").unwrap();
    assert_eq!(
        cmd,
        "sudo -n cp -- '/tmp/nekossh-edit-e1-hosts' '/etc/hosts'"
    );
    assert!(build_sudo_cp_command("/tmp/a\0", "/etc/x").is_err());
    println!("OK builder sudo -n cp + rechazo NUL");

    let store = FakeSftpStore::new();
    store.insert("/etc/hosts", b"old\n");
    let tmp_root = std::env::temp_dir().join(format!(
        "nekossh-smoke-elev-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&tmp_root).unwrap();
    let local = tmp_root.join("hosts");
    std::fs::write(&local, b"new\n").unwrap();

    // Upload normal denegado a /etc → elevable
    let deny_err = store
        .upload_from_local_denying(&local, "/etc/hosts", &["/etc"])
        .unwrap_err();
    assert_eq!(
        classify_upload_error(&deny_err),
        UploadErrorKind::PermissionDenied
    );
    println!("OK fake upload permission_denied");

    // Path elevado exitoso (mock exec copia temp → destino)
    let store_ok = store.clone();
    run_elevated_upload(
        &local,
        "/etc/hosts",
        "e-smoke",
        |l, temp| store_ok.upload_from_local(l, temp),
        |cmd| {
            assert!(cmd.starts_with("sudo -n cp -- "));
            // Extrae paths quoted: sudo -n cp -- 'temp' 'dest'
            let parts: Vec<&str> = cmd.split('\'').collect();
            // ['sudo -n cp -- ', temp, ' ', dest, '']
            let temp = parts.get(1).copied().unwrap_or("");
            let dest = parts.get(3).copied().unwrap_or("");
            store_ok.copy_remote(temp, dest).unwrap();
            Ok(ExecOutcome {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        },
        |temp| store_ok.unlink(temp),
    )
    .expect("elevated success");
    assert_eq!(store.get("/etc/hosts").unwrap(), b"new\n");
    assert!(store.get("/tmp/nekossh-edit-e-smoke-hosts").is_none());
    println!("OK elevated mock success + cleanup temp");

    // sudo_password_required
    let store_pw = FakeSftpStore::new();
    let err_pw = run_elevated_upload(
        &local,
        "/etc/shadow",
        "e-pw",
        |l, t| store_pw.upload_from_local(l, t),
        |_cmd| {
            Ok(ExecOutcome {
                exit_code: 1,
                stdout: String::new(),
                stderr: "sudo: a password is required".into(),
            })
        },
        |t| store_pw.unlink(t),
    )
    .unwrap_err();
    assert_eq!(err_pw.kind, UploadErrorKind::SudoPasswordRequired);
    println!("OK sudo_password_required");

    // sudo_failed
    let store_fail = FakeSftpStore::new();
    let err_fail = run_elevated_upload(
        &local,
        "/etc/shadow",
        "e-fail",
        |l, t| store_fail.upload_from_local(l, t),
        |_cmd| {
            Ok(ExecOutcome {
                exit_code: 1,
                stdout: String::new(),
                stderr: "cp: cannot create regular file".into(),
            })
        },
        |t| store_fail.unlink(t),
    )
    .unwrap_err();
    assert_eq!(err_fail.kind, UploadErrorKind::SudoFailed);
    println!("OK sudo_failed");

    // Modelo de sesión: tras fail_upload la fase vuelve a Watching (sin PTY roto — N/A mock)
    use app_lib::edit_session::{EditSessionPhase, EditSessionRegistry};
    use app_lib::edit_util::content_fingerprint;
    let mut reg = EditSessionRegistry::new();
    let info = reg.register_or_reuse(
        "e1".into(),
        "term".into(),
        "/etc/hosts".into(),
        local.clone(),
        content_fingerprint(b"new\n"),
    );
    reg.mark_confirm_pending(&info.edit_id);
    reg.begin_upload(&info.edit_id).unwrap();
    reg.fail_upload(&info.edit_id);
    assert_eq!(
        reg.get(&info.edit_id).unwrap().phase,
        EditSessionPhase::Watching
    );
    assert!(local.exists(), "temp dirty local conservado");
    println!("OK fase Watching tras fallo; temp local conservado");

    // Contador de intentos elevados: exactamente uno por aceptación
    let exec_count = Arc::new(Mutex::new(0u32));
    let ec = exec_count.clone();
    let store_once = FakeSftpStore::new();
    let _ = run_elevated_upload(
        &local,
        "/etc/hosts",
        "e-once",
        |l, t| store_once.upload_from_local(l, t),
        |_cmd| {
            *ec.lock().unwrap() += 1;
            Ok(ExecOutcome {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        },
        |t| store_once.unlink(t),
    );
    assert_eq!(*exec_count.lock().unwrap(), 1);
    println!("OK un solo intento elevado por aceptación");

    let _ = std::fs::remove_dir_all(&tmp_root);
    println!("OK cleanup temps locales");
    println!("=== PASS (sin mutaciones al lab SSH) ===");
}
