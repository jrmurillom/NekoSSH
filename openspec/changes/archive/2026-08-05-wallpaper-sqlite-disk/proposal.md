## Why

Los wallpapers por tema se guardan hoy como data URL (base64) en `localStorage` (`nekossh-bg-by-theme`). Ese almacén tiene un techo ~5 MB por origen: una sola imagen grande aplica en sesión pero no persiste (“Fondo aplicado, pero no guardado”). Meter binarios de imagen en `localStorage` no es sostenible.

## What Changes

- Persistir metadatos de wallpaper por `theme_id` en **SQLite** (tabla dedicada: label, opacity, nombre/ruta relativa del archivo).
- Guardar una **copia del archivo de imagen** en el directorio de datos de la app (disco), sin comprimir ni redimensionar de forma obligatoria.
- Al pintar, resolver la ruta absoluta y usar `convertFileSrc` (u equivalente asset) — no data URLs en storage.
- CRUD (examinar / aplicar URL http(s) / quitar / opacidad) escribe BD + disco; deja de depender de `localStorage` para el mapa de fondos.
- Migración one-shot desde `nekossh-bg-by-theme` (y residuales legacy si aún existen) hacia BD + disco; luego borrar esas claves.
- Eliminar el flujo de “aplicado pero no guardado por cupo de localStorage” para imágenes locales (el cupo deja de aplicar).

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `app-branding`: la persistencia de wallpaper por tema pasa de almacenamiento local (`localStorage` / data URL) a SQLite + archivo en disco; requisitos de migración y de render vía asset path.
- `conceptual-themes`: al aplicar/restaurar tema, el wallpaper sigue sincronizándose en el mismo ciclo, pero la fuente de verdad deja de ser el mapa en `localStorage`.

## Impact

- Backend Rust: migración SQL nueva, commands IPC (get/set/clear wallpaper por tema, opcionalmente list), escritura/borrado de archivos bajo app data.
- Frontend: `main.ts`, `theme-wallpaper-helper` (adaptar o reemplazar capa de I/O); dejar de serializar data URLs al mapa local.
- Docs: `docs/design/DESIGN.md` (persistencia wallpaper).
- Sin cambio de UI visual del popover salvo mensajes de error (ya no el aviso de cupo localStorage para locales).
