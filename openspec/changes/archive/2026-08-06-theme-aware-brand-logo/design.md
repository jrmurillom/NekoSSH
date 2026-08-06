## Context

Hoy el sidebar usa un único `logo.png` fijo. `applyTheme` ya sincroniza `data-theme`, `localStorage` y colores de xterm, pero no el logo. Existen PNG por tema en `docs/design/logos/` (~16 KB c/u), exportados desde SVG con la rampa `gradient-end → primary → light`. El id del tema (`nekossh`, `hatsune-miku`, …) es la clave natural del asset.

## Goals / Non-Goals

**Goals:**

- Que el logo del sidebar coincida con el tema activo al boot y al cambiar tema.
- Usar PNG livianos en runtime; nombres = ids del catálogo de 8 temas.
- Fallback seguro a `nekossh` si el id es desconocido.

**Non-Goals:**

- No recolorear SVG en runtime ni usar CSS mask / `currentColor`.
- No cambiar iconos de bundle del instalador (Windows/macOS/iOS/Android).
- No rediseñar el layout del brand (solo el `src` del logo).
- No tocar tokens CSS ni catálogo de temas.

## Decisions

1. **PNG por tema en `app/src/assets/logos/<theme-id>.png`**  
   Runtime solo necesita PNG. Los SVG maestros permanecen en `docs/design/logos/` como fuente de diseño.  
   *Alternativa descartada:* SVG inline / mask CSS (frágil con pixel art denso).

2. **Actualizar `src` desde `applyTheme(themeName)`**  
   Un único punto de sincronización (igual que xterm). El boot ya llama `applyTheme(getActiveTheme())`.  
   *Alternativa descartada:* CSS `content` / `background-image` por `[data-theme]` (más difícil de testear y de fallback).

3. **Helper puro `resolveBrandLogoUrl(themeId)`**  
   Mapa o convención `logos/${id}.png` con fallback a `nekossh`. Unit test del mapeo (ids válidos + desconocido).

4. **Normalizar `sailor_moon.png` → `sailor-moon.png`**  
   Alinear nombre de archivo con el id del tema `sailor-moon`.

5. **HTML inicial**  
   El `<img class="brand-logo">` puede apuntar a `logos/nekossh.png` por defecto; `applyTheme` lo corrige al boot si hay otro tema guardado.

## Risks / Trade-offs

- **[Risk] Asset faltante o mal nombrado** → Mitigation: fallback a `nekossh.png` + checklist de 8 archivos en tasks.  
- **[Risk] Cache del WebView muestra logo viejo** → Mitigation: mismo path por tema (archivos distintos); no hace falta busting de query.  
- **[Trade-off] 8 PNG en el bundle (~130 KB)** → Aceptable frente a 8 SVG de ~240 KB c/u.

## Migration Plan

1. Copiar/renombrar PNG a `app/src/assets/logos/`.  
2. Cablear `applyTheme` + helper.  
3. Dejar `logo.png` legacy solo si algún otro lugar lo usa; si no, retirar o redirigir al nekossh.  
4. Rollback: revertir `applyTheme` y restaurar `src` fijo a un solo PNG.

## Open Questions

- Ninguna bloqueante: el mapeo id→PNG y el punto de enganche (`applyTheme`) están definidos.
