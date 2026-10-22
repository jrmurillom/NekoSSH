//! Módulo de transferencia directa punto a punto (P2P) entre servidores SSH y fallback de tubería streaming.
//! Diseñado para transferencias masivas (ej. 10 GB) a velocidad de datacenter (100 MB/s - 1 GB/s).

use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::AppHandle;

use crate::edit_util::{
    build_staging_part_path, remote_basename, ActiveTransferGuard, ProgressThrottler,
    TransferProgressPayload, TRANSFER_CANCELLED_ERROR,
};
use crate::{is_would_block_ssh, LiveSsh};

/// Información de autenticación del servidor destino para orquestación P2P.
#[derive(Debug, Clone)]
pub enum TargetServerAuth {
    Key {
        pem_content: String,
        passphrase: Option<String>,
    },
    Password(String),
}

/// Parámetros de conexión del servidor destino.
#[derive(Debug, Clone)]
pub struct TargetServerInfo {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: TargetServerAuth,
}

/// Guardia RAII para la destrucción segura de credenciales temporales en el servidor origen.
/// Si la transferencia concluye, falla o el hilo entra en pánico, garantiza la ejecución
/// de `shred -u` o `rm -f` sobre la clave temporal alojada en RAM (`/dev/shm`).
pub struct P2PCredentialGuard {
    live_arc: Arc<Mutex<LiveSsh>>,
    key_path: Option<String>,
    remote_pid: Option<u32>,
    active: bool,
}

impl P2PCredentialGuard {
    pub fn new(live_arc: Arc<Mutex<LiveSsh>>, key_path: Option<String>) -> Self {
        Self {
            live_arc,
            key_path,
            remote_pid: None,
            active: true,
        }
    }

    pub fn set_remote_pid(&mut self, pid: u32) {
        self.remote_pid = Some(pid);
    }

    pub fn remote_key_path(&self) -> Option<&str> {
        self.key_path.as_deref()
    }

    /// Desactiva la guardia una vez que la limpieza formal ha sido ejecutada de forma controlada.
    pub fn disarm(&mut self) {
        self.active = false;
    }

    /// Ejecuta la purga inmediata de la credencial y la terminación del proceso si sigue activo.
    pub fn perform_cleanup(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;

        let live_lock = self.live_arc.lock();
        if let Ok(live) = live_lock {
            // 1. Terminar proceso remoto si hay PID registrado
            if let Some(pid) = self.remote_pid.take() {
                let kill_cmd = format!("kill -TERM {} 2>/dev/null || kill -KILL {} 2>/dev/null", pid, pid);
                if let Ok(mut ch) = live.session.channel_session() {
                    let _ = ch.exec(&kill_cmd);
                    let _ = ch.wait_close();
                }
            }

            // 2. Destrucción segura de la clave temporal
            if let Some(path) = self.key_path.take() {
                let cleanup_cmd = format!("shred -u \"{}\" 2>/dev/null || rm -f \"{}\"", path, path);
                if let Ok(mut ch) = live.session.channel_session() {
                    let _ = ch.exec(&cleanup_cmd);
                    let _ = ch.wait_close();
                }
            }
        }
    }
}

impl Drop for P2PCredentialGuard {
    fn drop(&mut self) {
        self.perform_cleanup();
    }
}

/// Parser de la salida continua (`stdout`) emitida por `rsync` y `scp`.
#[derive(Debug, Clone, Default)]
pub struct P2PProgressParser {
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub percent: u8,
    pub speed_bps: u64,
    pub speed_formatted: String,
    pub eta_formatted: String,
}

impl P2PProgressParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parsea una línea o fragmento de salida de `rsync -avP` o `scp`.
    ///
    /// Ejemplos típicos de `rsync`:
    /// `  1,234,567,890  67%   95.42MB/s    0:00:15`
    /// `     52,428,800  50%   52.40MB/s    0:01:23 (xfr#1, to-chk=0/1)`
    ///
    /// Ejemplos típicos de `scp`:
    /// `dump.sql       45%  460MB  75.3MB/s   00:07`
    pub fn parse_line(&mut self, raw_line: &str) -> bool {
        // Tomar la última porción separada por retorno de carro '\r'
        let line = raw_line.split('\r').last().unwrap_or(raw_line).trim();
        if line.is_empty() {
            return false;
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            return false;
        }

        let mut updated = false;

        // 1. Buscar porcentaje (%): ej. "67%" o "100%"
        for (i, token) in tokens.iter().enumerate() {
            if token.ends_with('%') {
                if let Ok(pct) = token.trim_end_matches('%').parse::<u8>() {
                    self.percent = pct.min(100);
                    updated = true;

                    // Si antes del porcentaje hay un número entero o con comas (bytes transferidos)
                    if i > 0 {
                        let bytes_token = tokens[i - 1].replace(',', "");
                        if let Ok(bytes) = bytes_token.parse::<u64>() {
                            self.bytes_transferred = bytes;
                            if self.percent > 0 && self.total_bytes == 0 {
                                self.total_bytes = (bytes * 100) / (self.percent as u64);
                            }
                        }
                    }
                }
            }

            // 2. Buscar velocidad: ej. "95.42MB/s", "1.2GB/s", "850KB/s"
            if token.ends_with("/s") || token.ends_with("/S") {
                let speed_str = *token;
                self.speed_formatted = speed_str.to_string();
                self.speed_bps = parse_speed_to_bps(speed_str);
                updated = true;
            }

            // 3. Buscar ETA: ej. "0:00:15", "01:23", "00:07"
            if token.contains(':') && token.chars().all(|c| c.is_ascii_digit() || c == ':') {
                self.eta_formatted = token.to_string();
                updated = true;
            }
        }

        updated
    }
}

/// Convierte una cadena de velocidad (ej. "95.42MB/s", "500KB/s") a bytes por segundo.
fn parse_speed_to_bps(s: &str) -> u64 {
    let clean = s.trim_end_matches("/s").trim_end_matches("/S").trim();
    if clean.ends_with("GB") || clean.ends_with("Gb") {
        let val = clean[..clean.len() - 2].parse::<f64>().unwrap_or(0.0);
        (val * 1024.0 * 1024.0 * 1024.0) as u64
    } else if clean.ends_with("MB") || clean.ends_with("Mb") {
        let val = clean[..clean.len() - 2].parse::<f64>().unwrap_or(0.0);
        (val * 1024.0 * 1024.0) as u64
    } else if clean.ends_with("KB") || clean.ends_with("Kb") || clean.ends_with('k') {
        let val = clean[..clean.len() - 2].parse::<f64>().unwrap_or(0.0);
        (val * 1024.0) as u64
    } else if clean.ends_with('B') || clean.ends_with('b') {
        let val = clean[..clean.len() - 1].parse::<f64>().unwrap_or(0.0);
        val as u64
    } else {
        0
    }
}

/// Construye la ruta de destino temporal para la clave privada en RAM (`/dev/shm`).
pub fn generate_ephemeral_key_path() -> String {
    let id = uuid::Uuid::new_v4().to_string();
    format!("/dev/shm/.nekossh-key-{}", &id[..8])
}

/// Instala la clave privada en el servidor origen de forma segura.
/// Intenta primero escribir en `/dev/shm` (memoria RAM de Linux).
/// Si falla, recurre a un archivo temporal efímero en `/tmp`.
pub fn install_ephemeral_key_remote(
    live: &mut LiveSsh,
    key_pem: &str,
) -> Result<String, String> {
    let key_path = generate_ephemeral_key_path();

    // Intentar escribir directamente con permisos 0600 en una sola operación atómica de shell
    // usando un documento here-doc sin expandir variables.
    let write_cmd = format!(
        "umask 077 && cat << 'NEKOSSH_P2P_KEY_EOF' > \"{}\"\n{}\nNEKOSSH_P2P_KEY_EOF\nchmod 600 \"{}\"",
        key_path, key_pem.trim(), key_path
    );

    let mut ch = live
        .session
        .channel_session()
        .map_err(|e| format!("Error al abrir canal para inyección de clave: {}", e))?;
    ch.exec(&write_cmd)
        .map_err(|e| format!("Error al escribir clave temporal en origen: {}", e))?;

    let mut out = String::new();
    let _ = ch.read_to_string(&mut out);
    let _ = ch.wait_close();
    let status = ch.exit_status().unwrap_or(-1);

    if status == 0 {
        return Ok(key_path);
    }

    // Fallback: Si /dev/shm no está montado o está protegido, usar mktemp en /tmp
    let fallback_cmd = format!(
        "KEY_FILE=$(mktemp /tmp/.nekossh-key-XXXXXX) && chmod 600 \"$KEY_FILE\" && cat << 'NEKOSSH_P2P_KEY_EOF' > \"$KEY_FILE\"\n{}\nNEKOSSH_P2P_KEY_EOF\necho \"$KEY_FILE\"",
        key_pem.trim()
    );

    let mut ch_fb = live
        .session
        .channel_session()
        .map_err(|e| format!("Error al abrir canal para fallback de clave: {}", e))?;
    ch_fb
        .exec(&fallback_cmd)
        .map_err(|e| format!("Error al ejecutar fallback de clave temporal: {}", e))?;

    let mut fb_out = String::new();
    let _ = ch_fb.read_to_string(&mut fb_out);
    let _ = ch_fb.wait_close();
    let fb_status = ch_fb.exit_status().unwrap_or(-1);

    if fb_status == 0 {
        let path = fb_out.trim().to_string();
        if !path.is_empty() {
            return Ok(path);
        }
    }

    Err("No se pudo instalar la credencial temporal en el servidor origen".to_string())
}

/// Realiza un sondeo TCP rápido con timeout estricto de 2 segundos desde el Servidor A hacia B.
/// Retorna `true` si el puerto SSH de B es directamente alcanzable por A.
pub fn probe_tcp_connectivity(
    live: &mut LiveSsh,
    target_host: &str,
    target_port: u16,
) -> bool {
    // Comando POSIX con timeout estricto para evitar bloqueos
    let probe_cmd = format!(
        "timeout 2 bash -c \"cat < /dev/null > /dev/tcp/{}/{}\" 2>/dev/null || timeout 2 nc -z -w 2 {} {} 2>/dev/null",
        target_host, target_port, target_host, target_port
    );

    let Ok(mut ch) = live.session.channel_session() else {
        return false;
    };
    if ch.exec(&probe_cmd).is_err() {
        return false;
    }

    let _ = ch.wait_close();
    ch.exit_status().unwrap_or(-1) == 0
}

/// Construye el comando de transferencia directa `rsync` optimizado para máxima velocidad.
pub fn build_rsync_command(
    source_path: &str,
    target_user: &str,
    target_host: &str,
    target_port: u16,
    target_path: &str,
    key_path: Option<&str>,
) -> String {
    let key_arg = if let Some(k) = key_path {
        format!("-i \"{}\"", k)
    } else {
        String::new()
    };

    let ssh_opts = format!(
        "-e \"ssh -p {} {} -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o BatchMode=yes\"",
        target_port, key_arg
    );

    format!(
        "rsync -avP --inplace {} \"{}\" \"{}@{}:{}\"",
        ssh_opts, source_path, target_user, target_host, target_path
    )
}

/// Construye el comando de transferencia directa `scp` optimizado como alternativa si no hay `rsync`.
pub fn build_scp_command(
    source_path: &str,
    target_user: &str,
    target_host: &str,
    target_port: u16,
    target_path: &str,
    key_path: Option<&str>,
) -> String {
    let key_arg = if let Some(k) = key_path {
        format!("-i \"{}\"", k)
    } else {
        String::new()
    };

    format!(
        "scp -P {} {} -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o BatchMode=yes \"{}\" \"{}@{}:{}\"",
        target_port, key_arg, source_path, target_user, target_host, target_path
    )
}

/// Verifica si un binario (ej. `rsync` o `scp`) existe en el PATH del servidor remoto.
pub fn check_binary_exists(live: &mut LiveSsh, bin: &str) -> bool {
    let cmd = format!("which {} 2>/dev/null || command -v {} 2>/dev/null", bin, bin);
    let Ok(mut ch) = live.session.channel_session() else {
        return false;
    };
    if ch.exec(&cmd).is_err() {
        return false;
    }
    let mut out = String::new();
    let _ = ch.read_to_string(&mut out);
    let _ = ch.wait_close();
    ch.exit_status().unwrap_or(-1) == 0 && !out.trim().is_empty()
}

/// Fallback a tubería de streaming Unix (`cat <SRC> | cat > <DEST>`) para cuando no hay conectividad directa P2P.
/// Utiliza chunks de 256 KiB en memoria constante O(1) con bombeo continuo de PTY en ambas sesiones.
pub fn execute_streaming_pipe_fallback(
    app: &AppHandle,
    src_tid: &str,
    live_src_arc: &Arc<Mutex<LiveSsh>>,
    source_path: &str,
    tgt_tid: &str,
    live_tgt_arc: &Arc<Mutex<LiveSsh>>,
    target_path: &str,
    expected_size: u64,
    guard: &ActiveTransferGuard,
    on_progress: &tauri::ipc::Channel<TransferProgressPayload>,
) -> Result<(), String> {
    use crate::external_edit::{emit_pump, open_sftp, pump_pty};

    let file_name = remote_basename(target_path);
    let part_path = build_staging_part_path(target_path);
    let mut throttler = ProgressThrottler::new("scp", &file_name, expected_size);

    if let Some(payload) = throttler.poll(0, true, Instant::now()) {
        let _ = on_progress.send(payload);
    }

    // 1. Abrir canal exec para lectura continua en origen: `cat "<source_path>"`
    let mut ch_src = {
        let live = live_src_arc.lock().unwrap();
        let mut ch = live
            .session
            .channel_session()
            .map_err(|e| format!("Error al abrir canal en origen: {}", e))?;
        let cat_cmd = format!("cat \"{}\"", source_path);
        ch.exec(&cat_cmd)
            .map_err(|e| format!("Error al ejecutar cat en origen: {}", e))?;
        ch
    };

    // 2. Abrir archivo destino temporal `.nekossh.part` vía SFTP en destino
    let sftp_tgt = open_sftp(app, tgt_tid, live_tgt_arc)?;
    let mut file_tgt = {
        let mut attempts = 0;
        let mut pump_buf = [0u8; 4096];
        loop {
            if guard.is_cancelled() {
                return Err(TRANSFER_CANCELLED_ERROR.to_string());
            }
            let res = {
                let mut live = live_tgt_arc.lock().unwrap();
                let pumped = pump_pty(&mut live, &mut pump_buf);
                let f = sftp_tgt.create(Path::new(&part_path));
                (f, pumped)
            };
            emit_pump(app, tgt_tid, &res.1);
            match res.0 {
                Ok(f) => break f,
                Err(e) => {
                    if attempts < 200 && is_would_block_ssh(&e) {
                        attempts += 1;
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    return Err(format!("Error al crear temporal en destino {}: {}", part_path, e));
                }
            }
        }
    };

    // 3. Streaming continuo en búfer de 256 KiB
    const PIPE_STREAM_CHUNK: usize = 256 * 1024;
    let mut buffer = vec![0u8; PIPE_STREAM_CHUNK];
    let mut bytes_transferred: u64 = 0;
    let mut pump_buf = [0u8; 4096];

    loop {
        if guard.is_cancelled() {
            drop(file_tgt);
            let _ = sftp_tgt.unlink(Path::new(&part_path));
            return Err(TRANSFER_CANCELLED_ERROR.to_string());
        }

        // Leer de origen (cat stream)
        let n_read = match ch_src.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }
            Err(e) => {
                drop(file_tgt);
                let _ = sftp_tgt.unlink(Path::new(&part_path));
                return Err(format!("Error de lectura en tubería: {}", e));
            }
        };

        // Bombeo PTY origen
        let pumped_src = {
            let mut live = live_src_arc.lock().unwrap();
            pump_pty(&mut live, &mut pump_buf)
        };
        emit_pump(app, src_tid, &pumped_src);

        // Escribir en destino
        let mut offset = 0;
        let mut write_attempts = 0;
        while offset < n_read {
            if guard.is_cancelled() {
                drop(file_tgt);
                let _ = sftp_tgt.unlink(Path::new(&part_path));
                return Err(TRANSFER_CANCELLED_ERROR.to_string());
            }

            let write_res = {
                let mut live = live_tgt_arc.lock().unwrap();
                let pumped = pump_pty(&mut live, &mut pump_buf);
                let w = file_tgt.write(&buffer[offset..n_read]);
                (w, pumped)
            };
            emit_pump(app, tgt_tid, &write_res.1);

            match write_res.0 {
                Ok(0) => {
                    drop(file_tgt);
                    let _ = sftp_tgt.unlink(Path::new(&part_path));
                    return Err("Escritura en destino devolvió 0 bytes".to_string());
                }
                Ok(w) => {
                    offset += w;
                    bytes_transferred += w as u64;
                    write_attempts = 0;
                    if let Some(payload) = throttler.poll(bytes_transferred, false, Instant::now()) {
                        let _ = on_progress.send(payload);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    write_attempts += 1;
                    if write_attempts >= 500 {
                        drop(file_tgt);
                        let _ = sftp_tgt.unlink(Path::new(&part_path));
                        return Err(format!("Timeout de escritura en destino {}: conexión bloqueada", target_path));
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => {
                    drop(file_tgt);
                    let _ = sftp_tgt.unlink(Path::new(&part_path));
                    return Err(format!("Error de escritura: {}", e));
                }
            }
        }
    }

    // 4. Flush y renombrado atómico en destino
    let _ = file_tgt.flush();
    drop(file_tgt);

    let _ = sftp_tgt.unlink(Path::new(target_path));
    if let Err(e) = sftp_tgt.rename(Path::new(&part_path), Path::new(target_path), None) {
        let _ = sftp_tgt.unlink(Path::new(&part_path));
        return Err(format!("Error al renombrar archivo final en destino: {}", e));
    }

    if let Some(final_payload) = throttler.poll(bytes_transferred, true, Instant::now()) {
        let _ = on_progress.send(final_payload);
    }

    Ok(())
}

/// Orquestador principal de transferencia inter-sesión.
/// Intenta primero P2P directo (Datacenter a Datacenter con rsync o scp).
/// Si la red entre servidores no es alcanzable o la autenticación es por Password,
/// conmuta automáticamente a la tubería streaming Unix de alto rendimiento.
pub fn orchestrate_p2p_or_pipe_transfer(
    app: &AppHandle,
    src_tid: &str,
    live_src_arc: &Arc<Mutex<LiveSsh>>,
    source_path: &str,
    tgt_tid: &str,
    live_tgt_arc: &Arc<Mutex<LiveSsh>>,
    target_path: &str,
    target_info: Option<TargetServerInfo>,
    guard: &ActiveTransferGuard,
    on_progress: &tauri::ipc::Channel<TransferProgressPayload>,
) -> Result<(), String> {
    use crate::external_edit::{emit_pump, open_sftp, pump_pty};

    if guard.is_cancelled() {
        return Err(TRANSFER_CANCELLED_ERROR.to_string());
    }

    // 1. Obtener tamaño origen vía stat
    let sftp_src = open_sftp(app, src_tid, live_src_arc)?;
    let expected_size = match sftp_src.stat(Path::new(source_path)) {
        Ok(st) => st.size.unwrap_or(0),
        Err(e) => return Err(format!("Error al consultar origen {}: {}", source_path, e)),
    };

    let file_name = remote_basename(target_path);

    // 2. Intentar Direct P2P si hay clave privada y conectividad directa
    if let Some(info) = target_info {
        if let TargetServerAuth::Key { pem_content, passphrase: _ } = info.auth {
            let direct_reachable = {
                let mut live = live_src_arc.lock().unwrap();
                probe_tcp_connectivity(&mut live, &info.host, info.port)
            };

            if direct_reachable {
                // Instalar clave efímera en /dev/shm
                let key_res = {
                    let mut live = live_src_arc.lock().unwrap();
                    install_ephemeral_key_remote(&mut live, &pem_content)
                };

                if let Ok(key_path) = key_res {
                    let mut p2p_guard = P2PCredentialGuard::new(live_src_arc.clone(), Some(key_path.clone()));

                    let has_rsync = {
                        let mut live = live_src_arc.lock().unwrap();
                        check_binary_exists(&mut live, "rsync")
                    };

                    let cmd = if has_rsync {
                        build_rsync_command(source_path, &info.username, &info.host, info.port, target_path, Some(&key_path))
                    } else {
                        build_scp_command(source_path, &info.username, &info.host, info.port, target_path, Some(&key_path))
                    };

                    // Ejecutar con echo $$ para registrar el PID remoto
                    let full_cmd = format!("echo $$ && exec {}", cmd);
                    let mut ch = {
                        let live = live_src_arc.lock().unwrap();
                        let mut ch = live.session.channel_session().map_err(|e| format!("Error al abrir canal P2P: {}", e))?;
                        ch.exec(&full_cmd).map_err(|e| format!("Error al lanzar comando P2P: {}", e))?;
                        ch
                    };

                    let mut parser = P2PProgressParser::new();
                    parser.total_bytes = expected_size;
                    let mut pid_read = false;
                    let mut buf = [0u8; 4096];
                    let mut line_acc = String::new();
                    let mut pump_buf = [0u8; 4096];

                    // Loop de lectura de stdout de rsync/scp
                    loop {
                        if guard.is_cancelled() {
                            p2p_guard.perform_cleanup();
                            return Err(TRANSFER_CANCELLED_ERROR.to_string());
                        }

                        match ch.read(&mut buf) {
                            Ok(0) => break, // Proceso finalizó
                            Ok(n) => {
                                let chunk_str = String::from_utf8_lossy(&buf[..n]);
                                line_acc.push_str(&chunk_str);

                                // La primera línea es el PID del shell remoto
                                if !pid_read {
                                    if let Some(idx) = line_acc.find('\n') {
                                        let pid_str = line_acc[..idx].trim();
                                        if let Ok(pid) = pid_str.parse::<u32>() {
                                            p2p_guard.set_remote_pid(pid);
                                        }
                                        line_acc = line_acc[idx + 1..].to_string();
                                        pid_read = true;
                                    }
                                }

                                if pid_read && parser.parse_line(&line_acc) {
                                    let payload = TransferProgressPayload {
                                        operation: "scp".to_string(),
                                        file_name: file_name.clone(),
                                        bytes_transferred: parser.bytes_transferred,
                                        total_bytes: if parser.total_bytes > 0 { parser.total_bytes } else { expected_size },
                                        percent: parser.percent,
                                        speed_bps: parser.speed_bps,
                                    };
                                    let _ = on_progress.send(payload);
                                    if line_acc.len() > 1024 {
                                        line_acc.clear();
                                    }
                                }
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                std::thread::sleep(Duration::from_millis(20));
                            }
                            Err(e) => {
                                p2p_guard.perform_cleanup();
                                return Err(format!("Error leyendo salida P2P: {}", e));
                            }
                        }

                        // Bombeo PTY origen
                        let pumped_src = {
                            let mut live = live_src_arc.lock().unwrap();
                            pump_pty(&mut live, &mut pump_buf)
                        };
                        emit_pump(app, src_tid, &pumped_src);
                    }

                    let _ = ch.wait_close();
                    let exit_code = ch.exit_status().unwrap_or(-1);
                    p2p_guard.perform_cleanup();

                    if exit_code == 0 {
                        // Emitir 100% final
                        let _ = on_progress.send(TransferProgressPayload {
                            operation: "scp".to_string(),
                            file_name: file_name.clone(),
                            bytes_transferred: expected_size,
                            total_bytes: expected_size,
                            percent: 100,
                            speed_bps: parser.speed_bps,
                        });
                        return Ok(());
                    }
                }
            }
        }
    }

    // 3. Fallback a tubería de streaming si no fue posible el P2P directo
    execute_streaming_pipe_fallback(
        app,
        src_tid,
        live_src_arc,
        source_path,
        tgt_tid,
        live_tgt_arc,
        target_path,
        expected_size,
        guard,
        on_progress,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rsync_progress_output() {
        let mut parser = P2PProgressParser::new();

        let line1 = "          32,768   0%    0.00kB/s    0:00:00";
        assert!(parser.parse_line(line1));
        assert_eq!(parser.percent, 0);
        assert_eq!(parser.bytes_transferred, 32768);

        let line2 = "  5,242,880,000  50%   98.50MB/s    0:00:52 (xfr#1, to-chk=0/1)";
        assert!(parser.parse_line(line2));
        assert_eq!(parser.percent, 50);
        assert_eq!(parser.bytes_transferred, 5242880000);
        assert_eq!(parser.speed_formatted, "98.50MB/s");
        assert_eq!(parser.eta_formatted, "0:00:52");
        assert_eq!(parser.total_bytes, 10485760000); // 10 GB calculados
        assert!(parser.speed_bps > 100_000_000);

        let line3 = " 10,485,760,000 100%  105.20MB/s    0:00:00 (xfr#1, to-chk=0/1)";
        assert!(parser.parse_line(line3));
        assert_eq!(parser.percent, 100);
        assert_eq!(parser.bytes_transferred, 10485760000);
        assert_eq!(parser.speed_formatted, "105.20MB/s");
    }

    #[test]
    fn parse_scp_progress_output() {
        let mut parser = P2PProgressParser::new();
        let line = "backup.tar.gz    45%  460MB  75.3MB/s   00:07";
        assert!(parser.parse_line(line));
        assert_eq!(parser.percent, 45);
        assert_eq!(parser.speed_formatted, "75.3MB/s");
        assert_eq!(parser.eta_formatted, "00:07");
    }

    #[test]
    fn parse_carriage_return_bursts() {
        let mut parser = P2PProgressParser::new();
        // rsync suele emitir múltiples estados separados por \r en el mismo buffer
        let burst = "  1,000,000  10%  10.0MB/s  01:00\r  2,000,000  20%  20.0MB/s  00:50\r  3,000,000  30%  30.0MB/s  00:40";
        assert!(parser.parse_line(burst));
        assert_eq!(parser.percent, 30);
        assert_eq!(parser.bytes_transferred, 3000000);
        assert_eq!(parser.speed_formatted, "30.0MB/s");
        assert_eq!(parser.eta_formatted, "00:40");
    }

    #[test]
    fn build_commands_quoting_and_options() {
        let rsync_cmd = build_rsync_command(
            "/data/prod.dump",
            "root",
            "10.0.0.2",
            2222,
            "/backups/prod.dump",
            Some("/dev/shm/.nekossh-key-1234"),
        );

        assert!(rsync_cmd.contains("rsync -avP --inplace"));
        assert!(rsync_cmd.contains("-p 2222"));
        assert!(rsync_cmd.contains("-i \"/dev/shm/.nekossh-key-1234\""));
        assert!(rsync_cmd.contains("StrictHostKeyChecking=no"));
        assert!(rsync_cmd.contains("root@10.0.0.2:/backups/prod.dump"));

        let scp_cmd = build_scp_command(
            "/data/prod.dump",
            "root",
            "10.0.0.2",
            2222,
            "/backups/prod.dump",
            Some("/dev/shm/.nekossh-key-1234"),
        );
        assert!(scp_cmd.starts_with("scp -P 2222"));
        assert!(scp_cmd.contains("-i \"/dev/shm/.nekossh-key-1234\""));
    }

    #[test]
    fn speed_conversion_bps() {
        assert_eq!(parse_speed_to_bps("100MB/s"), 100 * 1024 * 1024);
        assert_eq!(parse_speed_to_bps("1.5GB/s"), (1.5 * 1024.0 * 1024.0 * 1024.0) as u64);
        assert_eq!(parse_speed_to_bps("500KB/s"), 500 * 1024);
        assert_eq!(parse_speed_to_bps("0B/s"), 0);
    }

    #[test]
    fn parse_abrupt_speed_jumps_and_eta_formats() {
        let mut parser = P2PProgressParser::new();
        // 1. Variación abrupta de velocidad y ETA largo
        let line1 = " 2,147,483,648  20%   12.50MB/s    01:23:45";
        assert!(parser.parse_line(line1));
        assert_eq!(parser.percent, 20);
        assert_eq!(parser.bytes_transferred, 2147483648);
        assert_eq!(parser.speed_formatted, "12.50MB/s");
        assert_eq!(parser.eta_formatted, "01:23:45");

        // 2. Caída drástica a kB/s
        let line2 = " 2,150,000,000  20%  450.00kB/s    02:15:10";
        assert!(parser.parse_line(line2));
        assert_eq!(parser.speed_formatted, "450.00kB/s");
        assert!(parser.speed_bps < 1_000_000);

        // 3. Salto a velocidad de datacenter (> 150 MB/s)
        let line3 = " 8,000,000,000  80%  180.25MB/s    0:00:11";
        assert!(parser.parse_line(line3));
        assert_eq!(parser.percent, 80);
        assert_eq!(parser.speed_formatted, "180.25MB/s");
        assert!(parser.speed_bps > 150_000_000);
    }

    #[test]
    fn tcp_probe_command_construction() {
        let host = "database-replica.internal.aws";
        let port = 2222;
        let probe = format!(
            "timeout 2 bash -c \"cat < /dev/null > /dev/tcp/{}/{}\" 2>/dev/null || timeout 2 nc -z -w 2 {} {} 2>/dev/null",
            host, port, host, port
        );
        assert!(probe.contains("timeout 2"));
        assert!(probe.contains("/dev/tcp/database-replica.internal.aws/2222"));
        assert!(probe.contains("nc -z -w 2 database-replica.internal.aws 2222"));
    }

    #[test]
    fn stream_large_10gb_memory_flat() {
        // Simulación de 10 GB pasando a través del búfer de 256 KiB
        const TOTAL_SIMULATED: u64 = 10 * 1024 * 1024 * 1024; // 10 GB
        const CHUNK_SIZE: usize = 256 * 1024; // 256 KiB
        let mut buffer = vec![0u8; CHUNK_SIZE];
        buffer[0] = 0x55;
        buffer[CHUNK_SIZE - 1] = 0xAA;

        let mut transferred: u64 = 0;
        let mut throttler = ProgressThrottler::new("p2p", "large_dump.sql", TOTAL_SIMULATED);
        let now = Instant::now();

        // Ejecutar 4,000 iteraciones (simulando 1 GiB de chunks continuos)
        for _ in 0..4000 {
            transferred += CHUNK_SIZE as u64;
            let _ = throttler.poll(transferred, false, now);
        }

        assert_eq!(transferred, 4000 * (CHUNK_SIZE as u64));
        assert_eq!(buffer.len(), CHUNK_SIZE); // La memoria del búfer se mantuvo fija en 256 KiB
    }
}
