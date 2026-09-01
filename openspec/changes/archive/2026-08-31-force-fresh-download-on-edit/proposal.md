## Why

Al presionar el botón  Editar sobre un archivo en el explorador SFTP, el sistema actualmente reutiliza la sesión local en memoria sin descargar una copia fresca del servidor si la sesión previa sigue en estado de vigilancia (Watching). Esto provoca que el usuario edite una versión vieja o desactualizada del archivo si este cambió en el servidor remoto. Es necesario garantizar que cada solicitud de edición descargue obligatoriamente la versión más reciente del archivo desde el servidor, cancelando cualquier watcher anterior y levantando un nuevo watcher limpio.

## What Changes

- **Descarga fresca mandatoria al editar**: Al invocar start_external_edit, el sistema detiene y reemplaza cualquier watcher previo asociado al archivo remoto y descarga siempre la versión más reciente desde SFTP.
- **Reemplazo limpio de sesión de edición**: Se reinicia la sesión con un nuevo baseline (aseline_fingerprint) derivado de la versión fresca recién descargada.
- **Continuidad de vigilancia activa**: Tras la descarga y apertura del editor, un nuevo watcher managed queda activo para detectar cambios locales futuros y permitir el flujo de confirmación de subida A1 habitual.

## Capabilities

### New Capabilities

*(Ninguna)*

### Modified Capabilities

- external-file-edit: Se actualiza el requerimiento de sesión de edición externa para que cada acción de edición descargue obligatoriamente la versión actual del archivo remoto, cancelando el watcher previo y re-estableciendo la vigilancia.

## Impact

- **Backend Rust**: pp/src-tauri/src/external_edit.rs (start_external_edit) y pp/src-tauri/src/edit_session.rs.
- **Tests / Harness**: Pruebas unitarias de reemplazo y re-descarga de sesión de edición.
- **Frontend**: Transparente; al hacer clic en Editar, el usuario siempre recibe el contenido más reciente del servidor.