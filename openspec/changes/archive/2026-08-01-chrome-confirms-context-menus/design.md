## Context

SSOT visual ya define confirm A1 (dialog glass) y menú B3 (glifo + hover sakura). El código aún usa `confirm()`/`alert()` y botones inline en el árbol. Este change implementa esos patrones y limpia la chrome del árbol.

## Goals / Non-Goals

**Goals:**
- Helper(s) de UI: `confirmDialog` (A1) y `showContextMenu` (B3).
- Migrar deletes / “cerrar todas” / errores de producto que hoy usan nativos.
- Carpetas: fila = chevron + icono + nombre + `+`; menú: Renombrar, Eliminar.
- Conexiones: fila = nombre + host + copy; menú: Editar (modal perfil), Renombrar (inline), Eliminar; conectar sigue por doble clic en tarjeta.
- Rename inline sin depender de doble clic (Enter/Escape/blur como hoy).

**Non-Goals:**
- Rediseñar el modal de perfil completo.
- Menú contextual rico en SFTP más allá de reutilizar el mismo helper si es trivial (mínimo: árbol de conexiones + confirms globales).
- Toasts no solicitados; host keys; drag-and-drop.

## Decisions

### 1. Confirm = A1 únicamente
- Overlay + panel glass centrado; Cancelar / acción; Escape y click en overlay cancelan.
- API Promise\<boolean\> o equivalente; reemplaza `window.confirm`.
- Errores que hoy son `alert`: dialog A1 de un botón “Entendido” (o el mismo shell con una sola acción) — no OS alert.

### 2. Context menu = B3 + hover sakura
- Posicionar cerca del pointer; cerrar con Escape / click fuera / elegir ítem.
- Ítems: Lucide + label; danger separado; hover fondo/texto sakura (como `.btn-primary`), no cian.

### 3. Trigger de rename
- Ítem de menú “Cambiar nombre” / “Renombrar” → entra modo inline (mismo input que el rename actual).
- Quitar listeners de `dblclick` para rename en carpeta y no introducir dblclick-rename en conexiones.

### 4. Editar conexión
- “Editar” en menú → `openProfileModal(profile)` (formulario existente).
- “Renombrar” → solo nombre inline (si se implementa rename de conexión; si el nombre solo se edita en el modal, entonces “Editar” abre modal y no hay rename inline de conexión — **preferir**: Renombrar inline + Editar = modal completo para no perder campos host/auth).

### 5. Copy
- El botón copy `user@host` permanece en la tarjeta (no solo en menú).

## Risks / Trade-offs

- **[Risk]** Click derecho vs click de collapse en carpeta → *Mitigación*: `contextmenu` + `stopPropagation` en acciones; collapse solo en click primario.
- **[Risk]** Focus trap incompleto en dialog → *Mitigación*: Escape + botón Cancelar mínimo; mejorar focus trap si cabe en tasks.
- **[Trade-off]** SFTP menu puede quedar en estilo viejo un momento si no se migra en el mismo change — preferir reusar helper en SFTP en la misma pasada si es barato.

## Migration Plan

1. Helpers + estilos según SSOT.
2. Wire árbol + replaces de confirm/alert.
3. Verificar build + reports desktop-ui.
4. Rollback: revert frontend.

## Open Questions

- ¿“Renombrar” conexión además de “Editar” modal? **Sí** (decisión 4).
- ¿Migrar menú SFTP en este change? **Sí si el helper es compartido** (una task dedicada).

### Corrección de Ruta (Fix)

**Problema:** el hover de ítems B3 (`.chrome-context-item:hover`) no coincidía con el rosa del botón **“Nueva conexión”**. Se usaba un acento sakura genérico (`--color-sakura-light` / pastel) que se ve distinto del rosa del CTA.

**Hallazgo (CSS real del botón):** `#btn-new-profile` usa clases `btn-primary btn-primary-with-icon`. El rosa canónico es:

- `background: linear-gradient(135deg, var(--color-sakura-neon), #d82b7d)` en `.btn-primary`
- hover: `filter: brightness(1.15)` (no cambia de token; el rosa base sigue siendo `--color-sakura-neon` = `#ff69b4`)

**Estrategia:** alinear `.chrome-context-item:hover` / `:focus-visible` al mismo token `--color-sakura-neon` (texto hover + fondo semitransparente derivado de ese rosa). Actualizar SSOT (`DESIGN.md` § Menús contextuales) y preview B3 (`preview-overlays.html`) si nombraban `--color-sakura-light` u otro token distinto. No inventar un tercer rosa.
