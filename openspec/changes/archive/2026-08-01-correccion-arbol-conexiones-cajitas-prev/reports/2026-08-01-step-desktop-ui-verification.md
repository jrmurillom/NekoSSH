# Verificación desktop-ui — correccion-arbol-conexiones

**Fecha:** 2026-08-01  
**Rama:** `feature/correccion-arbol-conexiones`  
**SSOT visual:** `docs/design/preview-connection-tree-dense.html` (panel «Después»)  
**Harness auxiliar:** `openspec/changes/correccion-arbol-conexiones/reports/harness-connection-tree.html` (sirve las clases runtime + `app/src/styles.css`)

## Gate visual vs preview

Comparación token-a-token (preview «Después» ↔ CSS scoped `.connection-tree`):

| Token | Preview | App CSS | Match |
|-------|---------|---------|-------|
| Gap árbol | `1px` | `gap: 1px` | OK |
| Carpeta padding / min-height | `4px 6px` / `28px` | igual | OK |
| Carpeta hover | tint sakura sutil, sin tarjeta | igual | OK |
| Hijos indent + guía | `ml 14px`, `pl 10px`, `border-left 0.16` | igual | OK |
| Conexión padding / min-height | `5px 8px` / `36px` | igual | OK |
| Conexión reposo | fondo/borde transparentes | igual | OK |
| Hover / selected | sakura 0.07/0.18 y 0.1/0.28 | igual (`.active`) | OK |
| Nombre / endpoint | `0.86rem` / mono `0.68rem` cyan | igual | OK |
| Empty | italic `0.78rem`, sin dashed card | `.folder-empty` | OK |

## Smoke comportamiento (revisión de código + markup)

| Escenario | Evidencia | Resultado |
|-----------|-----------|-----------|
| Expand/collapse fila carpeta | `toggleFolderRow` en `.folder-row` click | Intactos |
| `+` nueva conexión | `stopPropagation` + `openProfileModal` | Intactos |
| Empty «Sin conexiones» | clase `.folder-empty` (sin estilos inline) | OK |
| Copy `user@host` | `copyUserAtHost` + `.btn-copy-endpoint` | Intactos |
| Doble clic conectar | `dblclick` → `startNewSshConnection` | Intactos |
| Menú contextual | `showContextMenu` carpeta/conexión | Intactos |
| Rename inline | inputs `.folder-name-input` / `.profile-name-input` densificados | OK |

## Smoke negativo (snippets / footer)

- CSS del árbol cualificado solo bajo `.connection-tree`.
- Reglas `#snippets-modal` y `.sidebar-footer` **no modificadas** en este change.
- No hay selectores globales residuales `.profile-item` / `.folder-row`.

## Nota de runtime Tauri

No se pudo capturar screenshot del WebView Tauri en esta sesión (browser MCP inestable para tabs locales). La verificación se cerró con: build OK + paridad de tokens con el preview + harness HTTP local (`python -m http.server 8765`) cargando el CSS de la app. Recomendado en handoff: `npm run tauri dev` y contrastar el sidebar con el panel «Después» del preview.

## Conclusión

Árbol denso alineado al SSOT visual del change. Comportamiento sin cambios de lógica. Sin regresión esperada en snippets/footer.
