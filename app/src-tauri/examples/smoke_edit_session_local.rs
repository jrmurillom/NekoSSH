//! Harness local: preferencias + fake SFTP edit cycle.
//! MUST NOT connect ni escribir a hosts SSH de lab.
//!
//! Ejecutar: `cargo run --manifest-path app/src-tauri/Cargo.toml --example smoke_edit_session_local`

use app_lib::edit_session::{EditSessionPhase, EditSessionRegistry};
use app_lib::edit_util::{
    content_fingerprint, edit_session_dir, exceeds_edit_size_limit, local_edit_file_path,
    looks_binary, MAX_EXTERNAL_EDIT_BYTES,
};
use app_lib::fake_sftp::FakeSftpStore;
use app_lib::preferences::{
    ensure_app_preferences_schema, get_preferred_external_editor, set_preferred_external_editor,
};
use rusqlite::Connection;
use std::path::PathBuf;

fn main() {
    println!("=== smoke_edit_session_local (mock/local only; cero writes al lab SSH) ===");

    let conn = Connection::open_in_memory().expect("mem db");
    ensure_app_preferences_schema(&conn).expect("prefs schema");
    assert_eq!(get_preferred_external_editor(&conn).unwrap(), "");
    set_preferred_external_editor(&conn, r"C:\Editors\fake-editor.exe").unwrap();
    assert_eq!(
        get_preferred_external_editor(&conn).unwrap(),
        r"C:\Editors\fake-editor.exe"
    );
    println!("OK preferencias get/set round-trip");

    let store = FakeSftpStore::new();
    store.insert("/home/neko/app.conf", b"listen=80\n");
    let tmp_root = std::env::temp_dir().join(format!(
        "nekossh-smoke-edit-{}",
        std::process::id()
    ));
    let session_dir = edit_session_dir(&tmp_root, "edit-smoke-1");
    let local = local_edit_file_path(&session_dir, "app.conf");
    store
        .download_to_local("/home/neko/app.conf", &local)
        .expect("download");
    assert_eq!(std::fs::read(&local).unwrap(), b"listen=80\n");
    std::fs::write(&local, b"listen=443\n").unwrap();
    store
        .upload_from_local(&local, "/home/neko/app.conf")
        .expect("upload mock");
    assert_eq!(store.get("/home/neko/app.conf").unwrap(), b"listen=443\n");
    println!("OK fake SFTP download/upload round-trip");

    let big = vec![b'x'; (MAX_EXTERNAL_EDIT_BYTES as usize) + 1];
    store.insert("/huge.bin", big);
    assert!(exceeds_edit_size_limit(store.file_size("/huge.bin").unwrap()));
    let reject_path = session_dir.join("reject.bin");
    assert!(store
        .download_to_local("/huge.bin", &reject_path)
        .is_err());
    println!("OK rechazo por tamaño >10 MiB");

    assert!(looks_binary(b"\0bin"));
    assert!(!looks_binary(b"texto"));
    println!("OK heurística binaria");

    let mut reg = EditSessionRegistry::new();
    let info = reg.register_or_reuse(
        "e1".into(),
        "term-1".into(),
        "/home/neko/app.conf".into(),
        local.clone(),
        content_fingerprint(b"listen=443\n"),
    );
    assert!(!info.reused);
    let again = reg.register_or_reuse(
        "e2".into(),
        "term-1".into(),
        "/home/neko/app.conf".into(),
        PathBuf::from("/tmp/other"),
        "other".into(),
    );
    assert!(again.reused);
    assert_eq!(again.edit_id, "e1");
    assert!(reg.mark_confirm_pending("e1"));
    assert!(!reg.mark_confirm_pending("e1"));
    assert_eq!(
        reg.get("e1").unwrap().phase,
        EditSessionPhase::ConfirmPending
    );
    reg.dismiss_confirm("e1");
    assert_eq!(reg.get("e1").unwrap().phase, EditSessionPhase::Watching);
    println!("OK edit session reuse + coalesce confirm");

    let _ = std::fs::remove_dir_all(&tmp_root);
    println!("OK cleanup temps locales");
    println!("=== PASS (sin mutaciones al lab SSH) ===");
}
