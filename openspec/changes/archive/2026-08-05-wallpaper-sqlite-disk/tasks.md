**Surface types:** desktop-ui | desktop-commands

## 0. Setup

- [x] 0.1 Crear y cambiar a la rama `feature/wallpaper-sqlite-disk`
- [x] 0.2 Confirmar baseline: wallpapers aún en `localStorage` (`nekossh-bg-by-theme`) + data URL en Examinar

## 1. Schema y persistencia Rust

- [x] 1.1 Añadir migración `006_theme_wallpapers.sql` (tabla `theme_wallpapers`) y registrarla en el runner de migraciones
- [x] 1.2 Implementar módulo/helpers: upsert/get/clear por `theme_id`, copia de archivo a `{app_data}/wallpapers/`, borrado de archivo al clear/replace
- [x] 1.3 Exponer commands IPC: `get_theme_wallpaper`, `set_theme_wallpaper_file`, `set_theme_wallpaper_url`, `clear_theme_wallpaper` (nombres finales alineados al design)
- [x] 1.4 Ajustar capabilities/scopes FS + asset para leer/escribir solo bajo wallpapers / path origen al copiar
- [x] 1.5 Tests Rust (o unitarios del módulo) para CRUD de fila + clear sin panic

## 2. Frontend: dejar localStorage del mapa

- [x] 2.1 Cablear Examinar / Aplicar URL / Quitar / opacidad a los commands IPC (sin persistir data URL)
- [x] 2.2 `applyTheme` / boot: cargar wallpaper del tema vía `get_theme_wallpaper` y pintar con `display_url` / `convertFileSrc`
- [x] 2.3 Migración one-shot al init: `nekossh-bg-by-theme` (+ legacy globales) → IPC/BD+disco; borrar claves localStorage
- [x] 2.4 Adaptar o reducir `theme-wallpaper-helper` (lógica pura que siga siendo testeable sin I/O de storage legado como SSOT)
- [x] 2.5 Eliminar el dialog de “Fondo aplicado, pero no guardado” por cupo de `localStorage` (errores reales de copia/IPC sí se informan)

## 3. Unit tests (N)

- [x] 3.1 Revisar/ajustar tests de `theme-wallpaper-helper` / `bg-settings-helper` afectados
- [x] 3.2 Añadir o actualizar pruebas de normalización/migración pura si aplica

## 4. Verificación unit + DB (N+1) — AGENT MUST EXECUTE

- [x] 4.1 Ejecutar `npm run test` y `cargo test` del área tocada; baseline/restore de `nekossh.db` si hubo mutación
- [x] 4.2 Report en `openspec/changes/wallpaper-sqlite-disk/reports/YYYY-MM-DD-step-4-unit-test-and-db-verification.md`

## 5. Verificación desktop-commands — AGENT MUST EXECUTE

- [x] 5.1 Invocar commands IPC (harness/example/script): set file, get, set url, clear; verificar fila + archivo en disco
- [x] 5.2 Report en `openspec/changes/wallpaper-sqlite-disk/reports/YYYY-MM-DD-step-desktop-commands-verification.md`

## 6. Verificación desktop-ui — AGENT MUST EXECUTE

- [x] 6.1 Validar en runtime: Examinar imagen grande → reinicio restaura; cambio de tema A/B; Quitar; URL http(s) si aplicable
- [x] 6.2 Report en `openspec/changes/wallpaper-sqlite-disk/reports/YYYY-MM-DD-step-desktop-ui-verification.md`

## 7. Docs (Last)

- [x] 7.1 Actualizar `docs/design/DESIGN.md` (persistencia wallpaper: SQLite + disco; sin data URL en localStorage) según documentation-standards
