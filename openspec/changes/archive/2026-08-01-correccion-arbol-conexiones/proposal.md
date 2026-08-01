## Why

El árbol de conexiones del sidebar se lee como tarjetas decorativas gruesas (herencia visual de `manager-profiles`), no como lista densa. Eso contradice el contrato de layout («densidad de lista, no tarjetas decorativas») y dificulta leer la jerarquía carpeta → conexiones. Ya hay un preview aprobado como SSOT visual (`docs/design/preview-connection-tree-dense.html`); este change formaliza e implementará ese estilo sin tocar comportamiento ni el trabajo de snippets/footer.

## What Changes

- Restilizar carpetas y conexiones del árbol como **filas densas de lista** (padding/altura reducidos, sin fondo/borde de tarjeta permanente en reposo).
- Reforzar lectura padre/hijo: indentación de hijos + guía vertical + agrupación por bloque de carpeta (como en el preview).
- Mantener comportamiento existente: crear conexión, carpetas, expand/collapse, vacío «Sin conexiones», copiar host, doble clic para conectar, menús contextuales, rename inline.
- Alinear copy de specs/contrato: de «tarjeta compacta» a **fila densa** / lista, sin cambiar reglas de interacción.
- Actualizar SSOT de diseño tocados (`ui-layout-contract.md` y, si aplica, nota en `DESIGN.md`) para fijar densidad de lista en el árbol.
- **Fuera de alcance:** footer, snippets, modal de snippets, cambios de persistencia/IPC, lógica de negocio nueva.

## Capabilities

### New Capabilities

_(ninguna)_

### Modified Capabilities

- `connection-profiles`: el requisito de presentación en el árbol pasa de «tarjeta compacta» a **fila densa de lista** (nombre + endpoint cyan + copy; sin chrome de tarjeta decorativa), conservando interacciones (doble clic, menú, rename).
- `connection-folders`: aclarar que la fila de carpeta y sus hijos se presentan con densidad de lista e indicadores de jerarquía (indent + guía), sin estilo de tarjeta decorativa.

## Impact

- **Frontend:** principalmente `app/src/styles.css` bajo selectores del árbol (`.connection-tree`, `.folder-*`, `.profile-item` / filas de conexión); markup mínimo en `app/src/main.ts` / `app/index.html` solo si hace falta alinear clases con el preview.
- **Docs:** `docs/design/ui-layout-contract.md` (lenguaje «tarjeta» → fila densa); opcionalmente `DESIGN.md` si hay patrón de lista lateral.
- **SSOT visual de implementación:** `docs/design/preview-connection-tree-dense.html` (gate visual antes de cerrar tasks de UI).
- **Sin impacto esperado:** backend Rust, SQLite, IPC, modal de snippets, `sidebar-footer`.
- **Riesgo:** CSS global que afecte `.profile-item` fuera del árbol o estilos compartidos con snippets — mitigar con selectores anidados bajo `.connection-tree`.
