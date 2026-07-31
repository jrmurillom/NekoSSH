---
description: SSOT visual — tokens, tipografía y look de componentes
alwaysApply: false
---

# DESIGN.md — Look & feel (SSOT visual)

Fuente de verdad de **estilo**: colores, tipografía, efectos y apariencia de componentes.  
La **estructura de páginas y zonas** vive en `ui-layout-contract.md` — no mezclar roles.

---

## 1. Concepto: Cyber-Sakura

Fusión de contraste cyberpunk (fondos profundos, neón) con suavidad sakura (rosas, transparencias).

Principios:

1. **Translucidez** — capas tipo glass sobre fondos configurables.
2. **Neon glow** — acentos intensos en cursor, bordes y estados activos.
3. **Alto contraste** — tipografía nítida sobre fondos oscuros para fatiga baja y lectura de terminal.

---

## 2. Tokens (CSS Custom Properties)

Definir en el stylesheet raíz de la app y heredar globalmente. No hardcodear hex en componentes salvo excepciones documentadas (p. ej. canvas de terminal opaco).

```css
:root {
  /* Paleta */
  --bg-dark-base: #0c060d;
  --bg-dark-card: rgba(20, 10, 22, 0.7);
  --color-sakura-neon: #ff69b4;
  --color-sakura-light: #ffb7d5;
  --color-cyan-electric: #00ffff;
  --color-purple-neon: #bd93f9;
  --color-text-primary: #f8f8f2;
  --color-text-muted: #a593ad;
  --color-success-neon: #39ff14;
  --color-error-neon: #ff3131;

  /* Glass */
  --glass-blur: blur(12px);
  --glass-border: 1px solid rgba(255, 105, 180, 0.15);
  --glass-shadow: 0 8px 32px 0 rgba(12, 6, 13, 0.37);

  /* Glow */
  --glow-sakura: 0 0 8px #ff69b4, 0 0 15px rgba(255, 105, 180, 0.5);
  --glow-cyan: 0 0 8px #00ffff, 0 0 15px rgba(0, 255, 255, 0.5);
  --glow-success: 0 0 8px #39ff14;

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

**Tipografía dinámica:** la terminal y superficies mono deben leer `--font-mono` en runtime (`getComputedStyle`), no fijar la familia en código.

---

## 3. Superficies y componentes (look)

### Shell / paneles

- Paneles con `backdrop-filter: var(--glass-blur)` y `background: var(--bg-dark-card)`.
- Fondo de app configurable (imagen o color) con opacidad de imagen en rango `0.1`–`0.4` para no matar legibilidad.

### Terminal

- Fondo de viewport de terminal **opaco** (`#080409` o equivalente documentado) — no glass sobre el stream de texto.
- Cursor: blink obligatorio; estilo configurable (`block` | `underline` | `bar`); glow del color activo del perfil (p. ej. `box-shadow: var(--glow-sakura)` en bloque).

### Editor (cuando aplique)

- Tema oscuro alineado a tokens (comentarios muted, keywords sakura/cian).
- Números de línea: `--color-text-muted`; línea activa: `--color-sakura-neon`.

### Controles interactivos

- Estados hover/focus/active con glow o borde sakura/cian, nunca colores fuera del token set sin actualizar este archivo.

---

## 4. Qué actualizar aquí

Cambios de color, tipografía, glow, glass, temas de terminal/editor, o look de controles → este archivo.  
Cambios de zonas del shell, paneles o patrones de página → `ui-layout-contract.md`.
