## Context

NekoSSH guarda hoy “perfiles” planos en `profiles` + `auth_credentials` + `ssh_tunnels`. La UI lista ítems planas con editar/eliminar. El usuario quiere jerarquía tipo referencia: **carpetas (profiles/groups)** → **conexiones SSH** anidadas; rename inline; icono carpeta = agregar carpeta; conexiones solo bajo carpeta; CRUD alineado.

Terminología en producto (español UI / inglés código):
- **Folder** (`connection_folders`): grupo nombrado (Production, Staging).
- **Connection** (tabla actual `profiles` o renombrada a `connections`): host/user/auth/túnel; FK a folder.

## Goals / Non-Goals

**Goals:**
- Un nivel de carpetas; conexiones hijas.
- Expand/collapse; rename inline de carpeta (y de conexión si es natural).
- Acción “agregar carpeta” (icono carpeta / +).
- “Nueva conexión” exige carpeta destino (contexto de carpeta seleccionada o menú en carpeta).
- Migración: conexiones existentes → carpeta default.
- CRUD completo carpetas + conexiones con FK.

**Non-Goals:**
- Carpetas anidadas (multi-nivel).
- Drag-and-drop reorder (nice-to-have futuro).
- Sync cloud de carpetas.
- Cambiar protocolo SSH/PTY/SFTP en este change.

## Decisions

### 1. Modelo de datos
- **Decisión**: Tabla `connection_folders (id, name, sort_order, created_at)`. Conexiones actuales (`profiles`) ganan `folder_id INTEGER NOT NULL REFERENCES connection_folders(id) ON DELETE CASCADE` (o SET NULL + regla UI — preferir CASCADE con confirmación).
- **Alternativa**: JSON embebido — se descarta por queries/CRUD.
- **Migración**: crear carpeta `"General"` (o `"Default"`); `UPDATE profiles SET folder_id = …`.

### 2. Naming en código vs UI
- **Decisión**: UI puede decir “Profiles”/carpetas según copy Cyber-Sakura; en código inglés: `folder` / `connection`. Evitar llamar “profile” a la conexión SSH en APIs nuevas para no confundir.
- **Legacy**: mantener tabla `profiles` como conexiones en v1 de migración (menos churn) **o** rename a `connections` en la misma migración — **preferir** `folder_id` en `profiles` sin rename de tabla en este change (menos riesgo); documentar alias semántico “connection profile row = connection”.

### 3. Inline rename
- **Decisión**: Doble-click o click en nombre → `<input>` inline; Enter guarda; Escape cancela; blur guarda o cancela según UX (preferir Enter=save, Escape=cancel, blur=save si dirty).
- Commands: `update_folder(id, name)`, `update_profile` ya existe para nombre de conexión.

### 4. UI árbol
- **Decisión**: Sidebar Servidores muestra árbol: folder rows (chevron Lucide + Folder icon + name) / connection rows indentadas (dot estado opcional). Header con acción add-folder (icono carpeta o Plus+Folder).
- Conexión: click = conectar (como hoy); acciones editar credenciales siguen modal; delete con confirm.
- Carpeta: no conecta; solo organiza.

### 5. CRUD commands
- Folders: `list_folders`, `create_folder`, `update_folder`, `delete_folder` (cascade conexiones).
- Connections: `list_profiles` / listado jerárquico `list_connection_tree`; `create_profile` requiere `folder_id`; update/delete existentes + folder_id.

## Risks / Trade-offs

- **[Risk]** Borrar carpeta borra conexiones → *Mitigación*: confirmación explícita con conteo.
- **[Risk]** Conexión sin carpeta tras bug → *Mitigación*: NOT NULL folder_id + migración.
- **[Risk]** Confusión nombre “profile” legacy → *Mitigación*: docs + UI copy claros.

## Migration Plan

1. Migration SQL `002_connection_folders.sql`.
2. Backend commands + tests rusqlite.
3. Frontend árbol + inline edit + wiring.
4. Rollback: revert migration solo en dev DBs; prod backup antes.

## Open Questions

- ¿Permitir mover conexión entre carpetas en este change? **Sí, mínimo**: update `folder_id` vía UI simple (menú “Mover”) si cabe en tasks; si no, dejar para follow-up y solo create-in-folder.

### Corrección de Ruta (Fix)

**Motivo:** Feedback de UI sobre la tarjeta de conexión en el árbol: el label `SSH (Contraseña|Llave)` no aporta valor y alarga la tarjeta; el endpoint `user@host:port` debe ser la línea de acento; abrir sesión con un solo click es demasiado fácil de disparar al editar/seleccionar.

**Nueva estrategia (tarjeta de conexión):**
1. **Quitar** la fila de detalle auth (`SSH (Contraseña)` / `SSH (Llave)` / túnel) — menos altura, más densidad.
2. **Reusar el color** de esa etiqueta (token `--color-cyan-electric`, clase previa `.profile-item-details`) en la línea `user@host:port` (`.profile-item-host`).
3. **Icono copiar** (Lucide `Copy` / paste-style) al final de esa línea: al activarlo copia al clipboard el string `user@host` (sin puerto, salvo que el producto ya muestre solo user@ip — copiar exactamente `username@host`).
4. **Abrir conexión con doble clic** en la tarjeta (no con click simple). Click simple puede seleccionar/resaltar; botones editar/eliminar siguen con stopPropagation. El botón copiar también stopPropagation y no abre sesión.

**Impacto:** solo frontend (`main.ts` render de `buildProfileItem`, `styles.css`, `icons.ts`). Sin cambios de schema/backend. Invalidar parcialmente la decisión 4 (“click = conectar”) → queda **doble clic = conectar**.

### Corrección de Ruta (Fix) — Scrollbars globales

**Motivo:** El explorador SFTP (y otros paneles con overflow) muestran scrollbars nativos del OS/WebView (track blanco, thumb gris) que rompen Cyber-Sakura. El template ya tiene un scrollbar temático solo en `.modal-content`.

**Nueva estrategia:**
1. **Generalizar** los estilos del template (webkit: track transparente, thumb sakura semitransparente, ~6px, border-radius) a **todos** los contenedores scrolleables de la app (`*`, `html`/`body`, o selectores globales `::-webkit-scrollbar*`).
2. Añadir equivalentes **Firefox**: `scrollbar-width: thin;` + `scrollbar-color` con tokens sakura / transparente.
3. Evitar track blanco; sin glow en el thumb.
4. **SSOT:** documentar en `docs/design/DESIGN.md` (look & feel) que todo scrollbar de chrome UI MUST usar estos estilos/tokens — no dejar defaults de OS.

**Impacto:** `app/src/styles.css` (+ posible limpieza de regla solo-modal); `docs/design/DESIGN.md`. Sin backend.

### Corrección de Ruta (Fix) — Click collapse en fila de carpeta

**Motivo:** Expandir/colapsar solo con el chevron es poco ergonómico; el usuario espera click en toda la fila (bloque header de carpeta).

**Nueva estrategia:**
1. **Click en toda la `.folder-row`** (chevron, icono folder, nombre, padding) alterna expand/collapse y actualiza el chevron.
2. **Acciones `+` y basurero** (`folder-actions`) MUST seguir funcionando: `stopPropagation` en sus click handlers; `z-index` / `pointer-events` para que no queden “detrás” del hit-area del collapse (la fila no debe capturar el click de esos botones).
3. Click en la fila también puede seguir marcando carpeta activa (contexto), sin interferir con add/delete.
4. Rename inline (doble clic en nombre) se mantiene; al editar, el click en el input no debe togglear collapse (`stopPropagation`).

**Impacto:** solo `app/src/main.ts` (wiring de `renderProfileList` / folder row) y CSS menor si hace falta (`pointer-events` / stacking). Sin backend.
