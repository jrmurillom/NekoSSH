## Why

Actualmente, al cerrar una sesión de terminal SSH en NekoSSH, se notifica al usuario con una advertencia ("La sesión se desconectó; no se pudo subir. El archivo local se conservó temporalmente") incluso si el archivo editado externamente ya fue guardado y subido con éxito al servidor. Esto genera un falso positivo confuso para el usuario.

## What Changes

- Modificar la desconexión de sesiones de edición para que solo emita la advertencia de archivos no guardados y preserve los archivos temporales cuando existan cambios locales reales pendientes de subir (sesiones en fase de `ConfirmPending`, `Uploading` o con eventos de modificación debounce pendientes de emitir).
- Si la sesión de edición está limpia (`Watching` sin cambios pendientes), eliminar el archivo temporal de forma silenciosa y no emitir la advertencia al cerrar la terminal.

## Capabilities

### New Capabilities

### Modified Capabilities
- `external-file-edit`: Cambiar el comportamiento al cerrar la sesión SSH para que solo advierta e informe sobre sesiones de edición "dirty" (con cambios pendientes) y limpie silenciosamente las sesiones inactivas o ya sincronizadas.

## Impact

- Afecta a `app/src-tauri/src/external_edit.rs` (función `disconnect_edit_sessions_for_terminal`) y `app/src-tauri/src/edit_session.rs` (método `take_for_terminal`).
