## Why

Actualmente, las subidas de archivos binarios (como .zip, imágenes, ejecutables y archivos comprimidos) a través de SFTP pueden corromperse o quedar incompletas en el servidor remoto. Esto ocurre porque la sesión SSH opera en modo no bloqueante (session.set_blocking(false)) y la subida termina destruyendo el handle SFTP (ssh2::File) sin un bucle de vaciado (lush) ni verificación de integridad en el destino. Es necesario garantizar transferencias íntegras de cualquier tipo de archivo (texto o binario) sin degradar la estabilidad de la terminal PTY interactiva.

## What Changes

- **Bucle de vaciado (Flush & Drain) en subida SFTP**: Se implementa un ciclo de vaciado obligatorio de buffers (emote.flush()) con reintentos no bloqueantes y bombeo continuo del PTY (pump_pty) antes de finalizar la transferencia.
- **Verificación de integridad post-transferencia (sftp.stat)**: Tras escribir y sincronizar el archivo remoto, se valida automáticamente mediante sftp.stat() que el tamaño en bytes en el destino coincida de forma exacta con el archivo local.
- **Detección temprana de discrepancias**: Si el tamaño remoto difiere del local, se reporta error de integridad explícito inmediatamente.
- **Sin cambios rompientes**: No se altera la interfaz de usuario ni la firma del comando sftp_upload_file.

## Capabilities

### New Capabilities

*(Ninguna)*

### Modified Capabilities

- sftp-explorer: Se especifica que la subida SFTP de archivos individuales (vía drag & drop o reemplazo) debe garantizar el vaciado completo de buffers en modo no bloqueante y la verificación exacta de tamaño en bytes post-subida.

## Impact

- **Backend Rust**: pp/src-tauri/src/external_edit.rs (sftp_upload_file_blocking).
- **Tests / Harness**: Pruebas unitarias y de integración simulada (ake_sftp / harness local) para verificar la subida de datos binarios y la validación de tamaño.
- **Frontend / UI**: Sin cambios de API; el usuario percibe subidas confiables de archivos .zip y binarios.
