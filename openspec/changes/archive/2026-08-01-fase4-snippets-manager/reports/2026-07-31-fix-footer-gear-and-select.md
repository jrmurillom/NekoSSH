# §12 + §13 — Select categoría + footer engrane / Snippets

Fecha: 2026-07-31  
Change: `fase4-snippets-manager`  
Referencias: `docs/design/preview-footer-gear-snippets.html`, `docs/design/preview-snippets-modal.html`

## §12 — Select categoría (Cyber-Sakura)

### 12.1 Implementación
Solo bajo `#snippets-modal .snippets-select` (sin tocar `select` global):
- `color-scheme: dark`
- fondo `rgba(0,0,0,0.4)` + chevron sakura (paridad con `.snippets-field`)
- padding `9px 32px 9px 12px`, `border-radius: var(--border-radius-md)`, `font-size: 0.9rem`
- `:focus` / `:focus-visible` → `border-color: var(--color-sakura-neon)`
- `option` con fondo `#140a16` y texto primary

### 12.2 Preview
`preview-snippets-modal.html` alineado (color-scheme, option, focus).

### Evidence gate (select)
Al abrir modal → “+ Snippet”: el `<select class="snippets-select">` debe verse como campo sakura oscuro (no nativo claro del SO), con focus rosa y opciones oscuras.

## §13 — Footer engrane + Snippets temático

### 13.3 Implementación
- Strip inline de prefs **oculto** por defecto (relocated a `#prefs-popover`).
- Botón `#btn-footer-gear` (icono Lucide `Settings`) abre/cierra `.prefs-popover.is-open`.
- Capacidades preservadas: editor path Guardar, fondo Aplicar, slider opacidad (mismos IDs / wiring en `initSettings`).
- Cierre: click fuera + Escape; CSS scoped bajo `.sidebar-footer`.

### 13.4 Snippets footer
`.sidebar-footer .snippets-footer-btn` con gradiente sakura, borde, tipografía y hover alineados al preview aprobado. Sin regresión del modal (`#snippets-modal`).

### Evidence gate (footer)
- **Cerrado:** solo fila `[ Snippets | ⚙ ]`; sin campos de editor/fondo/opacidad visibles.
- **Abierto:** popover encima del strip con título “Preferencias” + tres controles.
- Botón Snippets con tono sakura (no `btn-icon-text` transparente genérico).

## Build (12.3 / 13.5)

```
npm run build
> tsc && vite build
✓ built in ~2.18s
```

Exit code: 0.

## Verificación manual recomendada
1. Arrancar app desktop.
2. Footer: Snippets temático + engrane; click engrane → prefs; Guardar/Aplicar/opacidad siguen funcionando.
3. Snippets → + Snippet → select categoría temático.
4. Confirmar que `#btn-snippet-new` sigue en `width: auto` (toolbar no regresa).
