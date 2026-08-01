## Why

La Fase 1 dejó terminal SSH y perfiles, pero sin explorador remoto el usuario no puede navegar el filesystem del servidor sin comandos a ciegas. La Fase 2 desbloquea SFTP dedicado y sync Explorer ⇄ Terminal según `docs/project_scope.md`.

## What Changes

- Conexión SSH **A** = PTY solo; conexión SSH **B** = SFTP dedicado (viva con la terminal).
- Pestaña/panel **Archivos** con árbol SFTP lazy navegable (expand, abrir, subir, Ir, Actualizar).
- Menú contextual **Abrir en Terminal** (`cd` al PTY + listar path en explorador).
- **No** incluye sync automático del explorador tras `cd` tipado (fuera de alcance).
- Docs layout + verificación **con smoke SSH real** antes de marcar done.

## Capabilities

### New Capabilities
- `sftp-explorer`: Canal SFTP, listado de árbol remoto en sidebar y operaciones de navegación básicas del explorador.

### Modified Capabilities
- `ssh-terminal`: Aceptar `cd` desde el explorador; PTY estable.

## Impact

- **Backend (`app/src-tauri`)**: segundo canal/sesión SFTP con `ssh2`, commands Tauri de listado/navegación, ciclo de vida ligado a la sesión de terminal.
- **Frontend (`app/src`)**: habilitar panel Archivos, árbol UI, menú contextual, listeners de sync con la terminal.
- **Docs**: `ui-layout-contract.md` (Fase 2), posiblemente `project_scope` solo si hay ajuste de alcance.
- **No incluye** (Fase 3+): Monaco, descarga/edición/re-subida de archivos, snippets, mascotas.
