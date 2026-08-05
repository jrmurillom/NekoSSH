## Why

Hoy cada pestaña es un solo shell: para ver varios terminales del mismo servidor hay que abrir pestañas distintas, y el explorador SFTP ya está amarrado a esa pestaña. Hace falta un modelo de **pestaña = contexto** (login padre + SFTP) con shells hijos independientes en layout de cuadrícula, sin romper el fondo transparente ni el vínculo SFTP.

## What Changes

- Cada pestaña de terminal pasa a ser un **contexto** con un **shell padre** (ancla SFTP) y hasta **3 shells hijos** del mismo perfil (máximo 4 celdas en cuadrícula 2×2).
- Los hijos son **logins SSH independientes** (mismo perfil); cerrar o desconectar un hijo no afecta al SFTP ni al padre.
- Cerrar la **pestaña** cierra el contexto completo: padre + todos los hijos.
- El padre es **estático en UI**: no se cierra solo; si el padre se cae, el contexto queda desconectado (reconectar contexto o cerrar pestaña).
- Layout progresivo dentro de `.terminal-panel`: 1 → 2 → 3 (forma T) → 4 (2×2); el fondo/opacidad sigue en el panel de pestaña, no por celda.
- Acción “nuevo shell” (mismo contexto) deshabilitada al llegar a 3 hijos.
- Foco por celda: teclado e I/O van al shell enfocado; el explorador SFTP sigue ligado al **padre** del contexto activo.

## Capabilities

### New Capabilities

- `tab-context-multi-shell`: modelo padre/hijos por pestaña, límite de 3 hijos, ciclo de vida (cerrar tab = todo; cerrar hijo = solo hijo), acción nuevo shell, foco de celda y layout de cuadrícula hasta 4.

### Modified Capabilities

- `ssh-terminal`: cierre/reconexión y eventos pasan a distinguir contexto (padre) vs shell hijo; cerrar pestaña libera todas las Sessions del contexto.
- `terminal-layout`: el panel unificado aloja una cuadrícula de celdas; padding/fondo/glow siguen en el contenedor de pestaña.
- `sftp-explorer`: el explorador se liga al `terminal_id` del **padre** del contexto activo, no al hijo enfocado.

## Impact

- Frontend: `app/src/main.ts` (modelo `ActiveTerminal` → contexto con hijos), CSS del panel/grid, header con “+”, foco/atajos.
- Backend Rust: reutilizar `start_ssh_session` / `close_ssh_session` por `terminal_id` (un id por shell); agrupar cierre al cerrar pestaña; SFTP sigue usando el id del padre.
- Sin cambios de esquema SQLite ni de perfiles.
- Diseño: respetar `docs/design/DESIGN.md` y `ui-layout-contract.md` (glow solo en `.terminal-panel`, xterm transparente).
