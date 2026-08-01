## Why

La Fase 3 canónica en `docs/project_scope.md` asume Monaco integrado y re-subida silenciosa con `Ctrl+S`. Eso es costoso, acopla un editor completo al shell y no coincide con el flujo FileZilla que el equipo prefiere ahora: editar fuera, detectar cambios y **preguntar** antes de subir. Este change pivota la entrega de “edición remota” a **editor externo + sync con confirmación**, dejando Monaco fuera de alcance (Fase 3b / futuro).

## What Changes

- Pivot de producto: **Fase 3** pasa a “edición remota vía editor externo + sync con confirm A1”; Monaco **no** se implementa aquí y queda diferido (Fase 3b / futuro). Se actualiza la narrativa de alcance en `docs/project_scope.md` en el paso de docs del change.
- Entrada desde el explorador SFTP: **doble clic** en archivo, o menú contextual **“Editar”**, inicia el flujo.
- Descarga del archivo remoto a un directorio temporal de la app, apertura con **editor preferido** (ruta configurable) o, si no hay preferencia válida, con la **asociación del SO**.
- Vigilancia del archivo temporal; al detectar cambio, dialog glass **A1**: “¿Subir al servidor?” (estilo FileZilla). **Sin** auto-upload silencioso.
- Si el usuario confirma → upload/replace del archivo remoto vía el canal SFTP de la sesión. Cancelar / Escape deja el remoto intacto (el local dirty puede seguir en vigilancia o cerrarse según design).
- UI de preferencias: campo para **ruta del editor externo preferido**.
- Limpieza de temporales; política ante desconexión mid-edit; política por defecto para archivos grandes/binarios (con preguntas abiertas documentadas en design).
- Reuso del chrome de confirmaciones A1 ya entregado (no se modifica el change `chrome-confirms-context-menus`).

**Non-goals (explícitos):** Monaco, auto-upload sin confirm, runtime de túneles, host-key UI, auto-reconnect.

**Constraints (verificación / lab SSH):**
- **MANDATORIO**: implementación, tests automatizados y verificación por agente **MUST NOT** modificar/sobrescribir/borrar archivos en el host SSH de pruebas compartido.
- Evidencia de transfer y del ciclo edit → upload vía **mocks / fixtures locales / fake SFTP**; no uploads live al lab.
- Smoke live opcional solo si el usuario provisiona un **sandbox remoto desechable** documentado; default = cero writes remotos en verificación.
- El producto **sí** sube en uso real cuando el usuario confirma; la restricción aplica a trabajo de desarrollo/verificación contra el lab, no al comportamiento de producto.

## Capabilities

### New Capabilities
- `external-file-edit`: Ciclo de edición remota FileZilla-style (temp local, editor externo, file watcher, confirm A1 de subida, upload, cleanup, preferencia de editor, políticas de tamaño/binario y desconexión mid-edit).

### Modified Capabilities
- `sftp-explorer`: Entradas de UX para editar archivos (doble clic / menú “Editar”) y operaciones SFTP de descarga/subida de archivo individual asociadas a la sesión, sin romper listado/PTY.

## Impact

- **Backend (`app/src-tauri`)**: commands SFTP de download/upload de archivo; gestión de temp dir; posible watcher nativo o API para notar cambios; preferencia persistida (SQLite o store local según design).
- **Frontend (`app/src`)**: explorador (doble clic / B3 “Editar”); orquestación del ciclo edit; dialog A1 de subida; UI de settings para ruta de editor; mensajes de error/estado.
- **Docs**: `docs/project_scope.md` (pivot Fase 3), posiblemente `ui-layout-contract.md` / `DESIGN.md` si se añade zona de preferencias o copy de confirm.
- **Specs principales**: nueva `external-file-edit`; delta en `sftp-explorer`.
- **Dependencias**: sin Monaco; posible crate/API de file watch y `open` del SO vía Tauri.
- **Change activo `chrome-confirms-context-menus`**: **no tocar**; solo reutilizar el patrón A1/B3 ya en código.
