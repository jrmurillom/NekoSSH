## Why

Los `window.confirm` / `alert` nativos rompen Cyber-Sakura y ya hay reglas SSOT (dialog glass A1 + menú contextual B3 con hover sakura). El árbol de conexiones aún expone basurero/lápiz en la fila y rename por doble clic; el usuario quiere acciones secundarias en menú contextual y la fila más limpia (solo `+` en carpetas; copy en conexiones).

## What Changes

- Reemplazar confirms/alerts nativos de producto por **dialog glass centrado (A1)** según `DESIGN.md`.
- Árbol — **carpetas**: quitar basurero de la fila; **dejar `+`**. Basurero y “Cambiar nombre” viven en menú contextual. Rename sigue **inline**, disparado desde el menú (ya no por doble clic).
- Árbol — **conexiones**: quitar lápiz y basurero de la tarjeta. Acciones (editar perfil / renombrar / eliminar / …) vía menú contextual. Rename/edición de nombre **inline** sin doble clic. **Conservar** el botón copy `user@host`. Doble clic en la tarjeta sigue siendo conectar (si aplica el contrato actual).
- Menú contextual: patrón B3 (ítems con icono Lucide), hover sakura (como “Nueva conexión”).
- Documentar/verificar alineación con SSOT ya escrito; deltas de specs de carpetas/conexiones y capacidad de overlays si hace falta.

## Capabilities

### New Capabilities

- `ui-overlays`: dialogs de confirmación glass y menús contextuales de chrome (contrato de comportamiento + look referenciado a DESIGN).

### Modified Capabilities

- `connection-folders`: acciones de carpeta vía menú contextual; sin basurero en fila; rename inline desde menú.
- `connection-profiles`: acciones de conexión vía menú contextual; sin lápiz/basurero en tarjeta; copy permanece; rename inline desde menú.

## Impact

- Frontend: `main.ts` (árbol, confirms, context menus), `styles.css`, `index.html` (shell de dialog/menu si aplica), `icons.ts`.
- Docs: refuerzo menor en `DESIGN.md` / `ui-layout-contract.md` si el apply revela gaps.
- Fuera de alcance: auto-reconnect, túneles, Fase 3 Monaco, reescribir todo el modal de perfil (solo cómo se abre).
