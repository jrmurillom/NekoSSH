use serde::{Deserialize, Serialize};
use rusqlite::Connection;
use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_sql::{Migration, MigrationKind};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::io::{Read, Write};
use ssh2::Session;

// --- State management for SSH Connections ---
pub struct SshConnections(pub Arc<Mutex<HashMap<String, Arc<Mutex<ssh2::Channel>>>>>);

// --- Models ---
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionProfile {
    id: Option<i64>,
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
struct SshStdoutPayload {
    terminal_id: String,
    data: String,
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
    Ok(())
}

fn get_db_conn(app: &AppHandle) -> Result<Connection, String> {
    let app_config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_config_dir).map_err(|e| e.to_string())?;
    let db_path = app_config_dir.join("nekossh.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    init_schema(&conn)?;
    Ok(conn)
}

fn list_profiles_from_db(conn: &Connection) -> Result<Vec<ConnectionProfile>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.name, p.host, p.port, p.username, p.keepalive, 
                    c.auth_type, c.password, c.key_path, c.passphrase,
                    t.tunnel_type, t.local_port, t.dest
             FROM profiles p
             LEFT JOIN auth_credentials c ON p.id = c.profile_id
             LEFT JOIN ssh_tunnels t ON p.id = t.profile_id",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ConnectionProfile {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                port: row.get(3)?,
                username: row.get(4)?,
                keepalive: row.get(5)?,
                auth_type: row.get(6).unwrap_or_else(|_| "password".to_string()),
                password: row.get(7)?,
                key_path: row.get(8)?,
                passphrase: row.get(9)?,
                tunnel_type: row.get(10).unwrap_or_else(|_| "none".to_string()),
                tunnel_local_port: row.get(11)?,
                tunnel_dest: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut profiles = Vec::new();
    for row in rows {
        profiles.push(row.map_err(|e| e.to_string())?);
    }
    Ok(profiles)
}

fn create_profile_in_db(conn: &mut Connection, profile: &ConnectionProfile) -> Result<i64, String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO profiles (name, host, port, username, keepalive) VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &profile.name,
            &profile.host,
            profile.port,
            &profile.username,
            profile.keepalive,
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
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE profiles SET name = ?1, host = ?2, port = ?3, username = ?4, keepalive = ?5 WHERE id = ?6",
        (
            &profile.name,
            &profile.host,
            profile.port,
            &profile.username,
            profile.keepalive,
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
    state: tauri::State<'_, SshConnections>,
) -> Result<(), String> {
    let state_clone = state.0.clone();
    let term_id_clone = terminal_id.clone();
    let app_handle = app.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let app_handle_err = app_handle.clone();
        let term_id_err = term_id_clone.clone();

        let run_result = move || -> Result<(), String> {
            let tcp = TcpStream::connect(format!("{}:{}", host, port))
                .map_err(|e| format!("Error de conexión TCP: {}", e))?;

            let mut sess = Session::new()
                .map_err(|e| format!("Error al crear sesión SSH: {}", e))?;
            sess.set_tcp_stream(tcp);
            sess.handshake()
                .map_err(|e| format!("Error en SSH Handshake: {}", e))?;

            if auth_type == "password" {
                let pwd = password.as_deref().unwrap_or("");
                sess.userauth_password(&username, pwd)
                    .map_err(|e| format!("Error de autenticación por contraseña: {}", e))?;
            } else {
                let kp = key_path.as_deref().ok_or("Ruta de llave privada no provista")?;
                let key_file = std::path::Path::new(kp);
                sess.userauth_pubkey_file(&username, None, key_file, passphrase.as_deref())
                    .map_err(|e| format!("Error de autenticación por llave privada: {}", e))?;
            }

            let mut channel = sess.channel_session()
                .map_err(|e| format!("Error al abrir canal SSH: {}", e))?;
            
            // Request standard PTY
            channel.request_pty("xterm-256color", None, None)
                .map_err(|e| format!("Error al solicitar PTY: {}", e))?;
            
            channel.shell()
                .map_err(|e| format!("Error al iniciar shell: {}", e))?;

            // Set session to non-blocking
            sess.set_blocking(false);

            let channel_arc = Arc::new(Mutex::new(channel));

            // Store connection in global map
            {
                let mut conns = state_clone.lock().unwrap();
                conns.insert(term_id_clone.clone(), channel_arc.clone());
            }

            // Emit a "connected" event
            let _ = app_handle.emit("ssh-connected", SshStdoutPayload {
                terminal_id: term_id_clone.clone(),
                data: "".to_string(),
            });

            // Start reading loop
            let app_handle_read = app_handle.clone();
            let term_id_read = term_id_clone.clone();
            let state_read = state_clone.clone();
            
            std::thread::spawn(move || {
                let mut buf = [0; 4096];
                loop {
                    let read_res = {
                        let conns = state_read.lock().unwrap();
                        if let Some(ch_arc) = conns.get(&term_id_read) {
                            let mut ch = ch_arc.lock().unwrap();
                            ch.read(&mut buf)
                        } else {
                            break; // removed from outside
                        }
                    };

                    match read_res {
                        Ok(0) => {
                            break; // EOF
                        }
                        Ok(n) => {
                            let output_str = String::from_utf8_lossy(&buf[..n]).into_owned();
                            let _ = app_handle_read.emit("ssh-stdout", SshStdoutPayload {
                                terminal_id: term_id_read.clone(),
                                data: output_str,
                            });
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::WouldBlock {
                                std::thread::sleep(std::time::Duration::from_millis(10));
                            } else {
                                let _ = app_handle_read.emit("ssh-closed", SshClosedPayload {
                                    terminal_id: term_id_read.clone(),
                                    error: Some(e.to_string()),
                                });
                                break;
                            }
                        }
                    }
                }

                // Cleanup session from map
                {
                    let mut conns = state_read.lock().unwrap();
                    conns.remove(&term_id_read);
                }

                // Emit closed event
                let _ = app_handle_read.emit("ssh-closed", SshClosedPayload {
                    terminal_id: term_id_read.clone(),
                    error: None,
                });
            });

            Ok(())
        };

        if let Err(e) = run_result() {
            let _ = app_handle_err.emit("ssh-error", SshClosedPayload {
                terminal_id: term_id_err,
                error: Some(e),
            });
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
    let connections = state.0.lock().unwrap();
    if let Some(channel_arc) = connections.get(&terminal_id) {
        let mut channel = channel_arc.lock().unwrap();
        let bytes = data.as_bytes();
        let mut written = 0;
        while written < bytes.len() {
            match channel.write(&bytes[written..]) {
                Ok(n) => {
                    written += n;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    } else {
                        return Err(e.to_string());
                    }
                }
            }
        }
        let _ = channel.flush();
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
    let connections = state.0.lock().unwrap();
    if let Some(channel_arc) = connections.get(&terminal_id) {
        let mut channel = channel_arc.lock().unwrap();
        channel.request_pty_size(cols, rows, None, None).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn close_ssh_session(
    terminal_id: String,
    state: tauri::State<'_, SshConnections>,
) -> Result<(), String> {
    let mut connections = state.0.lock().unwrap();
    if let Some(channel_arc) = connections.remove(&terminal_id) {
        let mut channel = channel_arc.lock().unwrap();
        let _ = channel.close();
    }
    Ok(())
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
        }
    ];

    tauri::Builder::default()
        .manage(SshConnections(Arc::new(Mutex::new(HashMap::new()))))
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:nekossh.db", migrations)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            greet,
            get_profiles,
            create_profile,
            update_profile,
            delete_profile,
            start_ssh_session,
            write_ssh_input,
            resize_ssh_pty,
            close_ssh_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_profile(name: &str) -> ConnectionProfile {
        ConnectionProfile {
            id: None,
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

        delete_profile_in_db(&conn, id).expect("eliminar");
        let after = list_profiles_from_db(&conn).expect("listar vacío");
        assert!(after.is_empty());
    }

    #[test]
    fn actualiza_perfil_y_credenciales() {
        let mut conn = open_test_db();
        let id = create_profile_in_db(&mut conn, &sample_profile("old")).expect("crear");

        let mut updated = sample_profile("new-name");
        updated.id = Some(id);
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
}
