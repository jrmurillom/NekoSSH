//! Commands Tauri: probe/download/upload, start edit session, watcher, editor open.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;

use crate::edit_session::{
    new_shared_registry, EditSessionChangedPayload, EditSessionDisconnectedPayload,
    EditSessionInfo, EditSessionPhase, EditSessionRegistry, SharedEditSessions,
};
use crate::edit_util::{
    edit_session_dir, exceeds_edit_size_limit, file_fingerprint, local_edit_file_path,
    looks_binary, remote_basename, sweep_orphan_edit_temps, ORPHAN_TEMP_TTL,
    MAX_EXTERNAL_EDIT_BYTES,
};
use crate::elevated_upload::{
    run_elevated_upload, EditUploadError, ExecOutcome, UploadErrorKind,
};
use crate::preferences::{
    get_preferred_external_editor, set_preferred_external_editor,
};
use crate::{
    get_db_conn, is_would_block_ssh, LiveSsh, SshConnections, SshStdoutPayload,
};

pub struct EditWatchers(pub Mutex<HashMap<String, RecommendedWatcher>>);

pub fn manage_edit_state() -> (SharedEditSessions, EditWatchers) {
    (new_shared_registry(), EditWatchers(Mutex::new(HashMap::new())))
}

pub fn sweep_orphans_on_startup(app: &AppHandle) {
    if let Ok(dir) = app.path().app_data_dir() {
        sweep_orphan_edit_temps(&dir, ORPHAN_TEMP_TTL, std::time::SystemTime::now());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEditProbe {
    pub size: u64,
    pub too_large: bool,
    pub looks_binary: bool,
    pub basename: String,
}

fn with_live_ssh(
    state: &SshConnections,
    terminal_id: &str,
) -> Result<Arc<Mutex<LiveSsh>>, String> {
    state
        .0
        .lock()
        .unwrap()
        .get(terminal_id)
        .cloned()
        .ok_or_else(|| "No hay sesión SSH para esta terminal".to_string())
}

fn pump_pty(
    live: &mut LiveSsh,
    pump_buf: &mut [u8],
) -> Vec<u8> {
    let mut pumped = Vec::new();
    loop {
        match live.channel.read(pump_buf) {
            Ok(0) => break,
            Ok(n) => pumped.extend_from_slice(&pump_buf[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => break,
        }
    }
    pumped
}

fn emit_pump(app: &AppHandle, terminal_id: &str, data: &[u8]) {
    if data.is_empty() {
        return;
    }
    let _ = app.emit(
        "ssh-stdout",
        SshStdoutPayload {
            terminal_id: terminal_id.to_string(),
            data: String::from_utf8_lossy(data).into_owned(),
        },
    );
}

fn open_sftp(
    app: &AppHandle,
    terminal_id: &str,
    live_arc: &Arc<Mutex<LiveSsh>>,
) -> Result<ssh2::Sftp, String> {
    let mut pump_buf = [0u8; 4096];
    let mut attempts = 0;
    loop {
        let open_res = {
            let mut live = live_arc.lock().unwrap_or_else(|e| e.into_inner());
            let pumped = pump_pty(&mut live, &mut pump_buf);
            let res = live.session.sftp();
            (res, pumped)
        };
        emit_pump(app, terminal_id, &open_res.1);
        match open_res.0 {
            Ok(s) => return Ok(s),
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
}

/// Descarga archivo remoto a path local (chunks + pump PTY). Producto real.
pub fn sftp_download_file_blocking(
    app: &AppHandle,
    terminal_id: &str,
    live_arc: &Arc<Mutex<LiveSsh>>,
    remote_path: &str,
    local_path: &Path,
) -> Result<(), String> {
    let sftp = open_sftp(app, terminal_id, live_arc)?;
    let mut pump_buf = [0u8; 4096];
    let mut chunk = [0u8; 16 * 1024];

    // Stat size
    let size = {
        let mut attempts = 0;
        loop {
            let res = {
                let mut live = live_arc.lock().unwrap();
                let pumped = pump_pty(&mut live, &mut pump_buf);
                let st = sftp.stat(Path::new(remote_path));
                (st, pumped)
            };
            emit_pump(app, terminal_id, &res.1);
            match res.0 {
                Ok(st) => break st.size.unwrap_or(0),
                Err(e) => {
                    if attempts < 200 && is_would_block_ssh(&e) {
                        attempts += 1;
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                    return Err(format!("Error al obtener tamaño de {}: {}", remote_path, e));
                }
            }
        }
    };
    if exceeds_edit_size_limit(size) {
        return Err(format!(
            "El archivo es demasiado grande para edición externa (máx. {} MiB).",
            MAX_EXTERNAL_EDIT_BYTES / (1024 * 1024)
        ));
    }

    let remote = {
        let mut attempts = 0;
        loop {
            let res = {
                let mut live = live_arc.lock().unwrap();
                let pumped = pump_pty(&mut live, &mut pump_buf);
                let f = sftp.open(Path::new(remote_path));
                (f, pumped)
            };
            emit_pump(app, terminal_id, &res.1);
            match res.0 {
                Ok(f) => break f,
                Err(e) => {
                    if attempts < 200 && is_would_block_ssh(&e) {
                        attempts += 1;
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                    return Err(format!("Error al abrir remoto {}: {}", remote_path, e));
                }
            }
        }
    };

    if let Some(parent) = local_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut local = std::fs::File::create(local_path)
        .map_err(|e| format!("Error al crear temp local: {}", e))?;

    // ssh2::File Read requires &mut; hold lock briefly per chunk + pump
    let mut remote_file = remote;
    loop {
        let read_res = {
            let mut live = live_arc.lock().unwrap();
            let pumped = pump_pty(&mut live, &mut pump_buf);
            let n = remote_file.read(&mut chunk);
            (n, pumped)
        };
        emit_pump(app, terminal_id, &read_res.1);
        match read_res.0 {
            Ok(0) => break,
            Ok(n) => {
                local
                    .write_all(&chunk[..n])
                    .map_err(|e| format!("Error al escribir temp: {}", e))?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(e) => return Err(format!("Error al leer remoto: {}", e)),
        }
    }
    local.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Sube/reemplaza archivo remoto desde local (producto real tras confirm del usuario).
pub fn sftp_upload_file_blocking(
    app: &AppHandle,
    terminal_id: &str,
    live_arc: &Arc<Mutex<LiveSsh>>,
    local_path: &Path,
    remote_path: &str,
) -> Result<(), String> {
    let data = std::fs::read(local_path).map_err(|e| format!("Error al leer local: {}", e))?;
    let sftp = open_sftp(app, terminal_id, live_arc)?;
    let mut pump_buf = [0u8; 4096];

    // Truncate/create
    let mut remote = {
        let mut attempts = 0;
        loop {
            let res = {
                let mut live = live_arc.lock().unwrap();
                let pumped = pump_pty(&mut live, &mut pump_buf);
                let f = sftp.create(Path::new(remote_path));
                (f, pumped)
            };
            emit_pump(app, terminal_id, &res.1);
            match res.0 {
                Ok(f) => break f,
                Err(e) => {
                    if attempts < 200 && is_would_block_ssh(&e) {
                        attempts += 1;
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                    return Err(format!("Error al crear remoto {}: {}", remote_path, e));
                }
            }
        }
    };

    let mut offset = 0;
    while offset < data.len() {
        let end = (offset + 16 * 1024).min(data.len());
        let write_res = {
            let mut live = live_arc.lock().unwrap();
            let pumped = pump_pty(&mut live, &mut pump_buf);
            let n = remote.write(&data[offset..end]);
            (n, pumped)
        };
        emit_pump(app, terminal_id, &write_res.1);
        match write_res.0 {
            Ok(0) => {
                return Err("Escritura remota devolvió 0 bytes".to_string());
            }
            Ok(n) => offset += n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err(e) => return Err(format!("Error al escribir remoto: {}", e)),
        }
    }
    Ok(())
}

/// Best-effort unlink remoto (cleanup de temp elevado).
fn sftp_unlink_best_effort(
    app: &AppHandle,
    terminal_id: &str,
    live_arc: &Arc<Mutex<LiveSsh>>,
    remote_path: &str,
) {
    let Ok(sftp) = open_sftp(app, terminal_id, live_arc) else {
        return;
    };
    let mut pump_buf = [0u8; 4096];
    let mut attempts = 0;
    loop {
        let res = {
            let mut live = live_arc.lock().unwrap();
            let pumped = pump_pty(&mut live, &mut pump_buf);
            let r = sftp.unlink(Path::new(remote_path));
            (r, pumped)
        };
        emit_pump(app, terminal_id, &res.1);
        match res.0 {
            Ok(()) => return,
            Err(e) => {
                if attempts < 50 && is_would_block_ssh(&e) {
                    attempts += 1;
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                return;
            }
        }
    }
}

/// Exec corto en canal aparte (misma Session). No usa set_blocking(true) prolongado.
pub fn exec_remote_command_blocking(
    app: &AppHandle,
    terminal_id: &str,
    live_arc: &Arc<Mutex<LiveSsh>>,
    command: &str,
    timeout: std::time::Duration,
) -> Result<ExecOutcome, String> {
    let mut pump_buf = [0u8; 4096];
    let mut channel = {
        let mut attempts = 0;
        loop {
            let res = {
                let mut live = live_arc.lock().unwrap();
                let pumped = pump_pty(&mut live, &mut pump_buf);
                let ch = live.session.channel_session();
                (ch, pumped)
            };
            emit_pump(app, terminal_id, &res.1);
            match res.0 {
                Ok(ch) => break ch,
                Err(e) => {
                    if attempts < 200 && is_would_block_ssh(&e) {
                        attempts += 1;
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                    return Err(format!("Error al abrir canal exec: {}", e));
                }
            }
        }
    };

    {
        let mut attempts = 0;
        loop {
            let res = {
                let mut live = live_arc.lock().unwrap();
                let pumped = pump_pty(&mut live, &mut pump_buf);
                let r = channel.exec(command);
                (r, pumped)
            };
            emit_pump(app, terminal_id, &res.1);
            match res.0 {
                Ok(()) => break,
                Err(e) => {
                    if attempts < 200 && is_would_block_ssh(&e) {
                        attempts += 1;
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                    return Err(format!("Error al ejecutar comando remoto: {}", e));
                }
            }
        }
    }

    let started = Instant::now();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut out_buf = [0u8; 4096];
    let mut err_buf = [0u8; 4096];

    loop {
        if started.elapsed() > timeout {
            let _ = channel.close();
            return Err("Timeout al ejecutar comando elevado".to_string());
        }
        {
            let mut live = live_arc.lock().unwrap();
            let pumped = pump_pty(&mut live, &mut pump_buf);
            emit_pump(app, terminal_id, &pumped);
        }
        match channel.read(&mut out_buf) {
            Ok(0) => {}
            Ok(n) => stdout.extend_from_slice(&out_buf[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(format!("Error al leer stdout exec: {}", e)),
        }
        match channel.stderr().read(&mut err_buf) {
            Ok(0) => {}
            Ok(n) => stderr.extend_from_slice(&err_buf[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => {}
        }
        let eof = channel.eof();
        if eof {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }

    let mut attempts = 0;
    let exit_code = loop {
        match channel.exit_status() {
            Ok(code) => break code,
            Err(e) => {
                if attempts < 100 && is_would_block_ssh(&e) {
                    attempts += 1;
                    {
                        let mut live = live_arc.lock().unwrap();
                        let pumped = pump_pty(&mut live, &mut pump_buf);
                        emit_pump(app, terminal_id, &pumped);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                break 1;
            }
        }
    };
    let _ = channel.close();

    Ok(ExecOutcome {
        exit_code,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    })
}

fn elevated_upload_blocking(
    app: &AppHandle,
    terminal_id: &str,
    live_arc: &Arc<Mutex<LiveSsh>>,
    local_path: &Path,
    remote_dest: &str,
    edit_id: &str,
) -> Result<(), EditUploadError> {
    let app_u = app.clone();
    let tid_u = terminal_id.to_string();
    let live_u = live_arc.clone();
    let app_e = app.clone();
    let tid_e = terminal_id.to_string();
    let live_e = live_arc.clone();
    let app_c = app.clone();
    let tid_c = terminal_id.to_string();
    let live_c = live_arc.clone();

    run_elevated_upload(
        local_path,
        remote_dest,
        edit_id,
        move |local, temp| {
            sftp_upload_file_blocking(&app_u, &tid_u, &live_u, local, temp)
        },
        move |cmd| {
            exec_remote_command_blocking(
                &app_e,
                &tid_e,
                &live_e,
                cmd,
                std::time::Duration::from_secs(25),
            )
        },
        move |temp| {
            sftp_unlink_best_effort(&app_c, &tid_c, &live_c, temp);
        },
    )
}

fn sample_remote_file(
    app: &AppHandle,
    terminal_id: &str,
    live_arc: &Arc<Mutex<LiveSsh>>,
    remote_path: &str,
    max_bytes: usize,
) -> Result<(u64, Vec<u8>), String> {
    let sftp = open_sftp(app, terminal_id, live_arc)?;
    let mut pump_buf = [0u8; 4096];
    let size = {
        let mut attempts = 0;
        loop {
            let res = {
                let mut live = live_arc.lock().unwrap();
                let pumped = pump_pty(&mut live, &mut pump_buf);
                let st = sftp.stat(Path::new(remote_path));
                (st, pumped)
            };
            emit_pump(app, terminal_id, &res.1);
            match res.0 {
                Ok(st) => break st.size.unwrap_or(0),
                Err(e) => {
                    if attempts < 200 && is_would_block_ssh(&e) {
                        attempts += 1;
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                    return Err(format!("Error al obtener tamaño: {}", e));
                }
            }
        }
    };
    let mut remote = {
        let mut attempts = 0;
        loop {
            let res = {
                let mut live = live_arc.lock().unwrap();
                let pumped = pump_pty(&mut live, &mut pump_buf);
                let f = sftp.open(Path::new(remote_path));
                (f, pumped)
            };
            emit_pump(app, terminal_id, &res.1);
            match res.0 {
                Ok(f) => break f,
                Err(e) => {
                    if attempts < 200 && is_would_block_ssh(&e) {
                        attempts += 1;
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                    return Err(format!("Error al abrir remoto: {}", e));
                }
            }
        }
    };
    let mut buf = vec![0u8; max_bytes];
    let n = {
        let mut live = live_arc.lock().unwrap();
        let pumped = pump_pty(&mut live, &mut pump_buf);
        let n = remote.read(&mut buf).unwrap_or(0);
        emit_pump(app, terminal_id, &pumped);
        n
    };
    buf.truncate(n);
    Ok((size, buf))
}

pub fn open_local_in_editor(app: &AppHandle, local_path: &Path, preferred: &str) -> Result<(), String> {
    let path_str = local_path.to_string_lossy().into_owned();
    if !preferred.trim().is_empty() {
        let exe = Path::new(preferred.trim());
        if exe.exists() {
            std::process::Command::new(exe)
                .arg(&path_str)
                .spawn()
                .map_err(|e| format!("No se pudo abrir el editor preferido: {}", e))?;
            return Ok(());
        }
    }
    // Fallback: asociación OS vía opener
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(&path_str, None::<&str>)
        .map_err(|e| format!("No se pudo abrir con la asociación del SO: {}", e))?;
    Ok(())
}

fn stop_watcher(watchers: &EditWatchers, edit_id: &str) {
    let _ = watchers.0.lock().unwrap().remove(edit_id);
}

pub fn disconnect_edit_sessions_for_terminal(
    app: &AppHandle,
    terminal_id: &str,
    edits: &SharedEditSessions,
    watchers: &EditWatchers,
    preserve_temps: bool,
) {
    let taken = {
        let mut reg = edits.lock().unwrap();
        reg.take_for_terminal(terminal_id, preserve_temps)
    };
    if taken.is_empty() {
        return;
    }
    let mut ids = Vec::new();
    for rec in taken {
        stop_watcher(watchers, &rec.edit_id);
        // Solo borrar temp si no hay que preservar trabajo del usuario
        if !rec.preserve_temp_on_close {
            if let Some(parent) = rec.local_path.parent() {
                let _ = std::fs::remove_dir_all(parent);
            }
        } else {
            ids.push(rec.edit_id.clone());
        }
    }
    if !ids.is_empty() {
        let _ = app.emit(
            "edit-session-disconnected",
            EditSessionDisconnectedPayload {
                terminal_id: terminal_id.to_string(),
                edit_ids: ids,
                message: "La sesión se desconectó; no se pudo subir. El archivo local se conservó temporalmente."
                    .into(),
            },
        );
    }
}

// --- Tauri commands ---

#[tauri::command]
pub async fn get_preferred_external_editor_cmd(app: AppHandle) -> Result<String, String> {
    let conn = get_db_conn(&app)?;
    get_preferred_external_editor(&conn)
}

#[tauri::command]
pub async fn set_preferred_external_editor_cmd(
    app: AppHandle,
    path: String,
) -> Result<(), String> {
    let conn = get_db_conn(&app)?;
    set_preferred_external_editor(&conn, &path)
}

#[tauri::command]
pub async fn probe_external_edit(
    app: AppHandle,
    terminal_id: String,
    remote_path: String,
    state: State<'_, SshConnections>,
) -> Result<ExternalEditProbe, String> {
    let live_arc = with_live_ssh(&state, &terminal_id)?;
    let app2 = app.clone();
    let tid = terminal_id.clone();
    let rpath = remote_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (size, sample) = sample_remote_file(&app2, &tid, &live_arc, &rpath, 4096)?;
        Ok(ExternalEditProbe {
            size,
            too_large: exceeds_edit_size_limit(size),
            looks_binary: looks_binary(&sample),
            basename: remote_basename(&rpath),
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn sftp_download_file(
    app: AppHandle,
    terminal_id: String,
    remote_path: String,
    local_path: String,
    state: State<'_, SshConnections>,
) -> Result<(), String> {
    let live_arc = with_live_ssh(&state, &terminal_id)?;
    let app2 = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        sftp_download_file_blocking(
            &app2,
            &terminal_id,
            &live_arc,
            &remote_path,
            Path::new(&local_path),
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn sftp_upload_file(
    app: AppHandle,
    terminal_id: String,
    local_path: String,
    remote_path: String,
    state: State<'_, SshConnections>,
) -> Result<(), String> {
    let live_arc = with_live_ssh(&state, &terminal_id)?;
    let app2 = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        sftp_upload_file_blocking(
            &app2,
            &terminal_id,
            &live_arc,
            Path::new(&local_path),
            &remote_path,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn start_external_edit(
    app: AppHandle,
    terminal_id: String,
    remote_path: String,
    state: State<'_, SshConnections>,
    edits: State<'_, SharedEditSessions>,
    watchers: State<'_, EditWatchers>,
) -> Result<EditSessionInfo, String> {
    // Reuse?
    {
        let reg = edits.lock().unwrap();
        if let Some(existing) = reg.find(&terminal_id, &remote_path) {
            if existing.phase != EditSessionPhase::Closed {
                let preferred = {
                    let conn = get_db_conn(&app)?;
                    get_preferred_external_editor(&conn).unwrap_or_default()
                };
                open_local_in_editor(&app, &existing.local_path, &preferred)?;
                return Ok(EditSessionInfo {
                    edit_id: existing.edit_id.clone(),
                    terminal_id: existing.terminal_id.clone(),
                    remote_path: existing.remote_path.clone(),
                    local_path: existing.local_path.to_string_lossy().into_owned(),
                    reused: true,
                    phase: existing.phase,
                });
            }
        }
    }

    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let edit_id = Uuid::new_v4().to_string();
    let session_dir = edit_session_dir(&app_data, &edit_id);
    let local_path = local_edit_file_path(&session_dir, &remote_basename(&remote_path));

    let live_arc = with_live_ssh(&state, &terminal_id)?;
    let app_dl = app.clone();
    let tid = terminal_id.clone();
    let rpath = remote_path.clone();
    let lpath = local_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        sftp_download_file_blocking(&app_dl, &tid, &live_arc, &rpath, &lpath)
    })
    .await
    .map_err(|e| e.to_string())??;

    let baseline = file_fingerprint(&local_path)?;
    let info = {
        let mut reg = edits.lock().unwrap();
        reg.register_or_reuse(
            edit_id.clone(),
            terminal_id.clone(),
            remote_path.clone(),
            local_path.clone(),
            baseline,
        )
    };

    let preferred = {
        let conn = get_db_conn(&app)?;
        get_preferred_external_editor(&conn).unwrap_or_default()
    };
    open_local_in_editor(&app, &local_path, &preferred)?;

    spawn_file_watcher_managed(
        app.clone(),
        info.edit_id.clone(),
        local_path,
        (*edits).clone(),
        &watchers,
    )?;

    Ok(info)
}

fn spawn_file_watcher_managed(
    app: AppHandle,
    edit_id: String,
    local_path: PathBuf,
    edits: SharedEditSessions,
    watchers: &EditWatchers,
) -> Result<(), String> {
    let watch_path = local_path.clone();
    let parent = watch_path
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| "Path local sin directorio padre".to_string())?;
    let file_name = watch_path
        .file_name()
        .map(|s| s.to_os_string())
        .ok_or_else(|| "Path local sin nombre de archivo".to_string())?;

    let edit_id_cb = edit_id.clone();
    let edits_cb = edits.clone();
    let app_cb = app.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        let Ok(event) = res else {
            return;
        };
        let interesting = matches!(
            event.kind,
            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) | EventKind::Any
        );
        if !interesting {
            return;
        }
        let touches = event.paths.iter().any(|p| {
            p.file_name().map(|n| n == file_name).unwrap_or(false) || p == &watch_path
        });
        if !touches {
            return;
        }
        {
            let mut reg = edits_cb.lock().unwrap();
            reg.note_fs_event(&edit_id_cb);
        }
        let edits_timer = edits_cb.clone();
        let app_timer = app_cb.clone();
        let id_timer = edit_id_cb.clone();
        let path_timer = watch_path.clone();
        std::thread::spawn(move || {
            std::thread::sleep(EditSessionRegistry::debounce_duration());
            let fp = match file_fingerprint(&path_timer) {
                Ok(f) => f,
                Err(_) => return,
            };
            let payload = {
                let mut reg = edits_timer.lock().unwrap();
                reg.evaluate_after_debounce(
                    &id_timer,
                    &fp,
                    EditSessionRegistry::debounce_duration(),
                    Instant::now(),
                )
            };
            if let Some(info) = payload {
                let _ = app_timer.emit(
                    "edit-session-changed",
                    EditSessionChangedPayload {
                        edit_id: info.edit_id,
                        terminal_id: info.terminal_id,
                        remote_path: info.remote_path,
                        reason: "content_changed".into(),
                    },
                );
            }
        });
    })
    .map_err(|e| format!("Error al iniciar watcher: {}", e))?;

    watcher
        .watch(&parent, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Error al vigilar temp: {}", e))?;

    watchers.0.lock().unwrap().insert(edit_id, watcher);
    Ok(())
}

#[tauri::command]
pub async fn confirm_edit_upload(
    app: AppHandle,
    edit_id: String,
    state: State<'_, SshConnections>,
    edits: State<'_, SharedEditSessions>,
) -> Result<(), EditUploadError> {
    let (terminal_id, remote_path, local_path) = {
        let mut reg = edits.lock().unwrap();
        if let Err(e) = reg.begin_upload(&edit_id) {
            return Err(EditUploadError::new(UploadErrorKind::Other, e));
        }
        let rec = match reg.get(&edit_id) {
            Some(r) => r,
            None => {
                return Err(EditUploadError::new(
                    UploadErrorKind::Other,
                    "Sesión de edición no encontrada",
                ));
            }
        };
        (
            rec.terminal_id.clone(),
            rec.remote_path.clone(),
            rec.local_path.clone(),
        )
    };

    let live_arc = match with_live_ssh(&state, &terminal_id) {
        Ok(a) => a,
        Err(e) => {
            edits.lock().unwrap().fail_upload(&edit_id);
            return Err(EditUploadError::from_upload_message(e));
        }
    };

    let app2 = app.clone();
    let tid = terminal_id.clone();
    let lpath = local_path.clone();
    let rpath = remote_path.clone();
    let upload_res = tauri::async_runtime::spawn_blocking(move || {
        sftp_upload_file_blocking(&app2, &tid, &live_arc, &lpath, &rpath)
    })
    .await
    .map_err(|e| EditUploadError::new(UploadErrorKind::Other, e.to_string()))?;

    match upload_res {
        Ok(()) => {
            let fp = file_fingerprint(&local_path).map_err(|e| {
                EditUploadError::new(UploadErrorKind::Other, e)
            })?;
            edits.lock().unwrap().finish_upload(&edit_id, fp);
            Ok(())
        }
        Err(e) => {
            edits.lock().unwrap().fail_upload(&edit_id);
            Err(EditUploadError::from_upload_message(e))
        }
    }
}

/// Un único reintento elevado tras fallo de permisos (temp remoto + sudo -n cp).
#[tauri::command]
pub async fn edit_session_upload_with_sudo(
    app: AppHandle,
    edit_id: String,
    state: State<'_, SshConnections>,
    edits: State<'_, SharedEditSessions>,
) -> Result<(), EditUploadError> {
    let (terminal_id, remote_path, local_path) = {
        let mut reg = edits.lock().unwrap();
        if let Err(e) = reg.begin_upload(&edit_id) {
            return Err(EditUploadError::new(UploadErrorKind::Other, e));
        }
        let rec = match reg.get(&edit_id) {
            Some(r) => r,
            None => {
                return Err(EditUploadError::new(
                    UploadErrorKind::Other,
                    "Sesión de edición no encontrada",
                ));
            }
        };
        (
            rec.terminal_id.clone(),
            rec.remote_path.clone(),
            rec.local_path.clone(),
        )
    };

    let live_arc = match with_live_ssh(&state, &terminal_id) {
        Ok(a) => a,
        Err(e) => {
            edits.lock().unwrap().fail_upload(&edit_id);
            return Err(EditUploadError::from_upload_message(e));
        }
    };

    let app2 = app.clone();
    let tid = terminal_id.clone();
    let lpath = local_path.clone();
    let rpath = remote_path.clone();
    let eid = edit_id.clone();
    let elevated_res = tauri::async_runtime::spawn_blocking(move || {
        elevated_upload_blocking(&app2, &tid, &live_arc, &lpath, &rpath, &eid)
    })
    .await
    .map_err(|e| EditUploadError::new(UploadErrorKind::Other, e.to_string()))?;

    match elevated_res {
        Ok(()) => {
            let fp = file_fingerprint(&local_path).map_err(|e| {
                EditUploadError::new(UploadErrorKind::Other, e)
            })?;
            edits.lock().unwrap().finish_upload(&edit_id, fp);
            Ok(())
        }
        Err(e) => {
            edits.lock().unwrap().fail_upload(&edit_id);
            Err(e)
        }
    }
}

#[tauri::command]
pub async fn dismiss_edit_change(
    edit_id: String,
    edits: State<'_, SharedEditSessions>,
) -> Result<(), String> {
    edits.lock().unwrap().dismiss_confirm(&edit_id);
    Ok(())
}

#[tauri::command]
pub async fn stop_external_edit(
    edit_id: String,
    edits: State<'_, SharedEditSessions>,
    watchers: State<'_, EditWatchers>,
) -> Result<(), String> {
    stop_watcher(&watchers, &edit_id);
    if let Some(rec) = edits.lock().unwrap().remove(&edit_id) {
        if rec.phase != EditSessionPhase::Uploading
            && rec.phase != EditSessionPhase::ConfirmPending
        {
            if let Some(parent) = rec.local_path.parent() {
                let _ = std::fs::remove_dir_all(parent);
            }
        }
    }
    Ok(())
}
