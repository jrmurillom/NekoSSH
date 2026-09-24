## Why

Actualmente las transferencias SFTP en streaming (`Descargar`, `Subir` por arrastrar y soltar, y `Pegar SCP`) carecen de un mecanismo de cancelación determinista:
1. **Sin control manual en la UI:** El banner de progreso `#files-status.is-progress` no ofrece un botón `[ ✕ ]` para detener una transferencia en curso previa confirmación (`confirmDialog`).
2. **Persistencia fantasma al cerrar la sesión SSH (`close_ssh_session`):** Como `with_live_ssh` clona el `Arc<Mutex<LiveSsh>>` para el hilo `spawn_blocking`, cerrar la pestaña de la terminal mientras se descarga o sube un archivo grande no destruye el socket TCP ni detiene el hilo SFTP, provocando que la transferencia continúe en segundo plano y siga re-renderizando `#files-status` a través del `Channel`.
3. **Huérfanos `.nekossh.part` al cerrar la aplicación (`RunEvent::Exit`):** Si el usuario cierra la aplicación completa (`Alt+F4` o botón de cierre de ventana) durante una descarga, el proceso termina abruptamente sin ejecutar el desenrollado del hilo `spawn_blocking`, dejando archivos temporales incompletos `.<nombre>.nekossh.part` en el disco duro del usuario.

## What Changes

- **Registro Centralizado de Transferencias Activas (`ActiveTransferRegistry` en Rust):**
  - Introducir un estado compartido `ActiveTransferRegistry` (`Arc<Mutex<...>>`) que registra cada transferencia activa con un token atómico de cancelación (`Arc<AtomicBool>`), las terminales involucradas (`terminal_ids: Vec<String>`, cubriendo origen y destino en SCP) y la ruta temporal local (`local_part_path: Option<PathBuf>`).
  - Conectar la verificación `is_transfer_cancelled()` en cada iteración del bucle de `64 KiB` de los **3 motores de streaming** (`sftp_download_to_user_path_blocking`, `sftp_upload_file_blocking_with_progress` y `sftp_copy_between_sessions`), abortando en $<5\text{ ms}$, cerrando descriptores y eliminando el archivo parcial `.nekossh.part` (local con `std::fs::remove_file` o remoto con `sftp_unlink_best_effort`).
- **3 Disparadores Deterministas de Cancelación:**
  1. **Botón `[ ✕ ]` con Confirmación (`confirmDialog`) en `#files-status.is-progress`:** Añadir un botón de cancelación en el encabezado del loader de progreso que abre `confirmDialog` (`"Cancelar transferencia"`). Si el usuario confirma (`"Sí, cancelar"`), invoca el nuevo comando Tauri `sftp_cancel_transfer()`, detiene el archivo en curso (y cualquier archivo restante de un lote de subida) y limpia `.nekossh.part`. Si el usuario elige `"Seguir transfiriendo"`, la transferencia continúa sin interrupción.
  2. **Cancelación Automática al Cerrar Sesión SSH (`close_ssh_session` / `close_all_ssh_connections`):** Al cerrar una pestaña de terminal o todas las terminales, el backend invoca automáticamente `cancel_transfers_for_terminal(&terminal_id)` (tanto si actúa como sesión única o como origen/destino en SCP), y el frontend silencia los callbacks residuales de `Channel` para que `#files-status` quede limpio de inmediato.
  3. **Cancelación y Barrido Síncrono al Cerrar la Aplicación (`RunEvent::ExitRequested` / `RunEvent::Exit`):** En el hook de salida de Tauri, señalizar `cancel_all_transfers_and_cleanup_local_parts()` para eliminar síncronamente cualquier archivo `.nekossh.part` local en curso antes de terminar el proceso.

## Capabilities

### New Capabilities
- `sftp-transfer-cancellation`: Cancelación determinista de transferencias SFTP (`download`, `upload`, `scp`) mediante botón `[ ✕ ]` con diálogo de confirmación en `#files-status`, cancelación automática al cerrar la sesión SSH (`close_ssh_session`) y limpieza síncrona de archivos temporales `.nekossh.part` al cerrar la aplicación.

### Modified Capabilities

## Impact

- **Backend (Rust / Tauri v2):**
  - `app/src-tauri/src/edit_util.rs`: Definición de `ActiveTransferRegistry`, `ActiveTransferHandle` (con `Drop` RAII que desregistra automáticamente la transferencia al terminar), `TRANSFER_CANCELLED_ERROR` (`"TRANSFER_CANCELLED"`), y métodos `cancel_all()`, `cancel_for_terminal(terminal_id)` y `cleanup_local_parts_on_exit()`.
  - `app/src-tauri/src/external_edit.rs`: Integración de `ActiveTransferRegistry` en `sftp_download_to_user_path_blocking`, `sftp_upload_file_blocking_with_progress` y `sftp_copy_between_sessions`, además del nuevo comando Tauri `sftp_cancel_transfer`.
  - `app/src-tauri/src/fake_sftp.rs`: Soporte de token de cancelación `Arc<AtomicBool>` en `FakeSftpStore` para pruebas deterministas de cancelación en subida, descarga y copia SCP.
  - `app/src-tauri/src/lib.rs`: Registro de `ActiveTransferRegistry` en `.manage(...)`, invocación de `cancel_for_terminal` en `close_ssh_session`, y `cancel_all_and_cleanup_local_parts` en `RunEvent::ExitRequested | RunEvent::Exit`.
- **Frontend (TypeScript / CSS):**
  - `app/src/modules/transfer-progress-helper.ts`: Renderizado del botón `[ ✕ ]` (`.files-status-progress-cancel`) dentro de `.files-status-progress-header` en `createExplorerStatusController`, con soporte para callback `onCancelRequest`, detección de error de cancelación (`isTransferCancelledError`) y supresión de ticks post-cancelación.
  - `app/src/styles.css`: Estilos del botón `.files-status-progress-cancel` alineados a `docs/design/DESIGN.md` (sin neón, hover semántico).
  - `app/src/main.ts`: Integración del flujo `confirmDialog` al pulsar `[ ✕ ]`, invocación a `sftp_cancel_transfer`, interrupción de lotes en `runExplorerUpload`, y limpieza inmediata al cerrar terminal (`closeTerminalTab` / `closeAllTerminals`).
