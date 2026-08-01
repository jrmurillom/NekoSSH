## Why

La Fase 4 del alcance pide un diccionario local de snippets categorizados para acelerar comandos frecuentes en el día a día SSH. Hoy no hay superficie de snippets en el shell; este change entrega solo el gestor (sin Petdex ni inserción a PTY) para desbloquear la fase con un slice usable y persistente.

## What Changes

- Añadir **gestor de snippets** local: categorías de un solo nivel + snippets con título y cuerpo.
- Trigger de apertura: **botón único** en la franja inferior izquierda del sidebar (`sidebar-footer`) — sin atajo de teclado en este slice.
- Modal con chrome **glass/transparencia** al estilo del modal de perfil (no patrón browse A1).
- UI del modal: **lista plana** de snippets, **chips** de categoría, búsqueda por texto; CRUD in-modal (crear categoría/snippet, editar título/cuerpo, eliminar).
- Acción de fila **Copiar**: solo al portapapeles del sistema (sin `write_ssh_input` / PTY).
- Acción de fila **Eliminar**: confirmación con dialog glass A1 (`confirmDialog` existente).
- **Seed demo** al primer arranque (si la DB está vacía): categorías Apache, Tomcat, Permisos con unos pocos snippets cada una; todos eliminables.
- Persistencia SQLite alineada a patrones existentes (migración + módulo Rust + commands Tauri), no embebida en `app_preferences` key/value.
- Actualizar SSOT de producto: `docs/project_scope.md` (Fase 4 snippets vs Petdex) y `docs/design/ui-layout-contract.md` (zona sidebar / modal).

### Fuera de alcance (este slice)

- Petdex / mascotas
- Inserción de snippet al PTY (`write_ssh_input`)
- Atajo de teclado para abrir el gestor
- Categorías anidadas / árbol expandible
- Menús contextuales dedicados a snippets (salvo que el apply demuestre necesidad mínima)

## Capabilities

### New Capabilities

- `snippets-manager`: diccionario local de snippets categorizados (un nivel), modal glass con lista plana + chips + búsqueda, CRUD in-modal, copiar al portapapeles, eliminar con confirm A1, seed demo y persistencia SQLite vía commands.

### Modified Capabilities

- (ninguno — reutiliza `confirmDialog` de `ui-overlays` sin cambiar sus requisitos)

## Impact

- Frontend: `index.html` (botón footer + markup modal), `main.ts` / módulos UI, `styles.css` (chrome glass alineado a profile-modal), iconos Lucide si aplica.
- Backend Tauri: migración SQLite nueva, módulo de snippets, commands IPC (list/create/update/delete categorías y snippets; seed idempotente).
- Docs: `project_scope.md`, `ui-layout-contract.md` (y `DESIGN.md` solo si el apply revela gap de tokens).
- Superficies: `desktop-ui` + `desktop-commands`.
- Sin impacto en SFTP, edición externa, perfiles ni Petdex.
