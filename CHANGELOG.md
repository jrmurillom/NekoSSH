# Changelog

Todos los cambios notables de este proyecto se documentan en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [Unreleased]

### Added

- Persistencia de wallpaper por tema en SQLite (`theme_wallpapers`) con copia de imagen en el data dir de la app (`wallpapers/`).
- Commands IPC para get/set/clear de wallpaper (archivo, URL http(s), bytes/data URL de migración, opacidad).
- Migración one-shot desde `localStorage` (`nekossh-bg-by-theme` y claves legacy) hacia BD + disco.
- Protocolo `asset` de Tauri para renderizar wallpapers locales con `convertFileSrc`.
- Smoke `smoke_theme_wallpapers` y tests del módulo Rust/helper frontend.

### Changed

- El fondo de terminal ya no se guarda como data URL en `localStorage` (evita el techo ~5 MB).
- Specs `app-branding` y `conceptual-themes`: fuente de verdad SQLite + disco.
- `docs/design/DESIGN.md`: documenta la nueva persistencia.

### Fixed

- Imágenes grandes de fondo se pueden persistir entre reinicios (ya no “aplicado pero no guardado” por cupo de storage).
