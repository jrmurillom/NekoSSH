//! Helpers de edición externa: temp paths, límite de tamaño, heurística binaria.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

/// Código determinista devuelto cuando una transferencia es cancelada por usuario, cierre de sesión o salida de app.
pub const TRANSFER_CANCELLED_ERROR: &str = "TRANSFER_CANCELLED";

#[derive(Clone)]
pub struct ActiveTransferEntry {
    pub terminal_ids: Vec<String>,
    pub local_part_path: Option<PathBuf>,
    pub cancel_flag: Arc<AtomicBool>,
}

#[derive(Clone, Default)]
pub struct ActiveTransferRegistry {
    next_id: Arc<AtomicU64>,
    entries: Arc<Mutex<HashMap<u64, ActiveTransferEntry>>>,
}

pub struct ActiveTransferGuard {
    id: u64,
    registry: ActiveTransferRegistry,
    pub cancel_flag: Arc<AtomicBool>,
}

impl ActiveTransferGuard {
    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::Acquire)
    }

    pub fn cancel_flag(&self) -> &AtomicBool {
        &self.cancel_flag
    }
}

impl Drop for ActiveTransferGuard {
    fn drop(&mut self) {
        if let Ok(mut map) = self.registry.entries.lock() {
            map.remove(&self.id);
        }
    }
}

impl ActiveTransferRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &self,
        terminal_ids: Vec<String>,
        local_part_path: Option<PathBuf>,
    ) -> ActiveTransferGuard {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let entry = ActiveTransferEntry {
            terminal_ids,
            local_part_path,
            cancel_flag: cancel_flag.clone(),
        };
        if let Ok(mut map) = self.entries.lock() {
            map.insert(id, entry);
        }
        ActiveTransferGuard {
            id,
            registry: self.clone(),
            cancel_flag,
        }
    }

    pub fn active_count(&self) -> usize {
        self.entries.lock().map(|m| m.len()).unwrap_or(0)
    }

    /// Señaliza cancelación para todas las transferencias activas.
    pub fn cancel_all(&self) -> usize {
        let Ok(map) = self.entries.lock() else {
            return 0;
        };
        for entry in map.values() {
            entry.cancel_flag.store(true, Ordering::Release);
        }
        map.len()
    }

    /// Señaliza cancelación para toda transferencia donde participe `terminal_id` (origen o destino).
    pub fn cancel_for_terminal(&self, terminal_id: &str) -> usize {
        let Ok(map) = self.entries.lock() else {
            return 0;
        };
        let mut count = 0;
        for entry in map.values() {
            if entry.terminal_ids.iter().any(|t| t == terminal_id) {
                entry.cancel_flag.store(true, Ordering::Release);
                count += 1;
            }
        }
        count
    }

    /// Señaliza cancelación global y elimina síncronamente cualquier `.nekossh.part` local registrado.
    pub fn cancel_all_and_cleanup_local_parts(&self) {
        let local_parts: Vec<PathBuf> = {
            let Ok(map) = self.entries.lock() else {
                return;
            };
            for entry in map.values() {
                entry.cancel_flag.store(true, Ordering::Release);
            }
            map.values()
                .filter_map(|e| e.local_part_path.clone())
                .collect()
        };

        if local_parts.is_empty() {
            return;
        }

        for _ in 0..12 {
            if self.active_count() == 0 {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        for part in local_parts {
            let _ = std::fs::remove_file(&part);
        }
    }
}

/// Límite por defecto para edición externa (10 MiB).
pub const MAX_EXTERNAL_EDIT_BYTES: u64 = 10 * 1024 * 1024;

/// Tamaño del búfer fijo en streaming para subidas y copias SFTP (64 KiB, memoria O(1)).
pub const SFTP_STREAM_CHUNK_BYTES: usize = 64 * 1024;

/// Máximo de reintentos consecutivos ante WouldBlock sin transferir bytes (~2s a 5ms por intento).
pub const MAX_WOULD_BLOCK_ATTEMPTS: usize = 400;

/// Intervalo mínimo de throttling para emisión de progreso por canal IPC (100 ms).
pub const PROGRESS_THROTTLE_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferProgressPayload {
    pub operation: String,
    pub file_name: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
    pub percent: u8,
    pub speed_bps: u64,
}

/// Calcula porcentaje seguro en u64 (usando u128 intermedio para evitar overflow en archivos >4 GiB).
/// Si total_bytes == 0, retorna 100 para evitar división por cero.
pub fn calculate_transfer_percent(bytes_transferred: u64, total_bytes: u64) -> u8 {
    if total_bytes == 0 || bytes_transferred >= total_bytes {
        return 100;
    }
    let pct = (bytes_transferred as u128 * 100) / (total_bytes as u128);
    pct.min(100) as u8
}

/// Calcula la velocidad en bytes por segundo de forma segura sin división por cero.
pub fn calculate_speed_bps(bytes_transferred: u64, elapsed: Duration) -> u64 {
    let secs = elapsed.as_secs_f64();
    if secs <= 0.000_1 || bytes_transferred == 0 {
        return 0;
    }
    (bytes_transferred as f64 / secs).round() as u64
}

/// Genera la ruta remota temporal oculta en el mismo directorio destino (`.<nombre>.nekossh.part`).
pub fn build_staging_part_path(remote_path: &str) -> String {
    let trimmed = remote_path.trim_end_matches('/');
    let base = remote_basename(trimmed);
    if let Some(idx) = trimmed.rfind('/') {
        let dir = &trimmed[..idx];
        if dir.is_empty() {
            format!("/.{}.nekossh.part", base)
        } else {
            format!("{}/.{}.nekossh.part", dir, base)
        }
    } else {
        format!(".{}.nekossh.part", base)
    }
}

/// Genera la ruta local temporal oculta en el mismo directorio destino (`.<nombre>.nekossh.part`).
pub fn build_local_staging_part_path(local_path: &Path) -> PathBuf {
    let file_name = local_path
        .file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("download");
    let part_name = format!(".{}.nekossh.part", file_name);
    if let Some(parent) = local_path.parent() {
        if !parent.as_os_str().is_empty() {
            return parent.join(part_name);
        }
    }
    PathBuf::from(part_name)
}

/// Controlador de throttling temporal para emisiones de progreso.
pub struct ProgressThrottler {
    operation: String,
    file_name: String,
    total_bytes: u64,
    start_time: Instant,
    last_emit: Option<Instant>,
    min_interval: Duration,
}

impl ProgressThrottler {
    pub fn new(operation: impl Into<String>, file_name: impl Into<String>, total_bytes: u64) -> Self {
        Self::with_start(
            operation,
            file_name,
            total_bytes,
            Instant::now(),
            PROGRESS_THROTTLE_INTERVAL,
        )
    }

    pub fn with_start(
        operation: impl Into<String>,
        file_name: impl Into<String>,
        total_bytes: u64,
        start_time: Instant,
        min_interval: Duration,
    ) -> Self {
        Self {
            operation: operation.into(),
            file_name: file_name.into(),
            total_bytes,
            start_time,
            last_emit: None,
            min_interval,
        }
    }

    pub fn poll(
        &mut self,
        bytes_transferred: u64,
        force: bool,
        now: Instant,
    ) -> Option<TransferProgressPayload> {
        let should_send = force
            || match self.last_emit {
                None => true,
                Some(prev) => now.saturating_duration_since(prev) >= self.min_interval,
            };
        if !should_send {
            return None;
        }
        self.last_emit = Some(now);
        let elapsed = now.saturating_duration_since(self.start_time);
        let percent = if bytes_transferred == 0 && self.total_bytes > 0 && !force {
            0
        } else if bytes_transferred == 0 && self.total_bytes > 0 && force {
            0
        } else {
            calculate_transfer_percent(bytes_transferred, self.total_bytes)
        };
        Some(TransferProgressPayload {
            operation: self.operation.clone(),
            file_name: self.file_name.clone(),
            bytes_transferred,
            total_bytes: self.total_bytes,
            percent,
            speed_bps: calculate_speed_bps(bytes_transferred, elapsed),
        })
    }
}

/// TTL de temps huérfanos al sweep de startup (24 h).
pub const ORPHAN_TEMP_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Debounce de watcher antes de emitir cambio real (~750 ms).
pub const EDIT_WATCH_DEBOUNCE_MS: u64 = 750;

pub fn exceeds_edit_size_limit(size: u64) -> bool {
    size > MAX_EXTERNAL_EDIT_BYTES
}

/// Heurística ligera: NUL en la muestra inicial ⇒ probablemente binario.
pub fn looks_binary(sample: &[u8]) -> bool {
    sample.iter().any(|&b| b == 0)
}

pub fn edit_sessions_root(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("edit-sessions")
}

pub fn edit_session_dir(app_data_dir: &Path, edit_id: &str) -> PathBuf {
    edit_sessions_root(app_data_dir).join(edit_id)
}

pub fn local_edit_file_path(session_dir: &Path, remote_basename: &str) -> PathBuf {
    let base = if remote_basename.is_empty() {
        "untitled"
    } else {
        remote_basename
    };
    // Evitar path separators en basename
    let safe: String = base
        .chars()
        .map(|c| if c == '/' || c == '\\' { '_' } else { c })
        .collect();
    session_dir.join(safe)
}

pub fn remote_basename(remote_path: &str) -> String {
    remote_path
        .rsplit(['/', '\\'])
        .find(|s| !s.is_empty())
        .unwrap_or("untitled")
        .to_string()
}

pub fn content_fingerprint(data: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    data.len().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn file_fingerprint(path: &Path) -> Result<String, String> {
    let data = std::fs::read(path).map_err(|e| format!("Error al leer temp: {}", e))?;
    Ok(content_fingerprint(&data))
}

/// Sweep best-effort de dirs huérfanos bajo `edit-sessions/` más viejos que TTL.
/// No falla la app si no puede borrar alguno.
pub fn sweep_orphan_edit_temps(app_data_dir: &Path, ttl: Duration, now: SystemTime) {
    let root = edit_sessions_root(app_data_dir);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let age = now.duration_since(modified).unwrap_or(Duration::ZERO);
        if age > ttl {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::SystemTime;

    #[test]
    fn rechazo_por_tamano_sobre_10_mib() {
        assert!(!exceeds_edit_size_limit(MAX_EXTERNAL_EDIT_BYTES));
        assert!(!exceeds_edit_size_limit(0));
        assert!(exceeds_edit_size_limit(MAX_EXTERNAL_EDIT_BYTES + 1));
    }

    #[test]
    fn heuristica_binaria_detecta_nul() {
        assert!(!looks_binary(b"hola mundo\n"));
        assert!(looks_binary(b"abc\0def"));
        assert!(!looks_binary(&[]));
    }

    #[test]
    fn paths_temp_edit_session() {
        let root = PathBuf::from("/tmp/appdata");
        let dir = edit_session_dir(&root, "abc-123");
        assert_eq!(dir, PathBuf::from("/tmp/appdata/edit-sessions/abc-123"));
        let file = local_edit_file_path(&dir, "config.yaml");
        assert_eq!(
            file,
            PathBuf::from("/tmp/appdata/edit-sessions/abc-123/config.yaml")
        );
        let nested = local_edit_file_path(&dir, "a/b.txt");
        assert_eq!(nested.file_name().unwrap(), "a_b.txt");
    }

    #[test]
    fn fingerprint_cambia_con_contenido() {
        assert_ne!(content_fingerprint(b"a"), content_fingerprint(b"b"));
        assert_eq!(content_fingerprint(b"x"), content_fingerprint(b"x"));
    }

    #[test]
    fn sweep_borra_huerfanos_viejos() {
        let tmp = std::env::temp_dir().join(format!(
            "nekossh-edit-sweep-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(edit_sessions_root(&tmp)).unwrap();
        let old_dir = edit_session_dir(&tmp, "old");
        fs::create_dir_all(&old_dir).unwrap();
        fs::write(old_dir.join("f.txt"), b"x").unwrap();

        // TTL cero ⇒ cualquier dir existente es huérfano
        sweep_orphan_edit_temps(&tmp, Duration::ZERO, SystemTime::now() + Duration::from_secs(2));
        assert!(!old_dir.exists());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn calculo_porcentaje_seguro_cero_bytes_y_archivos_grandes() {
        // Archivo de 0 bytes -> 100% sin división por cero
        assert_eq!(calculate_transfer_percent(0, 0), 100);
        // Inicio de archivo no vacío -> 0%
        assert_eq!(calculate_transfer_percent(0, 1000), 0);
        // Mitad -> 50%
        assert_eq!(calculate_transfer_percent(500, 1000), 50);
        // Completo -> 100%
        assert_eq!(calculate_transfer_percent(1000, 1000), 100);
        // Archivo > 4 GiB (ej. 10 GiB) al 25%
        let ten_gib: u64 = 10 * 1024 * 1024 * 1024;
        let quarter: u64 = 2560 * 1024 * 1024;
        assert_eq!(calculate_transfer_percent(quarter, ten_gib), 25);
    }

    #[test]
    fn generacion_ruta_staging_part() {
        assert_eq!(
            build_staging_part_path("/var/www/backup.tar.gz"),
            "/var/www/.backup.tar.gz.nekossh.part"
        );
        assert_eq!(
            build_staging_part_path("/backup.tar.gz"),
            "/.backup.tar.gz.nekossh.part"
        );
        assert_eq!(
            build_staging_part_path("backup.tar.gz"),
            ".backup.tar.gz.nekossh.part"
        );
    }

    #[test]
    fn generacion_ruta_staging_part_local() {
        let local = PathBuf::from("/Users/test/Downloads/prod-db.sql.gz");
        assert_eq!(
            build_local_staging_part_path(&local),
            PathBuf::from("/Users/test/Downloads/.prod-db.sql.gz.nekossh.part")
        );
        let relative = PathBuf::from("prod-db.sql.gz");
        assert_eq!(
            build_local_staging_part_path(&relative),
            PathBuf::from(".prod-db.sql.gz.nekossh.part")
        );
    }

    #[test]
    fn throttling_temporal_100ms_y_emision_forzada() {
        let t0 = Instant::now();
        let mut throttler =
            ProgressThrottler::with_start("upload", "data.iso", 1000, t0, Duration::from_millis(100));

        // Emisión inicial forzada (0%)
        let first = throttler.poll(0, true, t0).expect("debe emitir inicio");
        assert_eq!(first.percent, 0);
        assert_eq!(first.bytes_transferred, 0);

        // A los 40ms sin force -> bloqueado por throttling
        assert!(throttler
            .poll(200, false, t0 + Duration::from_millis(40))
            .is_none());

        // A los 110ms -> permitido (20%)
        let mid = throttler
            .poll(200, false, t0 + Duration::from_millis(110))
            .expect("debe emitir tras 100ms");
        assert_eq!(mid.percent, 20);
        assert!(mid.speed_bps > 0);

        // A los 130ms pero con force=true (100% final) -> emitido inmediatamente
        let end = throttler
            .poll(1000, true, t0 + Duration::from_millis(130))
            .expect("debe emitir final forzado");
        assert_eq!(end.percent, 100);
        assert_eq!(end.bytes_transferred, 1000);
    }

    #[test]
    fn registro_raii_y_cancelacion_por_terminal_y_exit() {
        let reg = ActiveTransferRegistry::new();
        let tmp_part = std::env::temp_dir().join(format!(
            ".nekossh-exit-cleanup-{}.nekossh.part",
            std::process::id()
        ));
        fs::write(&tmp_part, b"partial-bytes").unwrap();
        assert!(tmp_part.exists());

        // 1. Registrar una descarga en "term-A" y una copia SCP entre "term-B" y "term-C"
        let guard_dl = reg.register(vec!["term-A".to_string()], Some(tmp_part.clone()));
        let guard_scp = reg.register(
            vec!["term-B".to_string(), "term-C".to_string()],
            None,
        );
        assert_eq!(reg.active_count(), 2);
        assert!(!guard_dl.is_cancelled());
        assert!(!guard_scp.is_cancelled());

        // 2. Cerrar "term-C" (destino de SCP) sólo cancela `guard_scp`, no `guard_dl`
        assert_eq!(reg.cancel_for_terminal("term-C"), 1);
        assert!(!guard_dl.is_cancelled());
        assert!(guard_scp.is_cancelled());

        // 3. Al caer `guard_scp` (Drop RAII), se desregistra automáticamente
        drop(guard_scp);
        assert_eq!(reg.active_count(), 1);

        // 4. Salida de app (`cancel_all_and_cleanup_local_parts`) cancela `guard_dl` y borra `tmp_part` del disco
        reg.cancel_all_and_cleanup_local_parts();
        assert!(guard_dl.is_cancelled());
        assert!(!tmp_part.exists());

        drop(guard_dl);
        assert_eq!(reg.active_count(), 0);
    }
}
