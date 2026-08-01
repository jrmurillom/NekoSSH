## Context

NekoSSH tiene shell de dos columnas (sidebar + workspace), perfiles/carpetas SQLite, PTY SSH, explorador SFTP, edición externa y overlays A1/B3. `docs/project_scope.md` describe Fase 4 como “Gestor de Snippets y Mascotas” aún por definir. Tras explore, este change **bloquea solo snippets**: Petdex queda fuera.

El `sidebar-footer` hoy concentra preferencias rápidas (editor externo, fondo, opacidad). El modal de perfil (`#profile-modal` / `.modal` + `.modal-content`) ya define el chrome glass (backdrop blur, border glass, card oscura) que se reutilizará. `confirmDialog` en `overlays.ts` cubre confirms destructivos. Persistencia de app sigue el patrón migración plugin SQL + helpers `rusqlite` con `ensure_*_schema` para tests in-memory (p. ej. `preferences.rs`, migración `003`).

Constraints: código de app solo en `app/`; TDD; español latino en docs/OpenSpec; identificadores en inglés; sin tocar Petdex ni PTY write en este slice.

## Goals / Non-Goals

**Goals:**
- Diccionario local de snippets con categorías de **un solo nivel**, persistido en SQLite.
- Abrir el gestor con **un botón** en `sidebar-footer` (franja inferior izquierda).
- Modal glass estilo profile-modal: lista plana + chips de categoría + búsqueda; CRUD in-modal (categoría y snippet; editar título/cuerpo).
- Copiar cuerpo del snippet al portapapeles; eliminar con confirm A1.
- Seed demo (Apache, Tomcat, Permisos) idempotente cuando no hay datos.
- Actualizar `project_scope.md` y `ui-layout-contract.md` para fijar Fase 4 snippets vs Petdex diferido.

**Non-Goals:**
- Petdex / mascotas.
- Insertar snippet al PTY (`write_ssh_input` u equivalente).
- Atajo de teclado para abrir el modal.
- Categorías anidadas o árbol expandible.
- Menús contextuales dedicados a snippets.
- Sync cloud / import-export masivo.
- Tags multi-categoría por snippet.

## Decisions

### 1. Alcance Fase 4 (producto)
- **Decisión**: Este change entrega **Fase 4a — Gestor de snippets**. Petdex se documenta como **Fase 4b (futuro / diferido)** en `project_scope.md` y la fila Fase 4 del layout contract describe el botón footer + modal, sin tercera columna.
- **Alternativa**: Entregar snippets + Petdex juntos — se descarta (explore acordó Petdex OUT).

### 2. Trigger de apertura
- **Decisión**: Botón icono+label (o icon-button con `aria-label` en español) en `sidebar-footer`, tipográficamente coherente con el footer existente; no shortcut en este slice.
- **Alternativa**: Solo atajo Ctrl+Shift+S — fuera de alcance acordado.
- **Ubicación visual**: franja inferior del sidebar; no ocupar zona del árbol ni del workspace.

### 3. Chrome del modal
- **Decisión**: Reutilizar el patrón `#profile-modal` (`.modal` + `.modal-content` glass). Ancho mayor que 480px si hace falta lista cómoda (p. ej. ~560–640px), sin inventar el look “browse A1” de confirms. Escape / botón Cerrar cierran sin mutar (salvo que un form inline esté a medias — descartar draft al cerrar).
- **Alternativa**: Dialog A1 confirm-style para todo el gestor — se descarta (A1 es para confirms, no CRUD browse).

### 4. Modelo de datos SQLite
- **Decisión**: Tablas propias (no meter JSON en `app_preferences`):
  - `snippet_categories(id INTEGER PK, name TEXT NOT NULL UNIQUE, sort_order INTEGER NOT NULL DEFAULT 0, created_at TEXT)`
  - `snippets(id INTEGER PK, category_id INTEGER NOT NULL REFERENCES snippet_categories(id) ON DELETE CASCADE, title TEXT NOT NULL, body TEXT NOT NULL, sort_order INTEGER NOT NULL DEFAULT 0, created_at TEXT, updated_at TEXT)`
  - Migración `004_snippets.sql` + registro en `lib.rs` migrations; módulo `snippets.rs` con `ensure_snippets_schema` para tests in-memory (mismo patrón que preferencias/folders).
- **Alternativa**: Un solo blob JSON en `app_preferences` — más frágil para CRUD/queries/cascade; se descarta.
- **Seed flag**: seed solo si `COUNT(*)` de categorías (o snippets) es 0; no re-sembrar tras borrado total del usuario (si el usuario borra todo, permanece vacío). Documentar en tasks; si se prefiere “seed una sola vez vía preferencia `snippets_seeded`”, usar esa clave solo si el apply encuentra race en tests — default: vacío ⇒ seed.

### 5. Commands IPC (desktop-commands)
- **Decisión**: Commands Tauri orientativos (nombres en inglés):
  - `list_snippet_categories` / `create_snippet_category` / `delete_snippet_category`
  - `list_snippets` (filtro opcional `category_id`, `query`)
  - `create_snippet` / `update_snippet` / `delete_snippet`
  - `ensure_snippet_seed` (o invocado en `list_*` / setup) — idempotente
- Lectura/escritura vía `rusqlite` + path DB existente (`nekossh.db`), igual que folders/preferences en el invoke path.
- **Alternativa**: Solo plugin SQL desde el frontend — menos tipado y peor TDD Rust; preferir commands Rust.

### 6. UI: lista plana + chips + búsqueda
- **Decisión**:
  - Chips: “Todas” + una chip por categoría; chip activa filtra la lista.
  - Lista plana de filas: título (+ opcional preview truncado del body); acciones por fila: Copiar, Editar, Eliminar (iconos Lucide outline).
  - Campo búsqueda filtra por título y cuerpo (case-insensitive) sobre el conjunto ya filtrado por chip.
  - Acciones globales en el modal: “Nueva categoría”, “Nuevo snippet” → formulario inline o panel inferior/lateral del mismo modal (no segundo modal anidado salvo confirm A1).
- **Alternativa**: Árbol expandible por categoría — acordado OUT.

### 7. Copiar vs insertar PTY
- **Decisión**: Copiar = `navigator.clipboard.writeText(body)` (patrón ya usado en copy de conexión). Feedback mínimo (toast sutil o estado temporal en el botón); **no** invocar write al PTY.
- **Alternativa**: Insertar en terminal activa — diferido a un change futuro.

### 8. Eliminar
- **Decisión**: `confirmDialog` A1. Copy sugerido:
  - Snippet: título “Eliminar snippet”, cuerpo con nombre, primaria destructiva “Eliminar”.
  - Categoría: si tiene snippets, el impacto MUST mencionar que se eliminarán los snippets asociados (CASCADE); si el producto prefiere bloquear delete de categoría no vacía, exigir vaciar antes — **preferir CASCADE con copy claro** para menos fricción en demo seed.
- Escape / Cancelar: no borra.

### 9. Seed demo
- **Decisión**: Categorías y ejemplos (cuerpos realistas cortos, en español o comandos shell típicos):
  - **Apache**: p. ej. reiniciar servicio, test config, ver error log.
  - **Tomcat**: p. ej. status, catalina out tail, deploy path hint.
  - **Permisos**: p. ej. `chmod`, `chown`, `ls -la` útil.
- Todos eliminables/editables como datos de usuario tras el seed.

### 10. Docs SSOT
- **Decisión**: En Step Last de tasks:
  - `project_scope.md`: Fase 4a snippets (este alcance); Fase 4b Petdex diferido.
  - `ui-layout-contract.md`: fila Fase 4 — botón en sidebar-footer + modal snippets; workspace sin cambio de columnas.
  - `DESIGN.md` solo si faltan tokens de chips/lista (reusar tokens existentes primero).

## Risks / Trade-offs

- **[Risk]** Footer ya denso (editor + fondo + opacidad) → el botón de snippets compite por espacio.
  → **Mitigación**: botón compacto (icono Lucide `ClipboardList` o similar + label corto “Snippets”); no añadir más controles en este change.
- **[Risk]** Seed re-inserta tras wipe total del usuario.
  → **Mitigación**: documentar regla vacío⇒seed; si UX lo rechaza en apply, añadir flag `snippets_seed_v1` en `app_preferences`.
- **[Risk]** Clipboard denegado por permisos del WebView.
  → **Mitigación**: capturar error y `alertDialog` A1 amable; no fallar en silencio.
- **[Trade-off]** Sin inserción PTY el valor es “copiar y pegar manual” — aceptable para desbloquear Fase 4a; PTY queda como follow-up claro.
- **[Trade-off]** CASCADE al borrar categoría es destructivo — mitigado con copy A1 explícito.

## Migration Plan

1. Añadir migración `004_snippets.sql` (tablas + índices por `category_id` / `sort_order`).
2. Registrar migración en plugin SQL; `ensure_snippets_schema` en path rusqlite/tests.
3. Deploy = actualizar app; usuarios existentes reciben tablas vacías → seed en primer list/open.
4. Rollback: no bajar migración automáticamente; feature flag no requerida (UI ausente = no commands usados). Datos seed/usuario quedan en DB si se revierte UI.

## Open Questions

- ¿Label exacto del botón: “Snippets” vs “Diccionario”? → Default **Snippets** (corto en footer).
- ¿Borrar categoría vacía sin A1? → Default: siempre A1 por consistencia destructiva.
- ¿Orden de chips/lista: alfabético vs `sort_order`? → Default `sort_order` luego nombre; seed asigna órdenes 0..n.

### Corrección de Ruta (Fix)

**Fecha:** 2026-07-31  
**Motivo:** El modal entregado en apply quedó desalineado del tema Cyber-Sakura y del SSOT (form in-modal). Usuario validó la maqueta `docs/design/preview-snippets-modal.html` (“mucho mejor”).

**Nueva estrategia (obligatoria en apply del fix):**
1. **Tema:** search y botón “+ Snippet” usan el mismo chrome de campos/botones del modal de perfil (borde sakura, fondo card, `btn-primary` / field tokens) — no bordes blancos/nativos ni gris desconectado.
2. **Lista:** filas con separación visual clara (divider / borde de lista).
3. **Fila:** mostrar **título + comando/body** únicamente; **no** repetir el nombre de categoría (los chips ya filtran/contextualizan).
4. **Nueva categoría:** panel/form **dentro del mismo modal** (como nuevo snippet). **Prohibido** `window.prompt` u otro modal del SO.
5. **CSS:** solo selectores `.snippets-*` / `#snippets-modal`; no tocar estilos globales del footer/prefs.
6. **Referencia visual SSOT del fix:** `docs/design/preview-snippets-modal.html`.

### Corrección de Ruta (Fix) — Toolbar vs preview

**Fecha:** 2026-07-31  
**Motivo:** Tras §9, el toolbar **no** coincide con `preview-snippets-modal.html`: el search queda colapsado/casi invisible y “+ Snippet” ocupa casi todo el ancho. El usuario rechazó el resultado y reiteró: **prohibido tocar estilos globales**.

**Causa probable:** reglas de `.snippets-field` con `width: 100%` (y/o flex mal acotado) rompen el flex row del toolbar.

**Nueva estrategia:**
1. Toolbar MUST ser fila: `[ search flex:1 min-width:0 ] [ + Snippet flex-shrink:0 ]` — igual que el preview.
2. En el search del toolbar: **no** usar `width: 100%` que aplaste al hermano; acotar estilos de campo al input del toolbar con selector específico (`#snippets-search` / `.snippets-toolbar .snippets-field`).
3. **HARD CONSTRAINT:** en este fix solo editar `#snippets-modal` / `.snippets-*` (y markup del modal en `index.html` / lógica en `snippets-ui.ts` si hace falta). **Cero** cambios a reglas globales (`input[type=…]`, `.btn-primary` global, footer prefs, etc.).
4. Verificación: comparar side-by-side con `docs/design/preview-snippets-modal.html` antes de marcar done.

### Corrección de Ruta (Fix) — Toolbar aún roto + gate de verificación

**Fecha:** 2026-07-31  
**Motivo:** Tras §10 el usuario reportó que el UI **sigue** sin search visible y “+ Snippet” a ancho completo. Marcar §10 done con solo `npm run build` **no** garantiza paridad con el preview. No hay tests unitarios de layout frontend; la garantía del plan es el preview + criterio de aceptación explícito.

**Estrategia:**
1. Diagnosticar por qué `#snippets-search` no participa en la fila (CSS conflictivo, markup, `display`, orden flex, o reglas de `input[type=search]` fuera de scope que no debemos tocar — preferir override **solo** bajo `#snippets-modal`).
2. Dejar el toolbar **idéntico en estructura al preview**: una fila `display:flex; flex-direction:row` con search creciendo y botón compacto a la derecha.
3. **Acceptance gate (obligatorio antes de `[x]`):**
   - En runtime/devtools o captura: search visible a la izquierda **y** “+ Snippet” a la derecha en la misma fila.
   - Diff CSS del fix limitado a `#snippets-modal` / `.snippets-*` (y markup/TS del modal si hace falta).
   - Si no se puede demostrar paridad con el preview, la task queda `[ ]` — **prohibido** marcar done solo con build verde.
4. Tests: los unitarios Rust de `snippets` **no** cubren layout; no inventar suite UI grande en este fix salvo smoke/manual documentado en report con evidencia (qué se vio vs preview).

### Corrección de Ruta (Fix) — Select categoría (nuevo snippet)

**Fecha:** 2026-07-31  
**Motivo:** En el form in-modal “Nuevo/Editar snippet”, el `<select>` de categoría (`#snippet-form-category` / `.snippets-select`) se ve nativo / desintegrado del tema Cyber-Sakura respecto a los demás campos del modal y al preview.

**Estrategia:**
1. Estilar **solo** `#snippets-modal .snippets-select` (y estados `:focus`, `option` si aplica) para igualar chrome de `.snippets-field` / preview: fondo oscuro, borde sakura, texto primary, chevron sakura, tipografía Outfit, `appearance: none`.
2. Añadir `color-scheme: dark` (u override de `option`) bajo el mismo scope para que la lista desplegable no salga blanca/sistema en Windows.
3. **HARD CONSTRAINT:** no modificar el `select { ... }` global de `styles.css`; solo overrides bajo `#snippets-modal`.
4. Actualizar `docs/design/preview-snippets-modal.html` si el select del preview diverge, para mantener SSOT visual.
5. Gate: select cerrado y abierto se ven alineados a campos del mismo form; report corto; build OK.

### Corrección de Ruta (Fix) — Footer: engrane + Snippets temático

**Fecha:** 2026-07-31  
**Motivo:** El `sidebar-footer` quedó sobrecargado: editor externo, fondo y opacidad viven siempre visibles en la franja, compitiendo con el botón Snippets. Además, el control Snippets no queda integrado al tema Cyber-Sakura (se percibe genérico / poco anclado a tokens).

**Nueva estrategia UX:**
1. **Footer mínimo:** la franja inferior del sidebar muestra acciones primarias + control **Snippets** + **icono de engrane** — sin campos de prefs expandidos en la tira.
2. **Prefs detrás del engrane:** editor externo, fondo y opacidad se mueven a un **popover/panel** abierto por el engrane (no siempre visibles en el footer strip). Misma capacidad funcional; solo se reubica la UI.
3. **Snippets temático:** botón icono+label (clipboard/lista) alineado a tokens Cyber-Sakura / DESIGN / preview (borde sakura, tipografía Outfit, hover sakura) — no botón de sistema genérico.
4. **Gate visual obligatorio:** entregar primero `docs/design/preview-footer-gear-snippets.html` para aprobación del usuario **antes** de que `/opsx:apply` implemente código de app.
5. Relación con §12 (select categoría temático): este fix es **chrome del footer**, independiente; **mantener §12** pendiente salvo que se vuelva obsoleto.

**HARD CONSTRAINT:**
- CSS del popover de prefs y del control Snippets del footer: **scoped** (selectores del footer/popover / `.snippets-footer-*` según markup); no regresionar el trabajo del modal de snippets.
- No borrar funcionalidad de prefs (editor / fondo / opacidad): solo ocultarlas de la tira y mostrarlas en el panel del engrane.
- No tocar CSS global de la app en este paso de plan; el apply implementará con scope acotado.

### Corrección de Ruta (Fix) — Aprobación preview footer engrane

- **Fecha:** 2026-07-31
- **Motivo:** Usuario aprobó la propuesta visual (`preview-footer-gear-snippets.html`) con "sí adelante".
- **Estrategia:** proceder a implementar §13.3–13.5 en el próximo `/opsx:apply` según ese preview (footer mínimo Snippets temático + engrane; prefs en popover). Mantener §12 (select categoría) pendiente o en el mismo apply si el usuario no lo excluyó.
- **No cambiar el scope:** no implementar en este fix step.
