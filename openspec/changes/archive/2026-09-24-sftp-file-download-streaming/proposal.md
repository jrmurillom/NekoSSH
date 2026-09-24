## Why

El explorador SFTP de NekoSSH ya cuenta con subida por arrastrar y soltar (`sftp_upload_file`) y copia entre sesiones (`sftp_copy_between_sessions`) con streaming $O(1)$ en RAM (`64 KiB`), staging atómico `.nekossh.part` y barra de progreso en tiempo real en `#files-status`. Sin embargo, el usuario aún no puede descargar archivos remotos desde el árbol del explorador hacia una ruta local elegida en su computadora: la función interna existente (`sftp_download_file_blocking`) está acoplada exclusivamente al flujo de edición externa temporal (`start_external_edit`), impone un límite duro de `10 MiB` (`MAX_EXTERNAL_EDIT_BYTES`), carece de selector de ruta local nativo, no utiliza staging `.nekossh.part` en disco local y no reporta progreso por `tauri::ipc::Channel`.

Incorporar la acción **"Descargar"** en el menú contextual de archivos remotos con diálogo nativo **"Guardar como…"** y el mismo motor de streaming y telemetría permite transferir archivos de cualquier tamaño (incluyendo backups y dumps de múltiples gigabytes) sin alterar el flujo de `Editar`.

## What Changes

- **Menú Contextual SFTP (`Descargar`):** Añadir la opción `"Descargar"` (con ícono `AppIcons.download`) al menú contextual de archivos remotos en el explorador SFTP (junto a `"Editar"` y `"Copiar scp"`), deshabilitándola u omitiéndola si ya hay una transferencia activa (`transferInProgress`).
- **Selector Nativo de Ruta Destino (`sftp_pick_download_path`):** Nuevo comando Tauri en Rust (basado en `rfd::FileDialog`) que abre el diálogo nativo **"Guardar como…"** del sistema operativo con el nombre del archivo remoto pre-rellenado (`set_file_name`), devolviendo `Option<String>` con la ruta absoluta elegida por el usuario (o `None` si cancela).
- **Descarga SFTP en Streaming $O(1)$ sin Límite de Tamaño (`sftp_download_to_user_path_blocking` / `sftp_download_file_with_progress`):**
  - Preservar intacta `sftp_download_file_blocking` (con su límite de `10 MiB`) para el flujo de `"Editar"` (`start_external_edit`), garantizando cero regresiones en la edición externa.
  - Crear la función dedicada de descarga de usuario con búfer fijo de `64 KiB` (`[0u8; 65536]`) en stack ($O(1)$ RAM), bombeo continuo del PTY (`pump_pty`), control determinista de `WouldBlock` (`MAX_WOULD_BLOCK_ATTEMPTS`), escritura sobre archivo temporal local `.<nombre>.nekossh.part` en el directorio destino elegido, verificación post-lectura (`bytes_written == total_bytes`) y renombrado atómico (`std::fs::rename`) al archivo final (o eliminación inmediata `std::fs::remove_file` del `.nekossh.part` ante cualquier fallo).
- **Integración con el Motor de Progreso (`TransferProgressPayload` y `#files-status`):**
  - Extender `TransferProgressPayload.operation` en TypeScript y Rust para soportar `"download"` (`"Descargando <archivo>"`).
  - Conectar `handleDownloadFile` en `app/src/main.ts` al controlador `createExplorerStatusController` y al cerrojo global `transferInProgress`.

## Capabilities

### New Capabilities
- `sftp-file-download`: Selección de ruta local mediante diálogo nativo "Guardar como…", descarga SFTP en streaming $O(1)$ sin límite de tamaño con staging atómico local `.nekossh.part`, y visualización de progreso en tiempo real (`%`, bytes, velocidad) en `#files-status`.

### Modified Capabilities

## Impact

- **Backend (Rust / Tauri v2):**
  - `app/src-tauri/Cargo.toml`: Adición de dependencia ligera `rfd = "0.15"` para el diálogo nativo del sistema operativo (`FileDialog::new().save_file()`).
  - `app/src-tauri/src/edit_util.rs`: Helper `build_local_staging_part_path(local_path: &Path) -> PathBuf` para generar la ruta temporal local `.<filename>.nekossh.part` en el mismo directorio destino.
  - `app/src-tauri/src/external_edit.rs` y `app/src-tauri/src/lib.rs`: Nuevos comandos `sftp_pick_download_path` y `sftp_download_file_with_progress` + función `sftp_download_to_user_path_blocking` (manteniendo intacta `sftp_download_file_blocking` para `start_external_edit`).
  - `app/src-tauri/src/fake_sftp.rs`: Soporte de simulación de descarga en streaming con progreso y verificación de limpieza `.nekossh.part` local ante fallos.
- **Frontend (TypeScript / CSS):**
  - `app/src/modules/transfer-progress-helper.ts`: Extensión del tipo `operation: "upload" | "scp" | "download"` y etiqueta `"Descargando"` en `formatTransferHeader`.
  - `app/src/main.ts`: Opción `"Descargar"` en el menú contextual de archivos remotos y manejador `handleDownloadFile(node)` integrado con `sftp_pick_download_path`, `Channel<TransferProgressPayload>` y `transferInProgress`.
