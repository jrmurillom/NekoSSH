# Verificación desktop-ui — wallpaper-sqlite-disk

**Change:** wallpaper-sqlite-disk  
**Fecha:** 2026-08-05  
**Rama:** `feature/wallpaper-sqlite-disk`

## Alcance de esta corrida

Verificación **estructural** + unitarios/smoke (sin ventana Tauri interactiva en el agente).

## Cableado UI

| Flujo | Evidencia en código | Estado |
|-------|---------------------|--------|
| Examinar → path Tauri | `persistBackgroundFromFile` / `File.path` | Cableado |
| Examinar sin path | `arrayBuffer` → `set_theme_wallpaper_bytes_cmd` | Cableado |
| Aplicar http(s) | `persistBackgroundUrl` | Cableado |
| Quitar | `clear_theme_wallpaper_cmd` | Cableado |
| Opacidad | `set_theme_wallpaper_opacity_cmd` | Cableado |
| Cambio de tema | `applyTheme` → `applyThemeWallpaper` → get IPC | Cableado |
| Migración boot | `migrateWallpapersFromLocalStorageIfNeeded` antes de `applyTheme` | Cableado |
| Sin dialog de cupo localStorage | eliminado; errores IPC con dialog real | Cableado |

## Checklist manual pendiente en máquina del usuario

- [ ] Examinar imagen grande → reiniciar app → fondo restaurado
- [ ] Tema A con fondo, tema B vacío, volver a A
- [ ] Quitar limpia panel y no reaparece al reiniciar
- [ ] URL http(s) opcional

## Conclusión

PASS estructural. Runtime interactivo: validar checklist arriba con `npm run tauri dev`.
