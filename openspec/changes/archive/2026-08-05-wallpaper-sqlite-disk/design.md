## Context

Hoy cada wallpaper por tema se serializa (a menudo como data URL) en `localStorage` bajo `nekossh-bg-by-theme`. El cupo del origen (~5 MB) hace fallar `setItem` con imágenes grandes: se pintan en sesión y se pierden al reiniciar. El change archivado `per-theme-wallpaper` ya fijó el modelo “un fondo por `theme_id`”; este change solo cambia **dónde** viven metadatos y bytes.

Preferencias de app ya usan SQLite (`app_preferences`). Perfiles y snippets también. El data dir de Tauri (`app.path().app_data_dir()`) ya se usa para temps de edición externa.

## Goals / Non-Goals

**Goals:**

- Metadatos de wallpaper por tema en SQLite.
- Copia del archivo de imagen en disco bajo el data dir de la app (sin comprimir/redimensionar obligatorio).
- URLs `http(s)` persistidas como URL en BD (sin descargar a disco en v1).
- Render con `convertFileSrc` para archivos locales copiados; URLs remotas sin transformación.
- Migración one-shot desde `nekossh-bg-by-theme` / legacy residuales.
- Quitar dependencia de data URLs en `localStorage` para wallpapers.

**Non-Goals:**

- Comprimir o redimensionar imágenes al guardar.
- Guardar BLOB de imagen dentro de SQLite.
- Mover `nekossh-theme` (id de tema activo) fuera de `localStorage` (sigue siendo un string pequeño).
- CDN / sync en la nube.
- Cambiar el lugar de pintado (sigue `.terminal-panel`).
- Reintroducir protocolo asset para rutas arbitrarias del usuario sin copiar (evita dependencias a archivos que el usuario mueve/borra).

## Decisions

1. **Tabla `theme_wallpapers` (no key/value genérico)**  
   Columnas: `theme_id` PK, `label`, `opacity`, `source_kind` (`file` | `url`), `file_name` (relativo, nullable), `remote_url` (nullable), `updated_at`.  
   *Alternativa descartada:* meter JSON en `app_preferences` — peor para consultas/borrado por tema y para integridad file+row.  
   *Alternativa descartada:* BLOB en SQLite — hincha la DB y complica backups.

2. **Directorio de archivos**  
   `{app_data_dir}/wallpapers/{theme_id}.{ext}` (ext desde el archivo origen). Un archivo por tema; al reemplazar, sobrescribir o borrar el anterior.  
   *Alternativa:* hash del contenido — innecesario si hay 1 wallpaper por tema.

3. **IPC Rust**  
   Commands mínimos, p. ej.:
   - `get_theme_wallpaper(theme_id)` → `{ label, opacity, display_url, source_kind }` donde `display_url` ya es usable en WebView (`convertFileSrc` resuelto en Rust o path absoluto + convert en front).
   - `set_theme_wallpaper_file(theme_id, source_path, label, opacity)` → copia a `wallpapers/`, upsert fila.
   - `set_theme_wallpaper_url(theme_id, url, label, opacity)` → upsert sin archivo.
   - `clear_theme_wallpaper(theme_id)` → borra fila + archivo si existía.
   - `list_theme_wallpapers()` opcional para boot (mapa completo) o N gets al cambiar tema.  
   Preferible **un get por tema** en `applyTheme` + get al boot del tema activo (simple).

4. **Flujo Examinar (file picker)**  
   Usar `File.path` (Tauri) cuando exista y llamar `set_theme_wallpaper_file`. Si no hay path, fallback: bytes vía command de escritura (mismo destino en disco). **No** persistir data URL.

5. **HTTP(S)**  
   Solo guardar la URL en BD; el WebView la carga en runtime. Sin caché local en v1.

6. **Migración**  
   Al boot (o primer init de settings):
   1. Si existe `nekossh-bg-by-theme`, por cada entry:
      - data URL → decodificar a archivo + fila `file`.
      - http(s) → fila `url`.
      - path de disco legacy (raro) → copiar si el archivo existe, si no omitir.
   2. Borrar claves `nekossh-bg-by-theme` y legacy `nekossh-bg-*` si aún quedan.
   3. Si la migración de una entry falla, log + continuar con las demás (no tumbar la app).

7. **Permisos Tauri**  
   Scope FS de lectura del path origen al copiar + escritura bajo `app_data_dir/wallpapers`. Asset protocol / `convertFileSrc` para servir solo esa carpeta (o path absoluto bajo app data).

## Risks / Trade-offs

- **[Risk]** Imagen muy grande en disco / memoria al pintar → Mitigación: aceptado en v1 (usuario elige el archivo); no es el cupo de localStorage.
- **[Risk]** Fallo al copiar (permisos, path UNC) → Mitigación: error claro en dialog; no dejar fila huérfana sin archivo (transacción lógica: copiar primero, luego upsert; si upsert falla, borrar archivo nuevo).
- **[Risk]** Migración data URL enorme agota memoria al decodificar → Mitigación: best-effort; si falla, entry se pierde (igual que hoy al reiniciar tras QuotaExceeded).
- **[Risk]** URL remota rota / CORS → Mitigación: mismo comportamiento actual de URLs http(s); fuera de alcance endurecer.

## Migration Plan

1. Añadir migración `006_theme_wallpapers.sql` + registro en el runner existente.
2. Deploy código que lee BD primero; si tabla vacía y hay localStorage, migrar.
3. Rollback: revertir código; filas/archivos huérfanos no rompen la app vieja. Las claves localStorage ya borradas no se recuperan (aceptable).

## Open Questions

- Ninguna bloqueante: acordado no comprimir; BD metadatos + disco bytes; http(s) sin descarga.
