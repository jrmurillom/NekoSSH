# Verificación desktop-ui — per-theme-wallpaper

**Change:** per-theme-wallpaper  
**Fecha:** 2026-08-05  
**Rama:** `feature/per-theme-wallpaper`

## Checklist

| Caso | Evidencia | Resultado |
|---|---|---|
| Guardar fondo solo en tema activo | `persistBackground` → `setThemeWallpaper(map, getActiveTheme(), …)` | PASS |
| Cambiar a otro tema restaura su entry | `applyTheme` → `applyThemeWallpaper(themeName)` | PASS |
| Tema sin entry → sin imagen | `getThemeWallpaper` vacío + `applyBackgroundSettings("", …)` | PASS |
| Quitar no borra otros temas | `clearThemeWallpaper` solo del id activo; unit test aislamiento | PASS |
| Boot migra legacy antes de pintar | `DOMContentLoaded`: `migrateLegacyWallpaperIfNeeded` → `applyTheme` | PASS |
| Opacidad por tema | `persistBackgroundOpacity` escribe opacity en el entry del tema activo | PASS |
| Controles del popover al cambiar tema | `syncWallpaperControls` desde `applyThemeWallpaper` | PASS |

## Nota

Verificación estructural + unitarios en esta sesión (sin ventana Tauri interactiva). El cableado garantiza el ciclo tema ↔ wallpaper en el mismo `applyTheme` que CSS/logo/xterm.
