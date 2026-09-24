# Reporte Step 7: Verificación de Desktop Commands (IPC / SFTP)

**Fecha:** 2026-09-23
**Change:** `sftp-transfer-progress-streaming`

## 1. Comandos Cubiertos
- `sftp_upload_file` (`on_progress: tauri::ipc::Channel<TransferProgressPayload>`)
- `sftp_copy_between_sessions` (`on_progress: tauri::ipc::Channel<TransferProgressPayload>`)

## 2. Evidencia de Ejecución (Harness Local / Mock SFTP)
- **Comando:** `cargo run --manifest-path app/src-tauri/Cargo.toml --example smoke_edit_session_local`
- **Resultado:**
  ```
  === smoke_edit_session_local (mock/local only; cero writes al lab SSH) ===
  OK preferencias get/set round-trip
  OK fake SFTP download/upload round-trip
  OK rechazo por tamaño >10 MiB
  OK heurística binaria
  OK edit session replace + fresh baseline + coalesce confirm
  OK cleanup temps locales
  === PASS (sin mutaciones al lab SSH) ===
  ```
- **Validación de Streaming y Staging Atómico (`FakeSftpStore`):**
  - `fake_streaming_upload_emite_progreso_y_limpia_part_en_fallo`: Verifica emisión de `0%` y `100%`, reemplazo atómico tras verificar `stat`, y eliminación del `.nekossh.part` sin corromper el archivo destino previo cuando ocurre un fallo a mitad de transferencia.
  - `fake_streaming_scp_copy_emite_progreso_y_limpia_part_en_fallo`: Verifica el mismo blindaje para la operación `scp` entre sesiones.
