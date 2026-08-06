## Context

Hoy el fondo de terminal usa tres claves globales en `localStorage` (`nekossh-bg-url`, `nekossh-bg-label`, `nekossh-bg-opacity`) y una variable en memoria `bgImageUrl`. `applyTheme` ya sincroniza CSS, xterm y logo, pero no el wallpaper. El usuario quiere un fondo independiente por tema conceptual: al cambiar de tema vuelve el fondo que tenía configurado ahí.

## Goals / Non-Goals

**Goals:**

- Persistir imagen + etiqueta + opacidad por id de tema.
- Al cambiar/restaurar tema, pintar el fondo de ese tema (o vacío).
- Guardar/quitar/ajustar solo afecta el tema activo.
- Migrar una vez las claves globales al tema correspondiente.
- Helper puro testeable para load/save/migrate.

**Non-Goals:**

- No fondos “oficiales” embebidos por tema (siguen siendo del usuario).
- No cambiar el lugar de pintado (sigue siendo `.terminal-panel`).
- No tocar logo por tema ni catálogo de temas.
- No usar SQLite para wallpapers.

## Decisions

1. **Un JSON por mapa de temas en `localStorage`**  
   Clave única p. ej. `nekossh-bg-by-theme` con forma `{ [themeId]: { url, label, opacity } }`.  
   *Alternativa descartada:* tres claves prefijadas por tema (`nekossh-bg-url:hatsune-miku`) — más frágil al enumerar/migrar.

2. **Opacidad también por tema**  
   Forma parte del “look” del wallpaper; si solo la imagen fuera por tema, al cambiar de tema el slider global ensuciaría el otro look.

3. **Enganche en `applyTheme(themeName)`**  
   Tras setear tema/logo/xterm, cargar el entry del tema y llamar `applyBackgroundSettings` + actualizar inputs del popover si existen.

4. **Migración one-shot**  
   Si existe `nekossh-bg-url` (o opacity) y el mapa aún no tiene entry migrado: copiar al tema activo actual (o `nekossh` si no hay tema) y eliminar/ignorar las claves globales (marcar flag `nekossh-bg-migrated` o simplemente borrar globales tras copiar).

5. **Helper puro**  
   `getThemeWallpaper`, `setThemeWallpaper`, `clearThemeWallpaper`, `migrateLegacyWallpaper` en módulo testeable; `main.ts` solo I/O de DOM/`localStorage`.

## Risks / Trade-offs

- **[Risk] Data URLs grandes × 8 temas llenan localStorage** → Mitigation: mismo aviso actual al fallar `setItem`; la imagen de la sesión sigue viva; no duplicar innecesariamente.  
- **[Risk] Usuario espera opacidad global** → Mitigation: documentar en DESIGN; comportamiento alineado al “look por tema”.  
- **[Risk] Migración mal aplicada dos veces** → Mitigation: borrar claves legacy tras migrar o flag explícito.

## Migration Plan

1. En boot/`initSettings`: leer legacy → poblar mapa del tema activo → quitar legacy.  
2. Rollback: revertir código; las claves nuevas no rompen la app vieja (quedan huérfanas).

## Open Questions

- Ninguna bloqueante: el alcance (imagen + opacidad por tema) queda fijado en este design.
