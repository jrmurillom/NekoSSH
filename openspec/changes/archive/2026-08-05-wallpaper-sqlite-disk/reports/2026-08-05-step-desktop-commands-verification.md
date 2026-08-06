# Verificación desktop-commands — wallpaper-sqlite-disk

**Change:** wallpaper-sqlite-disk  
**Fecha:** 2026-08-05  
**Rama:** `feature/wallpaper-sqlite-disk`

## Commands / API

| Operación | Evidencia | Resultado |
|-----------|-----------|-----------|
| set file → get | `smoke_theme_wallpapers` + unit `set_file_get_clear_round_trip` | PASS — archivo en `wallpapers/{theme}.ext`, fila `source_kind=file` |
| set url | smoke + unit `set_url_y_opacity` | PASS — `display_url` http(s); archivo previo borrado |
| clear | smoke + unit | PASS — fila eliminada, archivo eliminado |
| data URL → bytes | unit `data_url_persiste_bytes` | PASS (migración) |
| IPC registrados | `get/set_file/set_bytes/set_data_url/set_url/set_opacity/clear_theme_wallpaper_cmd` en `lib.rs` | PASS (compila) |

## Conclusión

PASS a nivel módulo + example. Validación end-to-end de invoke desde WebView queda cubierta en el report desktop-ui (estructural / runtime).
