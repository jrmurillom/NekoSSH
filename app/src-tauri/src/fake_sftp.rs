//! Fake SFTP in-process para tests/harness (cero writes a hosts SSH reales).

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::edit_util::{
    build_local_staging_part_path, build_staging_part_path, content_fingerprint,
    exceeds_edit_size_limit, looks_binary, remote_basename, ProgressThrottler,
    TransferProgressPayload, MAX_EXTERNAL_EDIT_BYTES, SFTP_STREAM_CHUNK_BYTES,
    TRANSFER_CANCELLED_ERROR,
};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};

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

    /// Descarga en streaming de 64 KiB hacia ruta elegida por el usuario (sin límite de 10 MiB),
    /// escribiendo en archivo temporal local `.nekossh.part`, validando tamaño final y renombrando atómicamente.
    pub fn stream_download_to_local_with_progress(
        &self,
        remote_path: &str,
        local_path: &Path,
        fail_after_bytes: Option<u64>,
        mut on_progress: Option<&mut dyn FnMut(TransferProgressPayload)>,
    ) -> Result<(), String> {
        let data = self
            .get(remote_path)
            .ok_or_else(|| format!("Path remoto inexistente: {}", remote_path))?;
        let expected_size = data.len() as u64;
        let file_name = local_path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| remote_basename(remote_path));

        if let Some(parent) = local_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }

        let part_path = build_local_staging_part_path(local_path);
        let mut throttler = ProgressThrottler::new("download", &file_name, expected_size);

        if let Some(cb) = on_progress.as_deref_mut() {
            if let Some(p) = throttler.poll(0, true, std::time::Instant::now()) {
                cb(p);
            }
        }

        let mut part_file = std::fs::File::create(&part_path)
            .map_err(|e| format!("Error al crear archivo temporal local: {}", e))?;

        let mut bytes_transferred: u64 = 0;
        for chunk in data.chunks(SFTP_STREAM_CHUNK_BYTES) {
            if let Some(limit) = fail_after_bytes {
                if bytes_transferred + (chunk.len() as u64) > limit {
                    drop(part_file);
                    let _ = std::fs::remove_file(&part_path);
                    return Err("Fallo simulado de red a mitad de descarga".to_string());
                }
            }

            if let Err(e) = part_file.write_all(chunk) {
                drop(part_file);
                let _ = std::fs::remove_file(&part_path);
                return Err(format!("Error al escribir descarga local: {}", e));
            }

            bytes_transferred += chunk.len() as u64;
            if let Some(cb) = on_progress.as_deref_mut() {
                if let Some(p) = throttler.poll(bytes_transferred, false, std::time::Instant::now())
                {
                    cb(p);
                }
            }
        }

        if let Err(e) = part_file.flush() {
            drop(part_file);
            let _ = std::fs::remove_file(&part_path);
            return Err(format!("Error al vaciar búfer local: {}", e));
        }
        drop(part_file);

        let written_size = std::fs::metadata(&part_path)
            .map(|m| m.len())
            .map_err(|e| {
                let _ = std::fs::remove_file(&part_path);
                format!("Error al verificar archivo temporal local: {}", e)
            })?;

        if written_size < expected_size {
            let _ = std::fs::remove_file(&part_path);
            return Err(format!(
                "Descarga truncada o incompleta: tamaño local ({} bytes) < esperado ({} bytes)",
                written_size, expected_size
            ));
        }

        if let Err(e) = std::fs::rename(&part_path, local_path) {
            let _ = std::fs::remove_file(&part_path);
            return Err(format!("Error al renombrar descarga final: {}", e));
        }

        if let Some(cb) = on_progress.as_deref_mut() {
            if let Some(p) = throttler.poll(bytes_transferred, true, std::time::Instant::now()) {
                cb(p);
            }
        }

        Ok(())
    }

    /// Variante de prueba que permite simular un `stat.size` inicial distinto al tamaño real al llegar a EOF
    /// (ej. un archivo de log activo que creció durante la descarga, o un archivo truncado).
    pub fn stream_download_to_local_with_custom_stat(
        &self,
        remote_path: &str,
        local_path: &Path,
        initial_stat_size: u64,
    ) -> Result<(), String> {
        let data = self
            .get(remote_path)
            .ok_or_else(|| format!("Path remoto inexistente: {}", remote_path))?;
        let part_path = build_local_staging_part_path(local_path);
        if let Some(parent) = local_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }
        let mut part_file = std::fs::File::create(&part_path).map_err(|e| e.to_string())?;
        for chunk in data.chunks(SFTP_STREAM_CHUNK_BYTES) {
            part_file.write_all(chunk).map_err(|e| {
                let _ = std::fs::remove_file(&part_path);
                e.to_string()
            })?;
        }
        part_file.flush().map_err(|e| e.to_string())?;
        drop(part_file);

        let written_size = std::fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0);
        if written_size < initial_stat_size {
            let _ = std::fs::remove_file(&part_path);
            return Err(format!(
                "Descarga truncada o incompleta: recibido {} bytes, esperado al menos {} bytes",
                written_size, initial_stat_size
            ));
        }
        std::fs::rename(&part_path, local_path).map_err(|e| {
            let _ = std::fs::remove_file(&part_path);
            e.to_string()
        })?;
        Ok(())
    }

    pub fn stream_download_cancellable(
        &self,
        remote_path: &str,
        local_path: &Path,
        cancel_flag: &AtomicBool,
        mut on_progress: Option<&mut dyn FnMut(TransferProgressPayload)>,
    ) -> Result<(), String> {
        let data = self
            .get(remote_path)
            .ok_or_else(|| format!("Path remoto inexistente: {}", remote_path))?;
        let part_path = build_local_staging_part_path(local_path);
        let mut throttler = ProgressThrottler::new(
            "download",
            remote_basename(remote_path),
            data.len() as u64,
        );
        let mut part_file = std::fs::File::create(&part_path).map_err(|e| e.to_string())?;
        let mut bytes_transferred: u64 = 0;

        for chunk in data.chunks(SFTP_STREAM_CHUNK_BYTES) {
            if cancel_flag.load(Ordering::Acquire) {
                drop(part_file);
                let _ = std::fs::remove_file(&part_path);
                return Err(TRANSFER_CANCELLED_ERROR.to_string());
            }
            part_file.write_all(chunk).map_err(|e| e.to_string())?;
            bytes_transferred += chunk.len() as u64;
            if let Some(cb) = on_progress.as_deref_mut() {
                if let Some(p) = throttler.poll(bytes_transferred, true, std::time::Instant::now()) {
                    cb(p);
                }
            }
        }
        drop(part_file);
        std::fs::rename(&part_path, local_path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn stream_upload_cancellable(
        &self,
        local_path: &Path,
        remote_path: &str,
        cancel_flag: &AtomicBool,
        mut on_progress: Option<&mut dyn FnMut(TransferProgressPayload)>,
    ) -> Result<(), String> {
        let mut file = std::fs::File::open(local_path).map_err(|e| e.to_string())?;
        let expected_size = file.metadata().map_err(|e| e.to_string())?.len();
        let part_path = build_staging_part_path(remote_path);
        let mut throttler =
            ProgressThrottler::new("upload", remote_basename(remote_path), expected_size);
        self.insert(&part_path, Vec::new());

        let mut buffer = [0u8; SFTP_STREAM_CHUNK_BYTES];
        let mut bytes_transferred: u64 = 0;
        loop {
            if cancel_flag.load(Ordering::Acquire) {
                self.unlink(&part_path);
                return Err(TRANSFER_CANCELLED_ERROR.to_string());
            }
            let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            {
                let mut map = self.inner.lock().unwrap();
                if let Some(vec) = map.get_mut(&part_path) {
                    vec.extend_from_slice(&buffer[..n]);
                }
            }
            bytes_transferred += n as u64;
            if let Some(cb) = on_progress.as_deref_mut() {
                if let Some(p) = throttler.poll(bytes_transferred, true, std::time::Instant::now()) {
                    cb(p);
                }
            }
        }
        let part_data = self.get(&part_path).unwrap_or_default();
        self.insert(remote_path, part_data);
        self.unlink(&part_path);
        Ok(())
    }

    pub fn stream_copy_cancellable(
        &self,
        source_path: &str,
        target_store: &FakeSftpStore,
        target_path: &str,
        cancel_flag: &AtomicBool,
        mut on_progress: Option<&mut dyn FnMut(TransferProgressPayload)>,
    ) -> Result<(), String> {
        let src_data = self
            .get(source_path)
            .ok_or_else(|| format!("Path remoto inexistente: {}", source_path))?;
        let part_path = build_staging_part_path(target_path);
        let mut throttler =
            ProgressThrottler::new("scp", remote_basename(target_path), src_data.len() as u64);
        target_store.insert(&part_path, Vec::new());

        let mut bytes_transferred: u64 = 0;
        for chunk in src_data.chunks(SFTP_STREAM_CHUNK_BYTES) {
            if cancel_flag.load(Ordering::Acquire) {
                target_store.unlink(&part_path);
                return Err(TRANSFER_CANCELLED_ERROR.to_string());
            }
            {
                let mut map = target_store.inner.lock().unwrap();
                if let Some(vec) = map.get_mut(&part_path) {
                    vec.extend_from_slice(chunk);
                }
            }
            bytes_transferred += chunk.len() as u64;
            if let Some(cb) = on_progress.as_deref_mut() {
                if let Some(p) = throttler.poll(bytes_transferred, true, std::time::Instant::now()) {
                    cb(p);
                }
            }
        }
        let final_data = target_store.get(&part_path).unwrap_or_default();
        target_store.insert(target_path, final_data);
        target_store.unlink(&part_path);
        Ok(())
    }

    pub fn upload_from_local(&self, local_path: &Path, remote_path: &str) -> Result<(), String> {
        self.stream_upload_from_local_with_progress(local_path, remote_path, None, None)
    }

    /// Sube en streaming de 64 KiB con archivo temporal `.nekossh.part`, verificación `stat`,
    /// renombrado atómico, limpieza `.part` en fallo y emisión de progreso.
    pub fn stream_upload_from_local_with_progress(
        &self,
        local_path: &Path,
        remote_path: &str,
        fail_after_bytes: Option<u64>,
        mut on_progress: Option<&mut dyn FnMut(TransferProgressPayload)>,
    ) -> Result<(), String> {
        let mut file = std::fs::File::open(local_path)
            .map_err(|e| format!("Error al leer local: {}", e))?;
        let expected_size = file
            .metadata()
            .map_err(|e| format!("Error al leer metadatos locales: {}", e))?
            .len();

        let file_name = remote_basename(remote_path);
        let part_path = build_staging_part_path(remote_path);
        let mut throttler = ProgressThrottler::new("upload", &file_name, expected_size);

        if let Some(cb) = on_progress.as_deref_mut() {
            if let Some(p) = throttler.poll(0, true, std::time::Instant::now()) {
                cb(p);
            }
        }

        // Crear archivo temporal `.part` vacío en el store
        self.insert(&part_path, Vec::new());

        let mut buffer = [0u8; SFTP_STREAM_CHUNK_BYTES];
        let mut bytes_transferred: u64 = 0;

        loop {
            let n = match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    self.unlink(&part_path);
                    return Err(format!("Error al leer local: {}", e));
                }
            };

            if let Some(limit) = fail_after_bytes {
                if bytes_transferred + (n as u64) > limit {
                    self.unlink(&part_path);
                    return Err("Fallo simulado de red a mitad de subida".to_string());
                }
            }

            {
                let mut map = self.inner.lock().unwrap();
                if let Some(vec) = map.get_mut(&part_path) {
                    vec.extend_from_slice(&buffer[..n]);
                }
            }

            bytes_transferred += n as u64;
            if let Some(cb) = on_progress.as_deref_mut() {
                if let Some(p) = throttler.poll(bytes_transferred, false, std::time::Instant::now()) {
                    cb(p);
                }
            }
        }

        let remote_size = match self.file_size(&part_path) {
            Ok(s) => s,
            Err(e) => {
                self.unlink(&part_path);
                return Err(e);
            }
        };

        if remote_size != expected_size {
            self.unlink(&part_path);
            return Err(format!(
                "Discrepancia de integridad en subida: tamaño remoto ({} bytes) no coincide con local ({} bytes)",
                remote_size, expected_size
            ));
        }

        // Rename atómico .part -> remote_path
        let part_data = self.get(&part_path).unwrap_or_default();
        self.insert(remote_path, part_data);
        self.unlink(&part_path);

        if let Some(cb) = on_progress.as_deref_mut() {
            if let Some(p) = throttler.poll(bytes_transferred, true, std::time::Instant::now()) {
                cb(p);
            }
        }

        Ok(())
    }

    /// Sube simulando verificación de integridad de tamaño (stat.size == local.len()).
    pub fn upload_from_local_with_integrity(
        &self,
        local_path: &Path,
        remote_path: &str,
    ) -> Result<(), String> {
        self.stream_upload_from_local_with_progress(local_path, remote_path, None, None)
    }

    /// Copia entre stores (simulando `sftp_copy_between_sessions`) con staging `.nekossh.part` y progreso.
    pub fn stream_copy_to_store_with_progress(
        &self,
        source_path: &str,
        target_store: &FakeSftpStore,
        target_path: &str,
        fail_after_bytes: Option<u64>,
        mut on_progress: Option<&mut dyn FnMut(TransferProgressPayload)>,
    ) -> Result<(), String> {
        let src_data = self
            .get(source_path)
            .ok_or_else(|| format!("Path remoto inexistente: {}", source_path))?;
        let total_bytes = src_data.len() as u64;
        let file_name = remote_basename(target_path);
        let part_path = build_staging_part_path(target_path);
        let mut throttler = ProgressThrottler::new("scp", &file_name, total_bytes);

        if let Some(cb) = on_progress.as_deref_mut() {
            if let Some(p) = throttler.poll(0, true, std::time::Instant::now()) {
                cb(p);
            }
        }

        target_store.insert(&part_path, Vec::new());
        let mut bytes_transferred: u64 = 0;

        for chunk in src_data.chunks(SFTP_STREAM_CHUNK_BYTES) {
            if let Some(limit) = fail_after_bytes {
                if bytes_transferred + (chunk.len() as u64) > limit {
                    target_store.unlink(&part_path);
                    return Err("Fallo simulado en copia SCP inter-sesión".to_string());
                }
            }
            {
                let mut map = target_store.inner.lock().unwrap();
                if let Some(vec) = map.get_mut(&part_path) {
                    vec.extend_from_slice(chunk);
                }
            }
            bytes_transferred += chunk.len() as u64;
            if let Some(cb) = on_progress.as_deref_mut() {
                if let Some(p) = throttler.poll(bytes_transferred, false, std::time::Instant::now()) {
                    cb(p);
                }
            }
        }

        let target_size = target_store.file_size(&part_path)?;
        if target_size != total_bytes {
            target_store.unlink(&part_path);
            return Err("Discrepancia de integridad en copia SCP".to_string());
        }

        let final_data = target_store.get(&part_path).unwrap_or_default();
        target_store.insert(target_path, final_data);
        target_store.unlink(&part_path);

        if let Some(cb) = on_progress.as_deref_mut() {
            if let Some(p) = throttler.poll(bytes_transferred, true, std::time::Instant::now()) {
                cb(p);
            }
        }

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
    fn fake_upload_binary_zip_intact() {
        let store = FakeSftpStore::new();
        // Simular un archivo zip con cabecera PK\x03\x04 y end of central directory PK\x05\x06 y bytes nulos
        let mut zip_bytes = Vec::new();
        zip_bytes.extend_from_slice(b"PK\x03\x04\x14\x00\x00\x00\x08\x00"); // Local file header
        zip_bytes.extend_from_slice(&[0u8; 64]); // Binary payload con NULs
        zip_bytes.extend_from_slice(b"PK\x01\x02\x14\x00\x14\x00"); // Central directory
        zip_bytes.extend_from_slice(b"PK\x05\x06\x00\x00\x00\x00"); // End of central directory

        let tmp = std::env::temp_dir().join(format!("nekossh-fake-zip-{}", std::process::id()));
        fs::write(&tmp, &zip_bytes).unwrap();

        store
            .upload_from_local_with_integrity(&tmp, "/remote/archive.zip")
            .expect("upload zip con integridad");

        let stored = store.get("/remote/archive.zip").expect("archivo en store");
        assert_eq!(stored, zip_bytes);
        assert_eq!(store.file_size("/remote/archive.zip").unwrap(), zip_bytes.len() as u64);

        let probe = store.probe("/remote/archive.zip").unwrap();
        assert!(probe.looks_binary);
        assert!(!probe.too_large);

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

    #[test]
    fn fake_streaming_upload_emite_progreso_y_limpia_part_en_fallo() {
        let store = FakeSftpStore::new();
        // Pre-poblar un archivo existente en destino para verificar que no se corrompe si falla la subida
        store.insert("/var/www/app.bin", b"version-estable-anterior");

        let payload = vec![b'Z'; 150 * 1024]; // 150 KiB (> 2 chunks de 64 KiB)
        let tmp = std::env::temp_dir().join(format!("nekossh-stream-up-{}", std::process::id()));
        fs::write(&tmp, &payload).unwrap();

        // 1. Simular fallo a los 70 KiB: debe limpiar `.app.bin.nekossh.part` y preservar `version-estable-anterior`
        let mut fail_events = Vec::new();
        let mut fail_cb = |ev: TransferProgressPayload| fail_events.push(ev);
        let err = store
            .stream_upload_from_local_with_progress(
                &tmp,
                "/var/www/app.bin",
                Some(70 * 1024),
                Some(&mut fail_cb),
            )
            .unwrap_err();
        assert!(err.contains("Fallo simulado"));
        assert!(store.get("/var/www/.app.bin.nekossh.part").is_none());
        assert_eq!(
            store.get("/var/www/app.bin").unwrap(),
            b"version-estable-anterior"
        );

        // 2. Subida completa exitosa: emite inicio 0% y final 100%, reemplaza destino y elimina `.part`
        let mut ok_events = Vec::new();
        let mut ok_cb = |ev: TransferProgressPayload| ok_events.push(ev);
        store
            .stream_upload_from_local_with_progress(&tmp, "/var/www/app.bin", None, Some(&mut ok_cb))
            .expect("subida exitosa");

        assert!(store.get("/var/www/.app.bin.nekossh.part").is_none());
        assert_eq!(store.get("/var/www/app.bin").unwrap().len(), payload.len());
        assert!(!ok_events.is_empty());
        assert_eq!(ok_events.first().unwrap().percent, 0);
        assert_eq!(ok_events.last().unwrap().percent, 100);
        assert_eq!(ok_events.last().unwrap().operation, "upload");

        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn fake_streaming_scp_copy_emite_progreso_y_limpia_part_en_fallo() {
        let src_store = FakeSftpStore::new();
        let tgt_store = FakeSftpStore::new();
        let data = vec![b'M'; 130 * 1024];
        src_store.insert("/home/user/dump.sql", data.clone());
        tgt_store.insert("/backups/dump.sql", b"backup-previo-intacto");

        // 1. Fallo intermedio en copia SCP
        let err = src_store
            .stream_copy_to_store_with_progress(
                "/home/user/dump.sql",
                &tgt_store,
                "/backups/dump.sql",
                Some(64 * 1024),
                None,
            )
            .unwrap_err();
        assert!(err.contains("Fallo simulado"));
        assert!(tgt_store.get("/backups/.dump.sql.nekossh.part").is_none());
        assert_eq!(
            tgt_store.get("/backups/dump.sql").unwrap(),
            b"backup-previo-intacto"
        );

        // 2. Copia SCP completa con eventos de progreso
        let mut scp_events = Vec::new();
        let mut scp_cb = |ev: TransferProgressPayload| scp_events.push(ev);
        src_store
            .stream_copy_to_store_with_progress(
                "/home/user/dump.sql",
                &tgt_store,
                "/backups/dump.sql",
                None,
                Some(&mut scp_cb),
            )
            .expect("copia scp exitosa");

        assert!(tgt_store.get("/backups/.dump.sql.nekossh.part").is_none());
        assert_eq!(tgt_store.get("/backups/dump.sql").unwrap(), data);
        assert_eq!(scp_events.first().unwrap().percent, 0);
        assert_eq!(scp_events.last().unwrap().percent, 100);
        assert_eq!(scp_events.last().unwrap().operation, "scp");
    }

    #[test]
    fn fake_streaming_download_emite_progreso_y_limpia_part_en_fallo() {
        let store = FakeSftpStore::new();
        // Archivo > 10 MiB (ej. 11 MiB) para demostrar que la descarga de usuario NO tiene el límite de edición externa
        let eleven_mib = (MAX_EXTERNAL_EDIT_BYTES as usize) + (1024 * 1024);
        let payload = vec![b'D'; eleven_mib];
        store.insert("/var/backups/full-dump.sql.gz", payload.clone());

        let tmp_dir = std::env::temp_dir().join(format!("nekossh-dl-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).unwrap();
        let target_local = tmp_dir.join("full-dump.sql.gz");
        let part_local = build_local_staging_part_path(&target_local);

        // Pre-poblar un archivo previo en la ruta local para verificar que no se destruye si la descarga falla a mitad
        fs::write(&target_local, b"version-local-anterior").unwrap();

        // 1. Mientras `download_to_local` (de Editar) rechaza >10 MiB, `stream_download_to_local_with_progress` sí lo permite
        assert!(store.download_to_local("/var/backups/full-dump.sql.gz", &target_local).is_err());

        // 2. Simular fallo de red a los 128 KiB: debe eliminar `.full-dump.sql.gz.nekossh.part` y dejar intacto `version-local-anterior`
        let err = store
            .stream_download_to_local_with_progress(
                "/var/backups/full-dump.sql.gz",
                &target_local,
                Some(128 * 1024),
                None,
            )
            .unwrap_err();
        assert!(err.contains("Fallo simulado"));
        assert!(!part_local.exists());
        assert_eq!(fs::read(&target_local).unwrap(), b"version-local-anterior");

        // 3. Descarga completa exitosa (>10 MiB): emite 0% y 100% con operation="download", renombra .part -> destino final
        let mut dl_events = Vec::new();
        let mut dl_cb = |ev: TransferProgressPayload| dl_events.push(ev);
        store
            .stream_download_to_local_with_progress(
                "/var/backups/full-dump.sql.gz",
                &target_local,
                None,
                Some(&mut dl_cb),
            )
            .expect("descarga exitosa >10 MiB");

        assert!(!part_local.exists());
        assert_eq!(fs::metadata(&target_local).unwrap().len(), eleven_mib as u64);
        assert!(!dl_events.is_empty());
        assert_eq!(dl_events.first().unwrap().percent, 0);
        assert_eq!(dl_events.last().unwrap().percent, 100);
        assert_eq!(dl_events.last().unwrap().operation, "download");

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn fake_streaming_download_soporta_log_en_crecimiento_y_rechaza_truncamiento() {
        let store = FakeSftpStore::new();
        store.insert("/var/log/nginx/access.log", vec![b'L'; 1500]);

        let tmp_dir =
            std::env::temp_dir().join(format!("nekossh-dl-live-log-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).unwrap();
        let target_local = tmp_dir.join("access.log");
        let part_local = build_local_staging_part_path(&target_local);

        // Caso A: Truncamiento (stat inicial era 2000 bytes, pero solo se recibieron 1500 bytes)
        // -> Debe fallar y eliminar `.access.log.nekossh.part`
        let err = store
            .stream_download_to_local_with_custom_stat(
                "/var/log/nginx/access.log",
                &target_local,
                2000,
            )
            .unwrap_err();
        assert!(err.contains("Descarga truncada"));
        assert!(!part_local.exists());
        assert!(!target_local.exists());

        // Caso B: Log vivo en crecimiento (stat inicial era 1000 bytes, y al llegar a EOF ya tenía 1500 bytes)
        // -> Debe tener éxito y guardar los 1500 bytes completos
        store
            .stream_download_to_local_with_custom_stat(
                "/var/log/nginx/access.log",
                &target_local,
                1000,
            )
            .expect("debe permitir descarga de archivo remoto en crecimiento");
        assert!(!part_local.exists());
        assert_eq!(fs::metadata(&target_local).unwrap().len(), 1500);

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn fake_cancelacion_en_tres_operaciones_limpia_part_inmediatamente() {
        let store = FakeSftpStore::new();
        let target_store = FakeSftpStore::new();
        let big_payload = vec![b'K'; 192 * 1024]; // 3 chunks de 64 KiB
        store.insert("/remote/source.iso", big_payload.clone());
        store.insert("/remote/upload-dest.iso", b"estable-remoto");
        target_store.insert("/scp/dest.iso", b"estable-scp");

        let tmp_dir =
            std::env::temp_dir().join(format!("nekossh-cancel-all3-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp_dir);
        fs::create_dir_all(&tmp_dir).unwrap();

        let local_src = tmp_dir.join("local-src.iso");
        fs::write(&local_src, &big_payload).unwrap();
        let local_dl_target = tmp_dir.join("downloaded.iso");
        fs::write(&local_dl_target, b"estable-local").unwrap();

        // 1. Cancelar DESCARGA en el primer tick de progreso (>0 bytes)
        let cancel_dl = Arc::new(AtomicBool::new(false));
        let cancel_dl_clone = cancel_dl.clone();
        let mut dl_cb = |ev: TransferProgressPayload| {
            if ev.bytes_transferred > 0 {
                cancel_dl_clone.store(true, Ordering::Release);
            }
        };
        let dl_err = store
            .stream_download_cancellable(
                "/remote/source.iso",
                &local_dl_target,
                &cancel_dl,
                Some(&mut dl_cb),
            )
            .unwrap_err();
        assert_eq!(dl_err, TRANSFER_CANCELLED_ERROR);
        assert!(!build_local_staging_part_path(&local_dl_target).exists());
        assert_eq!(fs::read(&local_dl_target).unwrap(), b"estable-local");

        // 2. Cancelar SUBIDA en el primer tick de progreso (>0 bytes)
        let cancel_up = Arc::new(AtomicBool::new(false));
        let cancel_up_clone = cancel_up.clone();
        let mut up_cb = |ev: TransferProgressPayload| {
            if ev.bytes_transferred > 0 {
                cancel_up_clone.store(true, Ordering::Release);
            }
        };
        let up_err = store
            .stream_upload_cancellable(
                &local_src,
                "/remote/upload-dest.iso",
                &cancel_up,
                Some(&mut up_cb),
            )
            .unwrap_err();
        assert_eq!(up_err, TRANSFER_CANCELLED_ERROR);
        assert!(store.get("/remote/.upload-dest.iso.nekossh.part").is_none());
        assert_eq!(store.get("/remote/upload-dest.iso").unwrap(), b"estable-remoto");

        // 3. Cancelar COPIA SCP en el primer tick de progreso (>0 bytes)
        let cancel_scp = Arc::new(AtomicBool::new(false));
        let cancel_scp_clone = cancel_scp.clone();
        let mut scp_cb = |ev: TransferProgressPayload| {
            if ev.bytes_transferred > 0 {
                cancel_scp_clone.store(true, Ordering::Release);
            }
        };
        let scp_err = store
            .stream_copy_cancellable(
                "/remote/source.iso",
                &target_store,
                "/scp/dest.iso",
                &cancel_scp,
                Some(&mut scp_cb),
            )
            .unwrap_err();
        assert_eq!(scp_err, TRANSFER_CANCELLED_ERROR);
        assert!(target_store.get("/scp/.dest.iso.nekossh.part").is_none());
        assert_eq!(target_store.get("/scp/dest.iso").unwrap(), b"estable-scp");

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
