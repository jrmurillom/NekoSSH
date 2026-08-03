## 1. Setup: Create Feature Branch (MANDATORY)

- [x] 1.1 Crear y cambiar a la rama `feature/fix-save-file` antes de modificar el código de la app.
- [x] 1.2 Verificar que la rama activa en Git sea `feature/fix-save-file`.

## 2. Lógica de Clasificación de Sesión de Edición (Rust Backend)

- [x] 2.1 En [app/src-tauri/src/edit_session.rs](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/edit_session.rs), en la función `take_for_terminal`, clasificar la sesión como `dirty` si su fase es `ConfirmPending` o `Uploading`, o si `pending_change_emit` es verdadero.
- [x] 2.2 En [app/src-tauri/src/external_edit.rs](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/external_edit.rs), en la función `disconnect_edit_sessions_for_terminal`, recopilar y enviar al frontend en el payload únicamente los IDs de las sesiones `dirty` (aquellas con `preserve_temp_on_close` habilitado).
- [x] 2.3 En la misma función, no emitir el evento `edit-session-disconnected` si la lista de IDs `dirty` está vacía.

## 3. Pruebas y Verificación

- [x] 3.1 Ejecutar `cargo test` para verificar la estabilidad de las pruebas de backend en Rust.
- [x] 3.2 Compilar el frontend con `npm run build` para asegurar compatibilidad.
- [x] 3.3 Validar que al cerrar una sesión con un archivo limpio (recientemente subido y sin cambios pendientes) no se muestre el cuadro de diálogo de sesión desconectada.
