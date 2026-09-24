## Context

El explorador SFTP de NekoSSH cuenta con:
1. Un motor de streaming en Rust (`external_edit.rs`, `edit_util.rs`) con búfer fijo de `64 KiB` ($O(1)$ RAM), `ProgressThrottler` (`100ms`), `calculate_transfer_percent` (`u128` para archivos $>4\text{ GiB}$), y `tauri::ipc::Channel<TransferProgressPayload>` para subidas (`sftp_upload_file`) y copias SCP (`sftp_copy_between_sessions`).
2. Un controlador DOM modular en el frontend (`createExplorerStatusController` en `transfer-progress-helper.ts`) que actualiza `#files-status` sin recrear nodos DOM (`0` reflows estructurales) y coordina el cerrojo global `transferInProgress`.
3. Una función preexistente `sftp_download_file_blocking` diseñada exclusivamente para `start_external_edit` (`Editar`), la cual aplica un límite estricto de `10 MiB` (`MAX_EXTERNAL_EDIT_BYTES`), descarga hacia una carpeta temporal oculta de sesión de edición (`nekossh-edits/<uuid>`) y registra un file watcher (`notify`).

El objetivo de este diseño es añadir la capacidad de **descargar cualquier archivo remoto hacia una ruta local elegida por el usuario** (incluyendo archivos de múltiples gigabytes) reutilizando el motor de progreso y streaming, **sin alterar ni tocar el flujo de `Editar` (`start_external_edit`)**.

## Goals / Non-Goals

**Goals:**
- Aislar completamente el flujo de `Descargar` del flujo de `Editar` (`start_external_edit`), preservando `sftp_download_file_blocking` intacta para la edición externa.
- Proporcionar un selector nativo **"Guardar como…"** del sistema operativo (`sftp_pick_download_path` vía `rfd::FileDialog`) con el nombre del archivo remoto pre-cargado.
- Implementar `sftp_download_to_user_path_blocking` y el comando Tauri `sftp_download_file_with_progress` con:
  - Memoria constante $O(1)$ (`[0u8; 65536]`, `64 KiB` en stack) y **cero límite de tamaño** (`u64` / `u128`).
  - Escritura atómica sobre archivo temporal oculto local `.<nombre>.nekossh.part` en el mismo directorio destino (`build_local_staging_part_path`).
  - Límite determinista de reintentos en `WouldBlock` (`MAX_WOULD_BLOCK_ATTEMPTS = 400`) + bombeo continuo de PTY (`pump_pty`).
  - Verificación de integridad (`bytes_written == total_bytes`) antes de ejecutar `std::fs::rename` (con fallback a `remove_file` + `rename` en Windows si el archivo destino ya existía y el usuario confirmó reemplazarlo en el diálogo del SO).
  - Limpieza inmediata (`std::fs::remove_file`) del `.nekossh.part` local si ocurre cualquier error o desconexión.
- Conectar `"download"` (`"Descargando <archivo>"`) a `TransferProgressPayload` y `createExplorerStatusController`.

**Non-Goals:**
- No modificar el límite de `10 MiB` ni el watcher de `start_external_edit` (`Editar`).
- No implementar descarga recursiva de directorios completos en esta iteración (alcance acotado a archivos individuales en el menú contextual, consistente con `Editar` y `Copiar scp`).

## Decisions

### 1. Separación Estricta entre `Editar` y `Descargar`
- **Decisión:** Mantener `sftp_download_file_blocking` sin cambios para `start_external_edit`, y crear `sftp_download_to_user_path_blocking` + comando `sftp_download_file_with_progress` para la descarga de usuario.
- **Racional:** `Editar` necesita el guardián `exceeds_edit_size_limit(size)` (`10 MiB`) para proteger al editor externo y la carpeta temporal gestionada por `EditSessionRegistry`. `Descargar` escribe directamente en el disco del usuario sin límite de tamaño y sin watchers. Separarlas garantiza **cero regresiones** en `Editar`.

### 2. Selector de Ruta Local con `rfd` (`sftp_pick_download_path`)
- **Decisión:** Utilizar la crate `rfd = "0.15"` en Rust dentro de `spawn_blocking` mediante un comando dedicado `sftp_pick_download_path(default_name: String) -> Result<Option<String>, String>`.
- **Racional:**
  - En un navegador/WebView, `<input type="file">` no permite abrir un diálogo "Guardar como…" ni obtener una ruta de destino arbitraria en el sistema de archivos local.
  - `rfd` (*Rusty File Dialog*) es la librería nativa oficial que Tauri utiliza internamente; invocarla desde un comando Rust evita añadir permisos ACL complejos de plugins en el frontend, pre-rellena el nombre original del archivo (`FileDialog::new().set_file_name(&default_name).save_file()`) y delega al sistema operativo la advertencia nativa de sobreescritura si el archivo ya existe.

### 3. Staging Atómico Local `.<nombre>.nekossh.part` y Reemplazo Seguro en Windows
- **Decisión:** Añadir `build_local_staging_part_path(local_path: &Path) -> PathBuf` en `edit_util.rs` que construye `.<file_name>.nekossh.part` dentro de `local_path.parent()`. Al finalizar y verificar que `bytes_transferred == total_bytes`, ejecutar `std::fs::rename(&part_path, local_path)` (si en Windows el destino ya existe y `rename` requiere limpieza previa o reemplazo, eliminar destino previo solo tras haber cerrado y sincronizado `part_path` al 100%). Si cualquier paso previo falla, cerrar `part_file` y ejecutar `let _ = std::fs::remove_file(&part_path);`.
- **Racional:** Garantiza que el usuario jamás vea un archivo corrupto o a medio descargar en la carpeta elegida si se corta la conexión SSH al 99%.

## Risks / Trade-offs

- **[Risk] Bloqueo del hilo de UI al abrir el diálogo nativo del SO** $\rightarrow$ **Mitigation:** Ejecutar `rfd::FileDialog::new().set_file_name(&default_name).save_file()` dentro de `tauri::async_runtime::spawn_blocking` para que el bucle de eventos principal y el WebView permanezcan libres.
- **[Risk] Sobreescritura en Windows cuando el archivo destino ya existe** $\rightarrow$ **Mitigation:** En Rust `std::fs::rename(from, to)` en Windows (`MoveFileExW` con `MOVEFILE_REPLACE_EXISTING` en Rust stdlib moderna) reemplaza archivos existentes; además, si el archivo destino tiene atributos o requiere remoción previa, se maneja de forma segura únicamente **después** de que `.nekossh.part` ha sido completamente escrito, vaciado (`flush`) y verificado en tamaño (`bytes_transferred >= total_bytes`).

### Corrección de Ruta (Fix)
- **Fecha:** 2026-09-23
- **Motivo:** Auditoría de producción post-implementación para blindar dos casos de borde reales en entornos de administración de servidores:
  1. **Descarga de archivos de log activos en crecimiento (`/var/log/...`):** Un archivo remoto puede recibir líneas adicionales entre el `sftp.stat()` inicial y el `EOF` (`Ok(0)`) del stream SFTP (`written_size > expected_size`). Rechazar `written_size != expected_size` causaría falsos fallos al descargar logs vivos. Se ajusta la invariante post-EOF de descarga para fallar y borrar `.nekossh.part` únicamente cuando `written_size < expected_size` (truncamiento/descarga incompleta), permitiendo `written_size >= expected_size`.
  2. **Cerrojo `transferInProgress` durante el diálogo nativo `"Guardar como…"` (`sftp_pick_download_path`):** Se adelanta `transferInProgress = true` al inicio de `handleDownloadFile` envolviendo `sftp_pick_download_path` dentro del bloque `try / finally`, impidiendo que se disparen diálogos o transferencias concurrentes mientras el usuario elige la ruta y garantizando su liberación inmediata si cancela el diálogo.
