# Desktop UI verification

**Change:** `chrome-confirms-context-menus`  
**Fecha:** 2026-07-31  
**Rama:** `feature/chrome-confirms-context-menus`

## Método

1. `npm run build` en `app/` → OK.
2. Inspección de código + CSS contra SSOT (`DESIGN.md` § Confirmaciones / Menús contextuales, `preview-overlays.html` A1/B3).

## Checklist

| Ítem | Evidencia | Estado |
|------|-----------|--------|
| Dialog A1 glass | `.chrome-dialog-root` / overlay / panel glass / ghost+danger/primary | PASS |
| Escape / overlay cancelan | `confirmDialog` / `alertDialog` en `overlays.ts` | PASS |
| Sin `window.confirm` / `alert` en producto | Grep `main.ts` limpio; usa `confirmDialog` / `alertDialog` | PASS |
| Menú B3 Lucide + hover sakura | `.chrome-context-item:hover` → `--color-sakura-neon` (mismo rosa que “Nueva conexión”); danger separado | PASS |
| Carpeta: solo `+` en fila | Trash inline removido; contextmenu Renombrar/Eliminar | PASS |
| Carpeta: sin dblclick rename | Listener eliminado; rename solo desde menú | PASS |
| Conexión: copy queda; sin lápiz/basurero | `buildProfileItem` solo copy + menú Editar/Renombrar/Eliminar | PASS |
| Conexión: dblclick conecta | Listener dblclick → `startNewSshConnection` (bloqueado si renaming) | PASS |
| Rename inline conexión | `renamingProfileId` + Enter/Escape/blur | PASS |
| Layout contract / DESIGN | Actualizados (sin basurero/lápiz inline; rename desde menú) | PASS |

## Resultado

**PASS** (build + inspección estática). Runtime visual en Tauri no ejecutado en esta pasada; listo para smoke manual en desktop.

---

## Fix (opsx:fix) — hover B3 = rosa “Nueva conexión”

**Problema:** hover de `.chrome-context-item` usaba `--color-sakura-light` (pastel `#ffb7d5`), distinto del rosa del CTA.

**Token canónico (botón `#btn-new-profile.btn-primary`):** `--color-sakura-neon` (`#ff69b4`); hover del botón = `filter: brightness(1.15)` sobre ese gradient.

**Corrección:** `.chrome-context-item:hover` / `:focus-visible` → `color: var(--color-sakura-neon)` + fondo `color-mix(..., var(--color-sakura-neon) 14%, transparent)`. SSOT/preview/contract alineados. `npm run build` OK.
