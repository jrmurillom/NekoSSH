# Verificación desktop-ui — theme-aware-brand-logo

**Change:** theme-aware-brand-logo  
**Fecha:** 2026-08-05  
**Rama:** `feature/theme-aware-brand-logo`

## Checklist

| Caso | Evidencia | Resultado |
|---|---|---|
| 8 PNG runtime con id = tema | `app/src/assets/logos/*.png` (8 archivos; `sailor-moon.png` normalizado) | PASS |
| HTML inicial apunta a nekossh | `index.html` → `/src/assets/logos/nekossh.png` | PASS |
| `applyTheme` actualiza `.brand-logo` | `main.ts`: `brandLogo.src = resolveBrandLogoUrl(...)` | PASS |
| Boot restaura tema + logo | `applyTheme(getActiveTheme())` en init | PASS |
| Fallback id desconocido | unit tests `resolveBrandLogoThemeId` / `resolveBrandLogoUrl` | PASS |
| Bundle incluye los 8 logos | `npm run build` lista los 8 PNG en `dist/assets/` | PASS |

## Nota

La verificación de esta sesión es estructural + build + unitarios (sin ventana Tauri interactiva). El cableado garantiza que al cambiar tema en preferencias o al restaurar desde `localStorage`, el logo cambia en el mismo ciclo que CSS y xterm.
