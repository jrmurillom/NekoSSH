# Verificación unit tests + DB — wallpaper-sqlite-disk

**Change:** wallpaper-sqlite-disk  
**Fecha:** 2026-08-05  
**Rama:** `feature/wallpaper-sqlite-disk`

## Unitarios

| Suite | Resultado |
|-------|-----------|
| `cargo test theme_wallpapers` (4 tests) | PASS |
| `cargo run --example smoke_theme_wallpapers` | PASS |
| Vitest `theme-wallpaper-helper` + `bg-settings-helper` (22) | PASS |
| `npx tsc --noEmit` | PASS |

## Persistencia

- Migración SQL `006_theme_wallpapers.sql` registrada.
- Tests in-memory + smoke escriben bajo temp `wallpapers/` y limpian (sin mutar `nekossh.db` de producción en esta corrida).
- Baseline/restore DB de usuario: **N/A** (sin mutación de la DB Roaming en verificación automatizada).

## Notas

- Feature Tauri `protocol-asset` + `assetProtocol.scope` en `tauri.conf.json` para `convertFileSrc` sobre app data.
