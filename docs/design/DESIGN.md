---
description: SSOT visual — tokens, tipografía y look de componentes
alwaysApply: false
---

# DESIGN.md — Look & feel (SSOT visual)

Fuente de verdad de **estilo**: colores, tipografía, efectos y apariencia de componentes.  
La **estructura de páginas y zonas** vive en `ui-layout-contract.md` — no mezclar roles.

**Referencia de aceptación (sin glow):** `docs/design/preview-no-glow.html` (modo Sin glow).

---

## 1. Concepto: Cyber-Sakura

Fusión de contraste cyberpunk (fondos profundos, acentos rosa/cian) con suavidad sakura.

Principios:

1. **Translucidez** — capas tipo glass sobre fondos configurables.
2. **Acentos planos** — rosa, cian y verdes/rojos de estado como color sólido o borde; **sin neon glow** (`text-shadow` / `box-shadow` de resplandor).
3. **Alto contraste** — tipografía nítida sobre fondos oscuros para fatiga baja y lectura de terminal.

---

## 2. Tokens (CSS Custom Properties)

Definir en el stylesheet raíz de la app y heredar globalmente. No hardcodear hex en componentes salvo excepciones documentadas (p. ej. canvas de terminal opaco).

```css
:root {
  /* Paleta */
  --bg-dark-base: #0c060d;
  --bg-dark-card: rgba(20, 10, 22, 0.75);
  --color-sakura-neon: #ff69b4;
  --color-sakura-light: #ffb7d5;
  --color-cyan-electric: #00ffff;
  --color-purple-neon: #bd93f9;
  --color-text-primary: #f8f8f2;
  --color-text-muted: #a593ad;
  --color-success-neon: #39ff14;
  --color-error-neon: #ff3131;

  /* Glass (profundidad — no glow de color) */
  --glass-blur: blur(16px);
  --glass-border: 1px solid rgba(255, 105, 180, 0.15);
  --glass-shadow: 0 8px 32px 0 rgba(12, 6, 13, 0.5);

  /* Tipografía */
  --font-sans: 'Outfit', 'Inter', system-ui, -apple-system, sans-serif;
  --font-mono: 'Fira Code', 'JetBrains Mono', monospace;

  /* Forma y movimiento */
  --border-radius-sm: 4px;
  --border-radius-md: 8px;
  --border-radius-lg: 16px;
  --transition-fast: 0.15s ease;
  --transition-normal: 0.3s ease;
}
```

**Prohibido en producción:** tokens `--glow-*` y sombras de resplandor rosa/cian/verde/rojo. Los nombres `*-neon` en la paleta son el **matiz** del acento, no un efecto glow.

**Tipografía dinámica:** la terminal y superficies mono deben leer `--font-mono` en runtime (`getComputedStyle`), no fijar la familia en código.

---

## 3. Superficies y componentes (look)

### Shell / paneles

- Paneles con `backdrop-filter: var(--glass-blur)` y `background: var(--bg-dark-card)`.
- `--glass-shadow` solo como sombra de profundidad (sin tinte neón).
- Fondo de app configurable (imagen o color) con opacidad de imagen en rango `0.1`–`0.4`.
- **Logo del sidebar:** Renderizado plano y nítido a `32x32px` sin efectos de resplandor o desenfoque. Queda estrictamente prohibido aplicar filtros como `drop-shadow` de neón rosa/cian que generen un brillo luminoso, en alineación con el principio de acentos planos "sin glow".

### Terminal

- Fondo de viewport de terminal **opaco** (`#080409` o equivalente) — no glass sobre el stream de texto.
- **Resplandor / Glow del Contenedor Unificado (Excepción de Diseño):** El panel de la terminal (`.terminal-panel`) es la única superficie del producto autorizada a poseer un resplandor luminoso de neón (`box-shadow: 0 0 25px rgba(255, 105, 180, 0.12)` y `border: 1px solid rgba(255, 105, 180, 0.25)`). Esto delimita el bloque de consola unificado de forma premium, manteniendo el glow estrictamente limitado al color sakura del tema.
- Cursor: blink obligatorio; estilo configurable (`block` | `underline` | `bar`); color de acento plano (p. ej. sakura), **sin** `box-shadow` glow.

### Editor (cuando aplique)

- Tema oscuro alineado a tokens (comentarios muted, keywords sakura/cian).
- Números de línea: `--color-text-muted`; línea activa: `--color-sakura-neon`.

### Controles interactivos

- Hover/focus/active: cambio de color de borde/fondo o brightness — **no** glow.
- Nunca colores fuera del token set sin actualizar este archivo.
- Indicador de sesión de terminal (`.status-dot`): `connecting` (sakura light + pulse), `connected` (success), `disconnected` (muted), `error` (error). Copy de desconexión incluye hint Ctrl+R.

### Iconografía (Lucide outline)

- Chrome UI (botones, árbol de archivos, árbol de conexiones/carpetas, acciones de perfil, cerrar pestaña) usa iconos **[Lucide](https://lucide.dev/icons/) de contorno** (stroke), no filled/sólidos ni emojis.
- Color: `stroke: currentColor` / herencia del contenedor → tokens `--color-text-*`, `--color-sakura-*`, `--color-error-neon`, etc. **Prohibido** hex fijo en el SVG del icono.
- Tamaño vía CSS `--icon-size` / clase `.icon`. Implementación: `app/src/icons.ts`.
- Controles solo-icono deben tener `title` y/o `aria-label`.
- Rename inline de carpeta/conexión: input en la fila del árbol (no modal obligatorio); se inicia desde el menú contextual; Enter = guardar, Escape = cancelar. No usar doble clic para renombrar.
- Cajita de conexión (árbol sidebar): fondo/borde/radius de tarjeta en `.connection-tree .profile-item`; endpoint en `--color-cyan-electric`; icono Copy Lucide; sin badge `SSH (Contraseña|Llave)`. Carpetas: filas planas sin borde/caja (solo tint en hover/activo). Header de zona **Conexiones** (`.connections-zone-header`): label en español latino + icon-buttons Lucide para crear conexión/carpeta — patrón de layout del panel Servidores, no un componente glass nuevo.
- Botón **Snippets** en `sidebar-footer` (`.snippets-footer-btn`): fill primario del tema — mismo `linear-gradient(135deg, var(--color-sakura-neon), #d82b7d)` y texto blanco que `.btn-primary`; no ghost/outline. El engrane permanece como icon-button secundario.

### Confirmaciones (dialog)

Patrón canónico: **dialog glass centrado** (referencia de preview: A1 en `docs/design/preview-overlays.html`).

- Overlay oscuro semitransparente; panel centrado con `background: var(--bg-dark-card)`, `backdrop-filter: var(--glass-blur)`, `border: var(--glass-border)`, `box-shadow: var(--glass-shadow)` (profundidad, **sin glow**).
- Estructura: título → cuerpo corto → impacto opcional en mono (p. ej. nombre de carpeta + conteo) → acciones alineadas a la derecha.
- Acciones: **Cancelar** (ghost / borde sutil) + acción primaria o destructiva.
- Destructivo: fondo/borde `--color-error-neon` semitransparente (mismo espíritu que `.btn-danger`); no usar confirm nativo del OS (`window.confirm`) en chrome nuevo.
- Tipografía: Outfit para título/cuerpo; Fira Code solo para líneas de impacto/meta.
- **Prohibido:** diálogos nativos del sistema para flujos de producto; banners sueltos sobre el viewport de terminal como “confirm”.
- **Sync edición externa (Fase 3):** mismo patrón A1 — título “Subir cambios”, cuerpo “¿Subir al servidor?”, detalle = **filename** por defecto + colapsable “ver ruta completa” (textarea readonly con path completo); primaria “Subir”. Aviso binario y errores de transfer también usan A1/`alertDialog`, nunca `window.confirm`.

### Menús contextuales

Patrón canónico: **ítems con glifo/icono Lucide** (referencia de preview: B3 en `docs/design/preview-overlays.html`), reutilizable en sidebar y explorador.

- Contenedor glass: mismos tokens de card que el dialog (`--bg-dark-card`, `--glass-border`, `--glass-shadow`, radius `--border-radius-md`).
- Cada ítem: icono outline a la izquierda + label; padding compacto; sin pills ni multi-shadow.
- **Hover / focus del ítem:** mismo rosa que el botón **“Nueva conexión”** (`.btn-primary` → token `--color-sakura-neon` / `#ff69b4`): fondo semitransparente derivado de ese token + texto `var(--color-sakura-neon)` — **no** `--color-sakura-light` pastel ni hover cian/eléctrico.
- Ítem destructivo: texto/hover con `--color-error-neon` (semitransparente), separado del resto con un separator hairline sakura sutil si hay más de un grupo.
- Iconos: Lucide outline + `currentColor` (ver § Iconografía).

### Scrollbars (chrome UI)

Todo contenedor scrolleable de la app (sidebar, árbol SFTP, modales, paneles) MUST usar scrollbars temáticos Cyber-Sakura — **prohibido** dejar el track/thumb nativo del OS/WebView (p. ej. track blanco).

Tokens (stylesheet raíz):

| Token | Rol |
|-------|-----|
| `--scrollbar-size` | Grosor (~6px) vertical y horizontal |
| `--scrollbar-thumb` / `--scrollbar-thumb-hover` | Thumb sakura semitransparente (sin glow) |
| `--scrollbar-track` | Track transparente |

Implementación: reglas globales `*` + `::-webkit-scrollbar*` y Firefox `scrollbar-width: thin` + `scrollbar-color`. No duplicar por componente salvo excepción documentada (p. ej. canvas de terminal opaco).

---

## 4. Qué actualizar aquí

Cambios de color, tipografía, glass, temas de terminal/editor, o look de controles → este archivo.  
Cambios de zonas del shell, paneles o patrones de página → `ui-layout-contract.md`.
