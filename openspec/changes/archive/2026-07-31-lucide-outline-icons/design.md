## Context

NekoSSH (frontend vanilla TS + Vite en `app/`) usa hoy mezcla de emojis, caracteres Unicode y labels cortos en botones/árbol. Los tokens de color viven en CSS (`DESIGN.md`). Lucide ofrece SVG de **stroke/contorno** por defecto; encaja con herencia `currentColor` para temas.

## Goals / Non-Goals

**Goals:**
- Helper único para montar iconos Lucide outline en el DOM.
- Reemplazar chrome UI listado en proposal (explorador, perfiles, tabs, etc.).
- Color vía `currentColor` + tokens/clase de tema; stroke configurable (grosor/tamaño) desde CSS.
- Documentar convención en DESIGN / layout.

**Non-Goals:**
- Iconos filled/sólidos de Lucide.
- Rediseño completo de layout o nuevos botones no existentes.
- Sustituir caracteres del terminal remoto / xterm.
- React/`lucide-react` (stack es vanilla).

## Decisions

### 1. Paquete `lucide` (vanilla) vs copiar SVG estáticos
- **Decisión**: Dependencia `lucide` + factory TS que crea `<svg>` / usa `createElement` de Lucide.
- **Alternativa**: SVG inline copiados — más control, peor mantenimiento.
- **Razón**: API oficial, tree-shakeable, nombres estables del catálogo.

### 2. Solo outline (stroke)
- **Decisión**: Usar iconos Lucide estándar (stroke); `fill="none"`, `stroke="currentColor"`. Prohibido variantes filled si el catálogo las ofrece como alternativas sólidas.
- **Alternativa**: Icon packs mixtos — se descarta por inconsistencia.

### 3. Color y temas
- **Decisión**: El SVG hereda color del contenedor (`color` CSS / `currentColor`). Clases utilitarias (p. ej. `.icon`, `.icon--muted`, `.icon--danger`) mapean a `--color-text-*`, `--color-error-neon`, etc. Cambio de tema = cambio de tokens; sin repintar JS.
- **Alternativa**: `stroke` hardcodeado en JS — se descarta.

### 4. Mapa de acciones → iconos (propuesta inicial)
| Acción UI | Lucide (outline) |
|-----------|------------------|
| Subir (files) | `ArrowUp` / `CornerLeftUp` |
| Ir | `ArrowRight` |
| Actualizar | `RefreshCw` |
| Expand / collapse | `ChevronRight` / `ChevronDown` |
| Carpeta / archivo | `Folder` / `File` |
| Editar perfil | `Pencil` |
| Eliminar | `Trash2` |
| Cerrar tab | `X` |
| Nuevo / plus | `Plus` |

Ajustes finos en apply si el look no encaja con Cyber-Sakura.

### 5. Integración
- Módulo `app/src/icons.ts` (o similar): `icon(name, opts?) → HTMLElement`.
- Sustituir `textContent` emoji/glifo en `main.ts` / HTML estático donde haya botones icónicos.
- CSS: tamaño (`width`/`height` / `--icon-size`), `stroke-width` coherente.

## Risks / Trade-offs

- **[Risk]** Bundle size si se importa todo Lucide → *Mitigación*: imports nombrados / tree-shaking.
- **[Risk]** Accesibilidad: icon-only sin label → *Mitigación*: mantener `title`/`aria-label` existentes.
- **[Risk]** HTML estático vs JS dinámico → *Mitigación*: hydratar iconos al `DOMContentLoaded` o construir botones desde TS.

## Migration Plan

1. Añadir dep + helper.
2. Sustituir superficies una a una (explorador → perfiles → tabs).
3. Quitar emojis/glifos residuales.
4. Docs DESIGN/layout.
5. Rollback: revert commit / quitar dep (UI vuelve a glifos).

## Open Questions

- ¿Incluir iconos en tabs de sidebar “Servidores” / “Archivos” en este change? **Sí, si ya hay affordance visual; si solo es texto, opcional nice-to-have en apply.**
