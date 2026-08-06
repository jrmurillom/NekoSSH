# Changelog

Todos los cambios notables de este proyecto se documentan en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [Unreleased]

## [0.1.2] - 2026-08-06

### Added

- Pestaña "Monitor" en la barra lateral con visualización de recursos en tiempo real.
- Gráficos Canvas dinámicos con efecto de brillo neón Cyber-Sakura para CPU y RAM.
- Visualización de almacenamiento de Disco Duro con barra de progreso clásica.
- Monitoreo en tiempo real de velocidad de red (descarga/subida) y deltas por segundo.
- Módulo de "Top Procesos" ordenados por consumo de CPU con distintivos de memoria y carga.
- Visualización de Uptime y Sistema Operativo del servidor remoto.
- Integración nativa de la iconografía de Lucide (`Cpu`, `Database`, `HardDrive`, `Network`, `Clock`, `Server`, `Activity`, `Crown`, `Play`, `Pause`).
- Controles de refresco (2s, 5s, 10s) y botón de pausa/reanudar interactivo.

## [0.1.1] - 2026-08-05

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

## [0.1.0] - 2026-08-05

- Release inicial publicado en GitHub.
