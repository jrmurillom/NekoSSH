## Context

Al cerrar una terminal, la app limpia todas las sesiones de edición asociadas. Sin embargo, advierte al usuario de manera indiscriminada sobre "ediciones no subidas" incluso si el archivo local ya fue guardado y subido de forma correcta.

## Goals / Non-Goals

**Goals:**
- Distinguir entre sesiones de edición "dirty" (con cambios pendientes de subir o confirmar) y "clean" (sincronizadas con el servidor).
- Evitar falsos positivos de advertencia de desconexión al cerrar la terminal si las sesiones de edición están sincronizadas.
- Limpiar silenciosamente los archivos temporales de las sesiones limpias al cerrar la terminal.

**Non-Goals:**
- Cambiar el debounce del watcher.
- Modificar el flujo de guardado y confirmación del archivo.

## Decisions

### Clasificación de Sesiones Dirty vs Clean en `take_for_terminal`
- **Decisión**: Modificar `EditSessionRegistry::take_for_terminal` en `app/src-tauri/src/edit_session.rs`.
- **Lógica**: Una sesión de edición se considera "dirty" únicamente si su fase es `ConfirmPending` o `Uploading`, o si tiene cambios en disco detectados por el watcher que aún no han sido procesados por el debounce (`pending_change_emit == true`).
- **Implementación**:
  ```rust
  let is_dirty = rec.phase == EditSessionPhase::ConfirmPending
      || rec.phase == EditSessionPhase::Uploading
      || rec.pending_change_emit;
  rec.preserve_temp_on_close = is_dirty && preserve_temps;
  ```

### Filtrado de Notificaciones en `disconnect_edit_sessions_for_terminal`
- **Decisión**: En `app/src-tauri/src/external_edit.rs`, recolectar únicamente los IDs de las sesiones que conservaron el temporal (`preserve_temp_on_close == true`) para poblar el payload de desconexión enviado al frontend.
- **Implementación**: Si tras recorrer la lista de sesiones cerradas, la lista de IDs "dirty" queda vacía, se omite el envío del evento `edit-session-disconnected` hacia el frontend.

## Risks / Trade-offs

- **Riesgo**: Que se elimine un archivo local que el usuario sí modificó pero que el debounce de 500ms no ha capturado en el instante exacto de cerrar la sesión.
- **Mitigación**: `pending_change_emit` se establece inmediatamente al recibir el evento de FS. Si el usuario modifica el archivo y cierra la terminal en menos de 500ms, `pending_change_emit` es `true`, por lo que se clasificará como "dirty" y se conservará el archivo temporal local informando al usuario.
