//! Helpers de edición externa: temp paths, límite de tamaño, heurística binaria.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Límite por defecto para edición externa (10 MiB).
pub const MAX_EXTERNAL_EDIT_BYTES: u64 = 10 * 1024 * 1024;

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
}
