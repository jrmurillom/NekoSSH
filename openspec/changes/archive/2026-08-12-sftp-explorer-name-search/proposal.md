## Why

En carpetas remotas con muchos archivos, localizar una entrada por nombre obliga a recorrer el árbol a ojo. Un filtro rápido por nombre en el explorador Archivos reduce fricción sin salir del panel ni listar de nuevo vía SFTP.

## What Changes

- Añadir un campo de filtro por nombre en el panel Archivos (explorador SFTP), con botón **×** para limpiar y volver a mostrar todo.
- Filtrar en cliente **todas las filas ya pintadas** del árbol (cwd + hijos de carpetas ya expandidas), por substring sin distinguir mayúsculas/minúsculas.
- Si un padre no coincide pero un hijo visible sí, el padre permanece visible para conservar la jerarquía.
- Mostrar estado vacío “(sin coincidencias)” cuando el filtro no deja ninguna fila.
- **Fuera de alcance:** búsqueda recursiva remota, regex, nuevos comandos SFTP en el backend, atajo de teclado obligatorio.

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `sftp-explorer`: requisitos de UI y comportamiento para filtrar por nombre las entradas ya pintadas del explorador remoto.

## Impact

- Frontend: `app/index.html` (toolbar / zona Archivos), `app/src/main.ts` (`renderExplorerTree` / `buildExplorerNodeEl`), `app/src/styles.css`.
- Backend Rust/SFTP: sin cambios de API.
- Specs: delta en `openspec/specs/sftp-explorer`.
