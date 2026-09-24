## Why

Actualmente, la subida de archivos por arrastrar y soltar (`sftp_upload_file`) y la copia inter-sesión desde el menú contextual (`sftp_copy_between_sessions` / "Pegar scp") carecen de un indicador visual de progreso (porcentaje, barra de avance, bytes transferidos y velocidad), mostrando únicamente un texto fijo hasta finalizar la operación. Además, la subida local carga la totalidad del archivo en memoria RAM (`std::fs::read`), exponiendo a la aplicación a fallos por agotamiento de memoria (*Out-Of-Memory*) con archivos grandes, carece de atomicidad ante cortes de red (truncando el archivo destino directamente) y presenta bucles `WouldBlock` de escritura sin límite de reintentos que pueden bloquear indefinidamente el hilo de transferencia.

## What Changes

- **Streaming de memoria constante $O(1)$ en subidas SFTP:** Reemplazar la lectura completa en RAM (`std::fs::read`) en `sftp_upload_file_blocking` por lectura en streaming desde disco (`std::fs::File`) utilizando un búfer fijo de 64 KiB, igualando la huella de memoria de `sftp_copy_between_sessions`.
- **Tolerancia cero a fallos (Atomicidad, Limpieza y Timeouts):**
  - Implementar escritura segura hacia un archivo temporal oculto (`.<nombre>.nekossh.part`) en el directorio destino tanto para `sftp_upload_file` como para `sftp_copy_between_sessions`, realizando el reemplazo/renombrado al destino final únicamente tras completar el `flush()` y verificar la integridad de tamaño (`sftp.stat()`). En caso de error o interrupción, eliminar el archivo `.nekossh.part` residual.
  - Añadir límite determinista de reintentos en `WouldBlock` durante los ciclos de `write` SFTP para eliminar posibles bucles infinitos ante caídas abruptas de conexión.
  - Unificar el semáforo de concurrencia en el frontend (`transferInProgress`) para impedir colisiones simultáneas entre subidas por Drag & Drop y operaciones de "Pegar scp".
- **Canal de progreso en tiempo real (`tauri::ipc::Channel`):**
  - Integrar `tauri::ipc::Channel<TransferProgressPayload>` en los comandos `sftp_upload_file` y `sftp_copy_between_sessions` con *throttling* temporal de `100ms` (más emisión garantizada al `0%` y `100%`), reportando nombre de archivo, bytes transferidos, bytes totales, porcentaje (`0..=100`) y velocidad en bytes por segundo.
- **Loader y barra de progreso visual en el Explorador SFTP:**
  - Enriquecer el overlay flotante inferior `#files-status` del panel de archivos para renderizar una barra de progreso temática (Cyber-Sakura), porcentaje de avance, contador de lote `(i/N)`, bytes transferidos/totales y velocidad de transferencia tanto al subir archivos como al ejecutar "Pegar scp", sin desplazar el árbol de archivos.

## Capabilities

### New Capabilities
<!-- Ninguna capacidad nueva independiente; se amplía la capacidad existente sftp-explorer -->

### Modified Capabilities
- `sftp-explorer`: Añadir requisitos de streaming en memoria constante $O(1)$, escritura temporal atómica con limpieza de residuos, protección contra bucles infinitos en `WouldBlock`, canal IPC de telemetría de transferencia y barra de progreso visual con porcentaje para subidas por arrastre y "Pegar scp".

## Impact

- **Backend (Rust / Tauri v2):**
  - `app/src-tauri/src/external_edit.rs` (y módulo auxiliar de transferencia SFTP): Refactorización de `sftp_upload_file_blocking`, `sftp_upload_file` y `sftp_copy_between_sessions`; incorporación de `tauri::ipc::Channel` y pruebas unitarias con `FakeSftpStore`.
- **Frontend (TypeScript / CSS / HTML):**
  - `app/src/modules/transfer-progress-helper.ts` y `app/src/modules/transfer-progress-helper.test.ts`: Lógica pura de cálculo, formateo de unidades (`B`, `KB`, `MB`, `GB`), velocidad (`KB/s`, `MB/s`) y porcentaje.
  - `app/src/main.ts` y `app/src/styles.css`: Integración del `Channel` de `@tauri-apps/api/core` en `runExplorerUpload` y `handlePasteScp`, bloqueo de concurrencia unificado y renderizado del loader con barra de progreso en `#files-status`.
- **Dependencias:** Cero dependencias externas nuevas (usa `tauri::ipc::Channel` y `std::fs::File` ya disponibles en el stack).
