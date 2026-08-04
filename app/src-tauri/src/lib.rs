use serde::{Deserialize, Serialize};
use rusqlite::Connection;
use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_sql::{Migration, MigrationKind};
use std::net::TcpStream;
use std::path::Path as FsPath;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::{Read, Write};
use ssh2::{MethodType, Session};

mod path_util;
mod osc7; // tests unitarios del parser; sync de producto fuera de alcance
pub mod preferences;
pub mod snippets;
pub mod edit_util;
pub mod edit_session;
pub mod fake_sftp;
pub mod elevated_upload;
mod external_edit;

use path_util::{join_remote_path, shell_quote};
use edit_session::SharedEditSessions;
use external_edit::{
    confirm_edit_upload, disconnect_edit_sessions_for_terminal, dismiss_edit_change,
    edit_session_upload_with_sudo, get_preferred_external_editor_cmd, manage_edit_state,
    probe_external_edit, set_preferred_external_editor_cmd, sftp_download_file, sftp_upload_file,
    start_external_edit, stop_external_edit, sweep_orphans_on_startup, EditWatchers,
    sftp_copy_between_sessions,
};
use preferences::ensure_app_preferences_schema;

// --- State: UNA Session SSH por terminal (PTY + SFTP como canales) ---
/// Session compartida: channel PTY + sftp() como canal aparte.
/// Nunca llamar set_blocking(true) aquí: corrompe el PTY no bloqueante.
pub struct LiveSsh {
    pub session: Session,
    pub channel: ssh2::Channel,
}

pub struct SshConnections(pub Arc<Mutex<HashMap<String, Arc<Mutex<LiveSsh>>>>>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SftpDirEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
}

// --- Models ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionFolder {
    id: Option<i64>,
    name: String,
    sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionProfile {
    id: Option<i64>,
    folder_id: Option<i64>,
    name: String,
    host: String,
    port: u16,
    username: String,
    auth_type: String, // "password" or "key"
    password: Option<String>,
    key_path: Option<String>,
    passphrase: Option<String>,
    keepalive: u32,
    tunnel_type: String, // "none", "local", "dynamic"
    tunnel_local_port: Option<u16>,
    tunnel_dest: Option<String>,
}

#[derive(Clone, Serialize)]
pub(crate) struct SshStdoutPayload {
    pub(crate) terminal_id: String,
    pub(crate) data: String,
}

#[derive(Clone, Serialize)]
struct SshClosedPayload {
    terminal_id: String,
    error: Option<String>,
}

// --- Helpers ---
fn init_schema(conn: &Connection) -> Result<(), String> {
    conn.execute("PRAGMA foreign_keys = ON;", [])
        .map_err(|e| e.to_string())?;
    conn.execute_batch(include_str!("../migrations/001_initial_schema.sql"))
        .map_err(|e| format!("Error al inicializar base de datos: {}", e))?;
    ensure_connection_folders_schema(conn)?;
    ensure_app_preferences_schema(conn)?;
    snippets::ensure_snippets_schema(conn)?;
    Ok(())
}

/// Idempotent folders schema for rusqlite path + in-memory tests (plugin migration 002 runs once).
fn ensure_connection_folders_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS connection_folders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_connection_folders_sort ON connection_folders(sort_order);
        INSERT INTO connection_folders (name, sort_order)
        SELECT 'General', 0
        WHERE NOT EXISTS (SELECT 1 FROM connection_folders WHERE name = 'General');
        "#,
    )
    .map_err(|e| format!("Error al crear connection_folders: {}", e))?;

    let has_folder_id = {
        let mut stmt = conn
            .prepare("PRAGMA table_info(profiles)")
            .map_err(|e| e.to_string())?;
        let cols = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| e.to_string())?;
        let mut found = false;
        for col in cols {
            if col.map_err(|e| e.to_string())? == "folder_id" {
                found = true;
                break;
            }
        }
        found
    };

    if !has_folder_id {
        conn.execute(
            "ALTER TABLE profiles ADD COLUMN folder_id INTEGER REFERENCES connection_folders(id) ON DELETE CASCADE",
            [],
        )
        .map_err(|e| format!("Error al añadir folder_id: {}", e))?;
    }

    conn.execute(
        "UPDATE profiles SET folder_id = (SELECT id FROM connection_folders WHERE name = 'General' LIMIT 1) WHERE folder_id IS NULL",
        [],
    )
    .map_err(|e| format!("Error al backfill folder_id: {}", e))?;

    Ok(())
}

fn default_folder_id(conn: &Connection) -> Result<i64, String> {
    conn.query_row(
        "SELECT id FROM connection_folders WHERE name = 'General' ORDER BY id LIMIT 1",
        [],
        |row| row.get(0),
    )
    .map_err(|e| format!("Carpeta General no encontrada: {}", e))
}

pub(crate) fn get_db_conn(app: &AppHandle) -> Result<Connection, String> {
    let app_config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_config_dir).map_err(|e| e.to_string())?;
    let db_path = app_config_dir.join("nekossh.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    init_schema(&conn)?;
    Ok(conn)
}

fn list_folders_from_db(conn: &Connection) -> Result<Vec<ConnectionFolder>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, sort_order FROM connection_folders ORDER BY sort_order ASC, id ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ConnectionFolder {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut folders = Vec::new();
    for row in rows {
        folders.push(row.map_err(|e| e.to_string())?);
    }
    Ok(folders)
}

fn create_folder_in_db(conn: &Connection, name: &str, sort_order: i32) -> Result<i64, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("El nombre de carpeta no puede estar vacío".to_string());
    }
    conn.execute(
        "INSERT INTO connection_folders (name, sort_order) VALUES (?1, ?2)",
        (trimmed, sort_order),
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

fn update_folder_in_db(conn: &Connection, id: i64, name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("El nombre de carpeta no puede estar vacío".to_string());
    }
    let updated = conn
        .execute(
            "UPDATE connection_folders SET name = ?1 WHERE id = ?2",
            (trimmed, id),
        )
        .map_err(|e| e.to_string())?;
    if updated == 0 {
        return Err(format!("Carpeta {} no encontrada", id));
    }
    Ok(())
}

fn delete_folder_in_db(conn: &Connection, id: i64) -> Result<(), String> {
    // Explicit cascade: profiles (and their credentials/tunnels via profile FKs).
    conn.execute("DELETE FROM profiles WHERE folder_id = ?1", [id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM connection_folders WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn count_profiles_in_folder(conn: &Connection, folder_id: i64) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM profiles WHERE folder_id = ?1",
        [folder_id],
        |row| row.get(0),
    )
    .map_err(|e| e.to_string())
}

fn list_profiles_from_db(conn: &Connection) -> Result<Vec<ConnectionProfile>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.folder_id, p.name, p.host, p.port, p.username, p.keepalive, 
                    c.auth_type, c.password, c.key_path, c.passphrase,
                    t.tunnel_type, t.local_port, t.dest
             FROM profiles p
             LEFT JOIN auth_credentials c ON p.id = c.profile_id
             LEFT JOIN ssh_tunnels t ON p.id = t.profile_id
             ORDER BY p.folder_id ASC, p.name ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ConnectionProfile {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                name: row.get(2)?,
                host: row.get(3)?,
                port: row.get(4)?,
                username: row.get(5)?,
                keepalive: row.get(6)?,
                auth_type: row.get(7).unwrap_or_else(|_| "password".to_string()),
                password: row.get(8)?,
                key_path: row.get(9)?,
                passphrase: row.get(10)?,
                tunnel_type: row.get(11).unwrap_or_else(|_| "none".to_string()),
                tunnel_local_port: row.get(12)?,
                tunnel_dest: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut profiles = Vec::new();
    for row in rows {
        profiles.push(row.map_err(|e| e.to_string())?);
    }
    Ok(profiles)
}

fn resolve_folder_id(conn: &Connection, folder_id: Option<i64>) -> Result<i64, String> {
    match folder_id {
        Some(id) => {
            let exists: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM connection_folders WHERE id = ?1",
                    [id],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            if exists == 0 {
                return Err(format!("Carpeta {} no existe", id));
            }
            Ok(id)
        }
        None => default_folder_id(conn),
    }
}

fn create_profile_in_db(conn: &mut Connection, profile: &ConnectionProfile) -> Result<i64, String> {
    let folder_id = resolve_folder_id(conn, profile.folder_id)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO profiles (name, host, port, username, keepalive, folder_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &profile.name,
            &profile.host,
            profile.port,
            &profile.username,
            profile.keepalive,
            folder_id,
        ),
    )
    .map_err(|e| e.to_string())?;

    let profile_id = tx.last_insert_rowid();

    tx.execute(
        "INSERT INTO auth_credentials (profile_id, auth_type, password, key_path, passphrase) 
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            profile_id,
            &profile.auth_type,
            &profile.password,
            &profile.key_path,
            &profile.passphrase,
        ),
    )
    .map_err(|e| e.to_string())?;

    if profile.tunnel_type != "none" {
        tx.execute(
            "INSERT INTO ssh_tunnels (profile_id, tunnel_type, local_port, dest) 
             VALUES (?1, ?2, ?3, ?4)",
            (
                profile_id,
                &profile.tunnel_type,
                profile.tunnel_local_port,
                &profile.tunnel_dest,
            ),
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(profile_id)
}

fn update_profile_in_db(conn: &mut Connection, profile: &ConnectionProfile) -> Result<(), String> {
    let profile_id = profile
        .id
        .ok_or_else(|| "Missing profile ID for update".to_string())?;
    let folder_id = resolve_folder_id(conn, profile.folder_id)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE profiles SET name = ?1, host = ?2, port = ?3, username = ?4, keepalive = ?5, folder_id = ?6 WHERE id = ?7",
        (
            &profile.name,
            &profile.host,
            profile.port,
            &profile.username,
            profile.keepalive,
            folder_id,
            profile_id,
        ),
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT OR REPLACE INTO auth_credentials (profile_id, auth_type, password, key_path, passphrase) 
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            profile_id,
            &profile.auth_type,
            &profile.password,
            &profile.key_path,
            &profile.passphrase,
        ),
    )
    .map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM ssh_tunnels WHERE profile_id = ?1", [profile_id])
        .map_err(|e| e.to_string())?;

    if profile.tunnel_type != "none" {
        tx.execute(
            "INSERT INTO ssh_tunnels (profile_id, tunnel_type, local_port, dest) 
             VALUES (?1, ?2, ?3, ?4)",
            (
                profile_id,
                &profile.tunnel_type,
                profile.tunnel_local_port,
                &profile.tunnel_dest,
            ),
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

fn delete_profile_in_db(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM profiles WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// --- Commands ---
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn list_folders(app: AppHandle) -> Result<Vec<ConnectionFolder>, String> {
    let conn = get_db_conn(&app)?;
    list_folders_from_db(&conn)
}

#[tauri::command]
async fn create_folder(app: AppHandle, name: String, sort_order: Option<i32>) -> Result<i64, String> {
    let conn = get_db_conn(&app)?;
    create_folder_in_db(&conn, &name, sort_order.unwrap_or(0))
}

#[tauri::command]
async fn update_folder(app: AppHandle, id: i64, name: String) -> Result<(), String> {
    let conn = get_db_conn(&app)?;
    update_folder_in_db(&conn, id, &name)
}

#[tauri::command]
async fn delete_folder(app: AppHandle, id: i64) -> Result<(), String> {
    let conn = get_db_conn(&app)?;
    delete_folder_in_db(&conn, id)
}

#[tauri::command]
async fn get_folder_connection_count(app: AppHandle, id: i64) -> Result<i64, String> {
    let conn = get_db_conn(&app)?;
    count_profiles_in_folder(&conn, id)
}

#[tauri::command]
async fn get_profiles(app: AppHandle) -> Result<Vec<ConnectionProfile>, String> {
    let conn = get_db_conn(&app)?;
    list_profiles_from_db(&conn)
}

#[tauri::command]
async fn create_profile(app: AppHandle, profile: ConnectionProfile) -> Result<i64, String> {
    let mut conn = get_db_conn(&app)?;
    create_profile_in_db(&mut conn, &profile)
}

#[tauri::command]
async fn update_profile(app: AppHandle, profile: ConnectionProfile) -> Result<(), String> {
    let mut conn = get_db_conn(&app)?;
    update_profile_in_db(&mut conn, &profile)
}

#[tauri::command]
async fn delete_profile(app: AppHandle, id: i64) -> Result<(), String> {
    let conn = get_db_conn(&app)?;
    delete_profile_in_db(&conn, id)
}

// --- SSH / SFTP helpers ---
fn authenticate_session(
    host: &str,
    port: u16,
    username: &str,
    auth_type: &str,
    password: Option<&str>,
    key_path: Option<&str>,
    passphrase: Option<&str>,
    keepalive_secs: u32,
) -> Result<Session, String> {
    // Reintentos: en Windows/WinCNG (y bajo MaxStartups) el KEX a veces falla con Session(-8).
    let mut last_err = String::from("Error de handshake SSH desconocido");
    for attempt in 1..=3 {
        match authenticate_session_once(
            host,
            port,
            username,
            auth_type,
            password,
            key_path,
            passphrase,
            keepalive_secs,
        ) {
            Ok(sess) => return Ok(sess),
            Err(e) => {
                last_err = e.clone();
                let kex_fail = e.contains("Session(-8)")
                    || e.to_lowercase().contains("exchange encryption keys");
                if !kex_fail || attempt == 3 {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(250 * attempt as u64));
            }
        }
    }
    Err(last_err)
}

fn authenticate_session_once(
    host: &str,
    port: u16,
    username: &str,
    auth_type: &str,
    password: Option<&str>,
    key_path: Option<&str>,
    passphrase: Option<&str>,
    keepalive_secs: u32,
) -> Result<Session, String> {
    use std::net::ToSocketAddrs;

    // connect_timeout: NO dejar SO_RCVTIMEO en el socket (eso mataba el PTY con "transport read").
    let addr = (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("Error resolviendo {}:{}: {}", host, port, e))?
        .next()
        .ok_or_else(|| format!("No se pudo resolver {}:{}", host, port))?;
    let tcp = TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(15))
        .map_err(|e| format!("Error de conexión TCP: {}", e))?;
    let _ = tcp.set_nodelay(true);
    // Explicitamente sin timeout de lectura/escritura en el socket viviente.
    let _ = tcp.set_read_timeout(None);
    let _ = tcp.set_write_timeout(None);

    let mut sess = Session::new().map_err(|e| format!("Error al crear sesión SSH: {}", e))?;
    // Timeout solo para el handshake/auth; se apaga al terminar.
    sess.set_timeout(20_000);

    // Preferir KEX moderno y evitar dh-gex (falla frecuente con backends Windows).
    let _ = sess.method_pref(
        MethodType::Kex,
        "curve25519-sha256,curve25519-sha256@libssh.org,ecdh-sha2-nistp256,ecdh-sha2-nistp384,ecdh-sha2-nistp521,diffie-hellman-group14-sha256,diffie-hellman-group16-sha512,diffie-hellman-group14-sha1",
    );
    let _ = sess.method_pref(
        MethodType::CryptCs,
        "aes256-ctr,aes192-ctr,aes128-ctr,chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com",
    );
    let _ = sess.method_pref(
        MethodType::CryptSc,
        "aes256-ctr,aes192-ctr,aes128-ctr,chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com",
    );
    let _ = sess.method_pref(
        MethodType::HostKey,
        "ssh-ed25519,ecdsa-sha2-nistp256,ecdsa-sha2-nistp384,ecdsa-sha2-nistp521,rsa-sha2-512,rsa-sha2-256,ssh-rsa",
    );

    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("Error en SSH Handshake: {}", e))?;

    // Keepalive SSH (libssh2): segundos entre paquetes; 0 = desactivado.
    let ka = if keepalive_secs == 0 { 60 } else { keepalive_secs };
    sess.set_keepalive(true, ka);

    if auth_type == "password" {
        let pwd = password.unwrap_or("");
        sess.userauth_password(username, pwd)
            .map_err(|e| format!("Error de autenticación por contraseña: {}", e))?;
    } else {
        let kp = key_path.ok_or_else(|| "Ruta de llave privada no provista".to_string())?;
        let key_file = FsPath::new(kp);
        sess.userauth_pubkey_file(username, None, key_file, passphrase)
            .map_err(|e| format!("Error de autenticación por llave privada: {}", e))?;
    }

    if !sess.authenticated() {
        return Err("Autenticación SSH fallida".to_string());
    }

    // Sesión interactiva: sin timeout libssh2 (0 = disabled).
    sess.set_timeout(0);
    Ok(sess)
}

// --- SSH Commands ---
#[tauri::command]
async fn start_ssh_session(
    app: AppHandle,
    terminal_id: String,
    host: String,
    port: u16,
    username: String,
    auth_type: String,
    password: Option<String>,
    key_path: Option<String>,
    passphrase: Option<String>,
    keepalive: Option<u32>,
    state: tauri::State<'_, SshConnections>,
) -> Result<(), String> {
    let state_clone = state.0.clone();
    let term_id_clone = terminal_id.clone();
    let app_handle = app.clone();
    let keepalive_secs = keepalive.unwrap_or(60);

    tauri::async_runtime::spawn_blocking(move || {
        let app_handle_err = app_handle.clone();
        let term_id_err = term_id_clone.clone();

        let run_result = move || -> Result<(), String> {
            // Cerrar sesión previa de este terminal_id (reconexión limpia).
            {
                let mut conns = state_clone.lock().unwrap();
                remove_and_shutdown_ssh(&mut conns, &term_id_clone);
            }

            // UNA conexión TCP/SSH: PTY + SFTP (subsystem) como canales.
            // Prohibido un 2º login (en muchos VPS tumba la sesión interactiva).
            let sess = authenticate_session(
                &host,
                port,
                &username,
                &auth_type,
                password.as_deref(),
                key_path.as_deref(),
                passphrase.as_deref(),
                keepalive_secs,
            )?;

            let mut channel = sess
                .channel_session()
                .map_err(|e| format!("Error al abrir canal SSH: {}", e))?;

            channel
                .request_pty("xterm-256color", None, None)
                .map_err(|e| format!("Error al solicitar PTY: {}", e))?;

            channel
                .shell()
                .map_err(|e| format!("Error al iniciar shell: {}", e))?;

            // No bloqueante para el PTY; SFTP usará la misma Session sin cambiar este modo.
            sess.set_blocking(false);
            // Por si acaso: timeout 0 en sesión interactiva (evita "transport read" falso).
            sess.set_timeout(0);

            let live = Arc::new(Mutex::new(LiveSsh {
                session: sess,
                channel,
            }));

            {
                let mut conns = state_clone.lock().unwrap();
                conns.insert(term_id_clone.clone(), live.clone());
            }

            let _ = app_handle.emit(
                "ssh-connected",
                SshStdoutPayload {
                    terminal_id: term_id_clone.clone(),
                    data: "".to_string(),
                },
            );

            let app_handle_read = app_handle.clone();
            let term_id_read = term_id_clone.clone();
            let state_read = state_clone.clone();

            std::thread::spawn(move || {
                let mut buf = [0; 4096];
                let mut close_reason: Option<String> = None;
                let mut last_ka = std::time::Instant::now();
                // Intervalo de keepalive SSH (libssh2). Errores soft no cierran la sesión:
                // el cierre real lo decide channel.read (EOF / transport).
                let ka_interval =
                    std::time::Duration::from_secs((keepalive_secs / 2).clamp(10, 30) as u64);
                loop {
                    let live_arc = {
                        let conns = state_read.lock().unwrap();
                        conns.get(&term_id_read).cloned()
                    };
                    let Some(live_arc) = live_arc else {
                        break;
                    };

                    let read_res = {
                        let mut live = live_arc.lock().unwrap();
                        if last_ka.elapsed() >= ka_interval {
                            // want_reply ya configurado en set_keepalive; ignore WouldBlock/soft.
                            let _ = live.session.keepalive_send();
                            last_ka = std::time::Instant::now();
                        }
                        live.channel.read(&mut buf)
                    };

                    match read_res {
                        Ok(0) => {
                            close_reason = Some("EOF del canal PTY (Ok(0))".to_string());
                            break;
                        }
                        Ok(n) => {
                            let output_str = String::from_utf8_lossy(&buf[..n]).into_owned();
                            let _ = app_handle_read.emit(
                                "ssh-stdout",
                                SshStdoutPayload {
                                    terminal_id: term_id_read.clone(),
                                    data: output_str,
                                },
                            );
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::WouldBlock
                                || e.kind() == std::io::ErrorKind::TimedOut
                            {
                                // TimedOut no debería ocurrir con set_timeout(0) + sin SO_RCVTIMEO.
                                // Si aparece, no matar la sesión (evita falsos "transport read").
                                std::thread::sleep(std::time::Duration::from_millis(10));
                            } else {
                                close_reason = Some(format!("error de lectura PTY: {}", e));
                                break;
                            }
                        }
                    }
                }

                // Salida del loop: si aún estábamos en el mapa, fue EOF/error remoto → emitir.
                // Si `close_ssh_session` ya nos quitó, no emitir (evita banner amarillo al cerrar pestaña).
                let still_tracked = {
                    let mut conns = state_read.lock().unwrap();
                    conns.remove(&term_id_read).is_some()
                };

                if still_tracked {
                    if let (Some(edits), Some(watchers)) = (
                        app_handle_read.try_state::<SharedEditSessions>(),
                        app_handle_read.try_state::<EditWatchers>(),
                    ) {
                        disconnect_edit_sessions_for_terminal(
                            &app_handle_read,
                            &term_id_read,
                            &edits,
                            &watchers,
                            true,
                        );
                    }
                    let _ = app_handle_read.emit(
                        "ssh-closed",
                        SshClosedPayload {
                            terminal_id: term_id_read.clone(),
                            error: close_reason,
                        },
                    );
                }
            });

            Ok(())
        };

        if let Err(e) = run_result() {
            let _ = app_handle_err.emit(
                "ssh-error",
                SshClosedPayload {
                    terminal_id: term_id_err,
                    error: Some(e),
                },
            );
        }
    });

    Ok(())
}

#[tauri::command]
async fn write_ssh_input(
    terminal_id: String,
    data: String,
    state: tauri::State<'_, SshConnections>,
) -> Result<(), String> {
    let live_arc = {
        let connections = state.0.lock().unwrap();
        connections.get(&terminal_id).cloned()
    };
    let Some(live_arc) = live_arc else {
        return Ok(());
    };

    let bytes = data.as_bytes();
    let mut written = 0;
    let mut attempts = 0;
    while written < bytes.len() {
        let write_res = {
            let mut live = live_arc.lock().unwrap();
            live.channel.write(&bytes[written..])
        };
        match write_res {
            Ok(n) => {
                written += n;
                attempts = 0;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    attempts += 1;
                    if attempts > 200 {
                        return Err("Timeout escribiendo al PTY (WouldBlock)".to_string());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                } else {
                    return Err(e.to_string());
                }
            }
        }
    }
    // Evidencia (smoke_isolate_burst): flush en CADA tecla con lector PTY concurrente
    // provoca "transport read" en libssh2/WinCNG. Solo flush en fin de línea o paste.
    let should_flush = bytes.iter().any(|b| *b == b'\n' || *b == b'\r') || bytes.len() > 1;
    if should_flush {
        let mut live = live_arc.lock().unwrap();
        let _ = live.channel.flush();
    }
    Ok(())
}

#[tauri::command]
async fn resize_ssh_pty(
    terminal_id: String,
    cols: u32,
    rows: u32,
    state: tauri::State<'_, SshConnections>,
) -> Result<(), String> {
    let live_arc = {
        let connections = state.0.lock().unwrap();
        connections.get(&terminal_id).cloned()
    };
    let Some(live_arc) = live_arc else {
        return Ok(());
    };
    let mut live = live_arc.lock().unwrap();
    live.channel
        .request_pty_size(cols, rows, None, None)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn close_ssh_session(
    app: AppHandle,
    terminal_id: String,
    state: tauri::State<'_, SshConnections>,
    edits: tauri::State<'_, SharedEditSessions>,
    watchers: tauri::State<'_, EditWatchers>,
) -> Result<(), String> {
    disconnect_edit_sessions_for_terminal(&app, &terminal_id, &edits, &watchers, true);
    let mut connections = state.0.lock().unwrap();
    remove_and_shutdown_ssh(&mut connections, &terminal_id);
    Ok(())
}

/// Cierra canal PTY + desconecta la Session. Best-effort; no propaga errores.
fn shutdown_live_ssh(live: &mut LiveSsh) {
    let _ = live.channel.close();
    let _ = live
        .session
        .disconnect(None, "NekoSSH session closed", None);
}

/// Quita `terminal_id` del mapa y apaga la Session. Idempotente si no existe.
/// Devuelve `true` si había una sesión viva.
fn remove_and_shutdown_ssh(
    connections: &mut HashMap<String, Arc<Mutex<LiveSsh>>>,
    terminal_id: &str,
) -> bool {
    let Some(live_arc) = connections.remove(terminal_id) else {
        return false;
    };
    // Evitar deadlock si el hilo PTY sostiene el mutex: try_lock + breve espera.
    for _ in 0..50 {
        match live_arc.try_lock() {
            Ok(mut live) => {
                shutdown_live_ssh(&mut live);
                return true;
            }
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(10)),
        }
    }
    // Último intento bloqueante (el reader debería haber soltado al ver remove del mapa).
    if let Ok(mut live) = live_arc.lock() {
        shutdown_live_ssh(&mut live);
    }
    true
}

/// Cierra todas las Sessions del mapa (salida de app / cerrar todas).
fn close_all_ssh_connections(connections: &mut HashMap<String, Arc<Mutex<LiveSsh>>>) {
    let ids: Vec<String> = connections.keys().cloned().collect();
    for id in ids {
        remove_and_shutdown_ssh(connections, &id);
    }
}

pub(crate) fn is_would_block_ssh(err: &ssh2::Error) -> bool {
    let msg = err.to_string();
    msg.contains("WouldBlock")
        || msg.contains("would block")
        || err.code() == ssh2::ErrorCode::Session(-37)
}

/// Lista un directorio vía SFTP en la **misma** Session del PTY (canal subsystem).
/// No usa set_blocking(true). Entre reintentos bombea el PTY y reemite stdout
/// para no perder output (libssh2 multiplex + non-blocking).
#[tauri::command]
async fn sftp_list_dir(
    app: AppHandle,
    terminal_id: String,
    path: String,
    state: tauri::State<'_, SshConnections>,
) -> Result<Vec<SftpDirEntry>, String> {
    let live_arc = {
        let map = state.0.lock().unwrap();
        map.get(&terminal_id)
            .cloned()
            .ok_or_else(|| "No hay sesión SSH para esta terminal".to_string())?
    };
    let term_id = terminal_id.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let list_path = if path.is_empty() {
            ".".to_string()
        } else {
            path
        };

        let mut pump_buf = [0u8; 4096];

        let emit_pump = |data: &[u8]| {
            if data.is_empty() {
                return;
            }
            let _ = app.emit(
                "ssh-stdout",
                SshStdoutPayload {
                    terminal_id: term_id.clone(),
                    data: String::from_utf8_lossy(data).into_owned(),
                },
            );
        };

        // Abrir SFTP: reintentar WouldBlock + bombear PTY (mismo socket SSH).
        let sftp = {
            let mut attempts = 0;
            loop {
                let open_res = {
                    let mut live = live_arc.lock().unwrap();
                    let mut pumped = Vec::new();
                    loop {
                        match live.channel.read(&mut pump_buf) {
                            Ok(0) => break,
                            Ok(n) => pumped.extend_from_slice(&pump_buf[..n]),
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                            Err(_) => break,
                        }
                    }
                    let res = live.session.sftp();
                    (res, pumped)
                };
                emit_pump(&open_res.1);
                match open_res.0 {
                    Ok(s) => break s,
                    Err(e) => {
                        if attempts < 200 && is_would_block_ssh(&e) {
                            attempts += 1;
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        }
                        return Err(format!("Error al abrir SFTP: {}", e));
                    }
                }
            }
        };

        let entries = {
            let mut attempts = 0;
            loop {
                let read_res = {
                    let mut live = live_arc.lock().unwrap();
                    let mut pumped = Vec::new();
                    loop {
                        match live.channel.read(&mut pump_buf) {
                            Ok(0) => break,
                            Ok(n) => pumped.extend_from_slice(&pump_buf[..n]),
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                            Err(_) => break,
                        }
                    }
                    let res = sftp.readdir(FsPath::new(&list_path));
                    (res, pumped)
                };
                emit_pump(&read_res.1);
                match read_res.0 {
                    Ok(e) => break e,
                    Err(e) => {
                        if attempts < 200 && is_would_block_ssh(&e) {
                            attempts += 1;
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        }
                        return Err(format!("Error al listar {}: {}", list_path, e));
                    }
                }
            }
        };
        drop(sftp);

        let mut result = Vec::new();
        for (entry_path, stat) in entries {
            let name = entry_path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| entry_path.to_string_lossy().into_owned());
            if name == "." || name == ".." {
                continue;
            }
            let full = if list_path == "." {
                join_remote_path("", &name)
            } else {
                join_remote_path(&list_path, &name)
            };
            result.push(SftpDirEntry {
                name,
                path: full,
                is_dir: stat.is_dir(),
                size: stat.size.unwrap_or(0),
            });
        }
        result.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        Ok(result)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn ssh_cd(
    terminal_id: String,
    path: String,
    state: tauri::State<'_, SshConnections>,
) -> Result<String, String> {
    let quoted = shell_quote(&path);
    let cmd = format!("cd {}\r", quoted);
    // Escribir cd en el PTY. Devolvemos `path` (el cwd del shell interactivo
    // no se obtiene con un exec "pwd" aparte: ese canal no comparte el cwd del PTY).
    {
        let live_arc = {
            let connections = state.0.lock().unwrap();
            connections.get(&terminal_id).cloned()
        };
        let Some(live_arc) = live_arc else {
            return Err("No hay sesión SSH para esta terminal".to_string());
        };
        let mut live = live_arc.lock().unwrap();
        let bytes = cmd.as_bytes();
        let mut written = 0;
        while written < bytes.len() {
            match live.channel.write(&bytes[written..]) {
                Ok(n) => written += n,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        let _ = live.channel.flush();
    }
    Ok(path)
}

#[tauri::command]
fn list_snippet_categories(app: AppHandle) -> Result<Vec<snippets::SnippetCategory>, String> {
    let conn = get_db_conn(&app)?;
    snippets::ensure_snippet_seed(&conn)?;
    snippets::list_categories(&conn)
}

#[tauri::command]
fn create_snippet_category(app: AppHandle, name: String) -> Result<snippets::SnippetCategory, String> {
    let conn = get_db_conn(&app)?;
    snippets::create_category(&conn, &name)
}

#[tauri::command]
fn delete_snippet_category(app: AppHandle, id: i64) -> Result<(), String> {
    let conn = get_db_conn(&app)?;
    snippets::delete_category(&conn, id)
}

#[tauri::command]
fn list_snippets_cmd(
    app: AppHandle,
    category_id: Option<i64>,
    query: Option<String>,
) -> Result<Vec<snippets::Snippet>, String> {
    let conn = get_db_conn(&app)?;
    snippets::ensure_snippet_seed(&conn)?;
    snippets::list_snippets(&conn, category_id, query.as_deref())
}

#[tauri::command]
fn create_snippet_cmd(
    app: AppHandle,
    category_id: i64,
    title: String,
    body: String,
) -> Result<snippets::Snippet, String> {
    let conn = get_db_conn(&app)?;
    snippets::create_snippet(&conn, category_id, &title, &body)
}

#[tauri::command]
fn update_snippet_cmd(
    app: AppHandle,
    id: i64,
    category_id: i64,
    title: String,
    body: String,
) -> Result<snippets::Snippet, String> {
    let conn = get_db_conn(&app)?;
    snippets::update_snippet(&conn, id, category_id, &title, &body)
}

#[tauri::command]
fn delete_snippet_cmd(app: AppHandle, id: i64) -> Result<(), String> {
    let conn = get_db_conn(&app)?;
    snippets::delete_snippet(&conn, id)
}

#[tauri::command]
fn ensure_snippet_seed_cmd(app: AppHandle) -> Result<bool, String> {
    let conn = get_db_conn(&app)?;
    snippets::ensure_snippet_seed(&conn)
}

// --- App Entry Point ---
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "initial_schema",
            sql: include_str!("../migrations/001_initial_schema.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "connection_folders",
            sql: include_str!("../migrations/002_connection_folders.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "app_preferences",
            sql: include_str!("../migrations/003_app_preferences.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 4,
            description: "snippets",
            sql: include_str!("../migrations/004_snippets.sql"),
            kind: MigrationKind::Up,
        },
    ];

    let (edit_sessions, edit_watchers) = manage_edit_state();

    let app = tauri::Builder::default()
        .manage(SshConnections(Arc::new(Mutex::new(HashMap::new()))))
        .manage(edit_sessions)
        .manage(edit_watchers)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:nekossh.db", migrations)
                .build(),
        )
        .setup(|app| {
            sweep_orphans_on_startup(&app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            list_folders,
            create_folder,
            update_folder,
            delete_folder,
            get_folder_connection_count,
            get_profiles,
            create_profile,
            update_profile,
            delete_profile,
            start_ssh_session,
            write_ssh_input,
            resize_ssh_pty,
            close_ssh_session,
            sftp_list_dir,
            ssh_cd,
            get_preferred_external_editor_cmd,
            set_preferred_external_editor_cmd,
            probe_external_edit,
            sftp_download_file,
            sftp_upload_file,
            start_external_edit,
            confirm_edit_upload,
            edit_session_upload_with_sudo,
            dismiss_edit_change,
            stop_external_edit,
            sftp_copy_between_sessions,
            list_snippet_categories,
            create_snippet_category,
            delete_snippet_category,
            list_snippets_cmd,
            create_snippet_cmd,
            update_snippet_cmd,
            delete_snippet_cmd,
            ensure_snippet_seed_cmd
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        // Al salir o al pedir salida: cerrar todas las Sessions SSH vivas.
        match event {
            tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
                if let Some(state) = app_handle.try_state::<SshConnections>() {
                    let ids: Vec<String> = {
                        let conns = state.0.lock().unwrap();
                        conns.keys().cloned().collect()
                    };
                    if let (Some(edits), Some(watchers)) = (
                        app_handle.try_state::<SharedEditSessions>(),
                        app_handle.try_state::<EditWatchers>(),
                    ) {
                        for id in &ids {
                            disconnect_edit_sessions_for_terminal(
                                app_handle, id, &edits, &watchers, false,
                            );
                        }
                    }
                    let mut conns = state.0.lock().unwrap();
                    close_all_ssh_connections(&mut conns);
                }
            }
            _ => {}
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_profile(name: &str) -> ConnectionProfile {
        ConnectionProfile {
            id: None,
            folder_id: None,
            name: name.to_string(),
            host: "192.168.1.10".to_string(),
            port: 22,
            username: "neko".to_string(),
            auth_type: "password".to_string(),
            password: Some("secret".to_string()),
            key_path: None,
            passphrase: None,
            keepalive: 60,
            tunnel_type: "none".to_string(),
            tunnel_local_port: None,
            tunnel_dest: None,
        }
    }

    fn open_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("abrir sqlite en memoria");
        init_schema(&conn).expect("inicializar esquema");
        conn
    }

    #[test]
    fn crea_lista_y_elimina_perfil() {
        let mut conn = open_test_db();
        let id = create_profile_in_db(&mut conn, &sample_profile("lab")).expect("crear");
        assert!(id > 0);

        let profiles = list_profiles_from_db(&conn).expect("listar");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "lab");
        assert_eq!(profiles[0].auth_type, "password");
        assert_eq!(profiles[0].password.as_deref(), Some("secret"));
        assert!(profiles[0].folder_id.is_some());

        delete_profile_in_db(&conn, id).expect("eliminar");
        let after = list_profiles_from_db(&conn).expect("listar vacío");
        assert!(after.is_empty());
    }

    #[test]
    fn actualiza_perfil_y_credenciales() {
        let mut conn = open_test_db();
        let id = create_profile_in_db(&mut conn, &sample_profile("old")).expect("crear");
        let general_id = default_folder_id(&conn).expect("general");

        let mut updated = sample_profile("new-name");
        updated.id = Some(id);
        updated.folder_id = Some(general_id);
        updated.host = "10.0.0.5".to_string();
        updated.auth_type = "key".to_string();
        updated.password = None;
        updated.key_path = Some("/keys/id_ed25519".to_string());
        updated.passphrase = Some("frase".to_string());
        updated.tunnel_type = "dynamic".to_string();
        updated.tunnel_local_port = Some(1080);
        updated.tunnel_dest = None;

        update_profile_in_db(&mut conn, &updated).expect("actualizar");

        let profiles = list_profiles_from_db(&conn).expect("listar");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "new-name");
        assert_eq!(profiles[0].host, "10.0.0.5");
        assert_eq!(profiles[0].auth_type, "key");
        assert_eq!(profiles[0].key_path.as_deref(), Some("/keys/id_ed25519"));
        assert_eq!(profiles[0].tunnel_type, "dynamic");
        assert_eq!(profiles[0].tunnel_local_port, Some(1080));
        assert_eq!(profiles[0].folder_id, Some(general_id));
    }

    #[test]
    fn eliminar_perfil_cascada_credenciales() {
        let mut conn = open_test_db();
        let id = create_profile_in_db(&mut conn, &sample_profile("cascade")).expect("crear");
        delete_profile_in_db(&conn, id).expect("eliminar");

        let cred_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM auth_credentials WHERE profile_id = ?1",
                [id],
                |row| row.get(0),
            )
            .expect("contar credenciales");
        assert_eq!(cred_count, 0);
    }

    #[test]
    fn carpeta_crud_y_conexion_en_carpeta() {
        let mut conn = open_test_db();
        let folders = list_folders_from_db(&conn).expect("listar folders");
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].name, "General");

        let prod_id = create_folder_in_db(&conn, "Production", 1).expect("crear folder");
        update_folder_in_db(&conn, prod_id, "Prod").expect("rename");

        let mut profile = sample_profile("vps");
        profile.folder_id = Some(prod_id);
        let pid = create_profile_in_db(&mut conn, &profile).expect("crear en folder");
        let listed = list_profiles_from_db(&conn).expect("listar");
        assert_eq!(listed[0].folder_id, Some(prod_id));
        assert_eq!(count_profiles_in_folder(&conn, prod_id).unwrap(), 1);

        delete_folder_in_db(&conn, prod_id).expect("cascade delete folder");
        assert!(list_profiles_from_db(&conn).unwrap().is_empty());
        let _ = pid;
        // Idempotent delete
        delete_folder_in_db(&conn, prod_id).expect("idempotent");
    }

    #[test]
    fn migracion_backfill_profiles_sin_folder() {
        let conn = Connection::open_in_memory().expect("mem");
        conn.execute_batch(include_str!("../migrations/001_initial_schema.sql"))
            .expect("001");
        conn.execute(
            "INSERT INTO profiles (name, host, port, username, keepalive) VALUES ('legacy', '1.1.1.1', 22, 'u', 60)",
            [],
        )
        .expect("legacy profile");
        ensure_connection_folders_schema(&conn).expect("folders schema");
        let profiles = list_profiles_from_db(&conn).expect("list");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "legacy");
        assert!(profiles[0].folder_id.is_some());
    }

    #[test]
    fn close_all_ssh_on_empty_map_is_noop() {
        let mut map: HashMap<String, Arc<Mutex<LiveSsh>>> = HashMap::new();
        close_all_ssh_connections(&mut map);
        assert!(map.is_empty());
    }

    #[test]
    fn remove_missing_session_is_idempotent() {
        let mut map: HashMap<String, Arc<Mutex<LiveSsh>>> = HashMap::new();
        assert!(!remove_and_shutdown_ssh(&mut map, "term-missing"));
        assert!(map.is_empty());
        // Segunda llamada sigue siendo no-op
        assert!(!remove_and_shutdown_ssh(&mut map, "term-missing"));
    }
}
