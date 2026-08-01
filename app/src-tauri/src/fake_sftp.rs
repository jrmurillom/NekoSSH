//! Fake SFTP in-process para tests/harness (cero writes a hosts SSH reales).

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::edit_util::{
    content_fingerprint, exceeds_edit_size_limit, looks_binary, MAX_EXTERNAL_EDIT_BYTES,
};

/// Almacén remoto simulado (path → bytes). Solo memoria/local.
#[derive(Clone, Default)]
pub struct FakeSftpStore {
    inner: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl FakeSftpStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, remote_path: &str, data: impl Into<Vec<u8>>) {
        self.inner
            .lock()
            .unwrap()
            .insert(remote_path.to_string(), data.into());
    }

    pub fn get(&self, remote_path: &str) -> Option<Vec<u8>> {
        self.inner.lock().unwrap().get(remote_path).cloned()
    }

    pub fn file_size(&self, remote_path: &str) -> Result<u64, String> {
        self.inner
            .lock()
            .unwrap()
            .get(remote_path)
            .map(|d| d.len() as u64)
            .ok_or_else(|| format!("Path remoto inexistente: {}", remote_path))
    }

    pub fn sample(&self, remote_path: &str, max_bytes: usize) -> Result<Vec<u8>, String> {
        let data = self
            .inner
            .lock()
            .unwrap()
            .get(remote_path)
            .cloned()
            .ok_or_else(|| format!("Path remoto inexistente: {}", remote_path))?;
        Ok(data.into_iter().take(max_bytes).collect())
    }

    pub fn download_to_local(&self, remote_path: &str, local_path: &Path) -> Result<(), String> {
        let data = self
            .inner
            .lock()
            .unwrap()
            .get(remote_path)
            .cloned()
            .ok_or_else(|| format!("Path remoto inexistente: {}", remote_path))?;
        if exceeds_edit_size_limit(data.len() as u64) {
            return Err(format!(
                "El archivo supera el límite de {} bytes para edición externa",
                MAX_EXTERNAL_EDIT_BYTES
            ));
        }
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(local_path, &data).map_err(|e| format!("Error al escribir temp: {}", e))?;
        Ok(())
    }

    pub fn upload_from_local(&self, local_path: &Path, remote_path: &str) -> Result<(), String> {
        let data = std::fs::read(local_path).map_err(|e| format!("Error al leer local: {}", e))?;
        self.inner
            .lock()
            .unwrap()
            .insert(remote_path.to_string(), data);
        Ok(())
    }

    /// Simula denegación de escritura en paths concretos (mock de permisos).
    pub fn upload_from_local_denying(
        &self,
        local_path: &Path,
        remote_path: &str,
        deny_prefixes: &[&str],
    ) -> Result<(), String> {
        if deny_prefixes
            .iter()
            .any(|p| remote_path == *p || remote_path.starts_with(&format!("{}/", p)))
        {
            return Err(format!(
                "Error al crear remoto {}: Permission denied",
                remote_path
            ));
        }
        self.upload_from_local(local_path, remote_path)
    }

    pub fn unlink(&self, remote_path: &str) {
        self.inner.lock().unwrap().remove(remote_path);
    }

    /// Copia bytes de un path remoto a otro (mock de `cp` exitoso).
    pub fn copy_remote(&self, from: &str, to: &str) -> Result<(), String> {
        let data = self
            .get(from)
            .ok_or_else(|| format!("Path remoto inexistente: {}", from))?;
        self.inner.lock().unwrap().insert(to.to_string(), data);
        Ok(())
    }

    pub fn probe(
        &self,
        remote_path: &str,
    ) -> Result<ProbeResult, String> {
        let size = self.file_size(remote_path)?;
        let sample = self.sample(remote_path, 4096)?;
        Ok(ProbeResult {
            size,
            too_large: exceeds_edit_size_limit(size),
            looks_binary: looks_binary(&sample),
            fingerprint: content_fingerprint(&self.get(remote_path).unwrap_or_default()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeResult {
    pub size: u64,
    pub too_large: bool,
    pub looks_binary: bool,
    pub fingerprint: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit_util::MAX_EXTERNAL_EDIT_BYTES;
    use std::fs;

    #[test]
    fn fake_download_upload_round_trip_local() {
        let store = FakeSftpStore::new();
        store.insert("/etc/hosts", b"127.0.0.1 localhost\n");
        let tmp = std::env::temp_dir().join(format!("nekossh-fake-dl-{}", std::process::id()));
        let _ = fs::remove_file(&tmp);
        store.download_to_local("/etc/hosts", &tmp).expect("download");
        assert_eq!(fs::read(&tmp).unwrap(), b"127.0.0.1 localhost\n");
        fs::write(&tmp, b"modified\n").unwrap();
        store
            .upload_from_local(&tmp, "/etc/hosts")
            .expect("upload");
        assert_eq!(store.get("/etc/hosts").unwrap(), b"modified\n");
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn fake_rechaza_archivo_grande() {
        let store = FakeSftpStore::new();
        let big = vec![b'x'; (MAX_EXTERNAL_EDIT_BYTES as usize) + 8];
        store.insert("/big.bin", big);
        let tmp = std::env::temp_dir().join(format!("nekossh-fake-big-{}", std::process::id()));
        let err = store.download_to_local("/big.bin", &tmp).unwrap_err();
        assert!(err.contains("límite") || err.contains("supera"));
        let probe = store.probe("/big.bin").unwrap();
        assert!(probe.too_large);
    }

    #[test]
    fn fake_detecta_binario() {
        let store = FakeSftpStore::new();
        store.insert("/a.bin", b"MZ\0\0pe");
        let probe = store.probe("/a.bin").unwrap();
        assert!(probe.looks_binary);
        assert!(!probe.too_large);
    }

    #[test]
    fn fake_deny_permisos_y_copy_unlink() {
        let store = FakeSftpStore::new();
        let tmp = std::env::temp_dir().join(format!("nekossh-fake-deny-{}", std::process::id()));
        fs::write(&tmp, b"data\n").unwrap();
        let err = store
            .upload_from_local_denying(&tmp, "/etc/hosts", &["/etc"])
            .unwrap_err();
        assert!(err.to_ascii_lowercase().contains("permission denied"));
        store.upload_from_local(&tmp, "/tmp/nekossh-x").unwrap();
        store.copy_remote("/tmp/nekossh-x", "/etc/hosts").unwrap();
        assert_eq!(store.get("/etc/hosts").unwrap(), b"data\n");
        store.unlink("/tmp/nekossh-x");
        assert!(store.get("/tmp/nekossh-x").is_none());
        let _ = fs::remove_file(&tmp);
    }
}
