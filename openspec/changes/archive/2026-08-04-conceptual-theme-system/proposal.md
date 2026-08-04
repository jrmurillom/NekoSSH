## Why

Actualmente NekoSSH tiene una única paleta visual (Cyber-Sakura) hardcoded en `:root` de `styles.css` y colores de terminal fijados directamente en el objeto `theme` de xterm.js dentro de `main.ts`. No existe infraestructura para cambiar de tema ni un selector en la UI. Se necesita un sistema de temas conceptuales —paletas inspiradas en personajes y universos anime/cyberpunk reconocibles— que permita al usuario personalizar la estética completa de la aplicación desde el popover de preferencias.

## What Changes

- **Desacoplar los colores hardcoded**: Extraer los tokens CSS de `:root` a un esquema por tema y eliminar los colores literales del objeto `theme` de xterm.js en `main.ts`, haciéndolos leer desde CSS custom properties.
- **Infraestructura de theming**: Implementar un mecanismo basado en atributo `data-theme` en `<html>` que sobreescriba los tokens CSS por tema, con persistencia en `localStorage`.
- **Selector de tema en el popover de preferencias**: Agregar un listado de temas en `#prefs-popover` con preview visual (bolita split-color que muestra los colores dominantes del tema).
- **8 temas conceptuales**:
  - 🌸 **NekoSSH** (default — la Cyber-Sakura actual)
  - 🩵 **Hatsune Miku** — turquesa/cyan dominante, negro, rosa acento
  - 🤍 **Rei Ayanami** — blanco azulado, azul profundo, rojo sutil
  - 💜 **Neon Evangelion (Unidad-01)** — verde neón, púrpura oscuro, naranja acento
  - 🟡 **Cyberpunk David** — amarillo dorado, negro, rojo intenso
  - 🩷 **Cyberpunk Lucy** — rosa/magenta, blanco plateado, azul eléctrico
  - 🔴 **Persona 5 (Phantom Thieves)** — rojo carmesí, negro absoluto, blanco puro
  - 🌙 **Sailor Moon Serena** — dorado cálido, azul marino, rosa pastel

## Capabilities

### New Capabilities
- `conceptual-themes`: Sistema de temas conceptuales con infraestructura de switching dinámico (CSS custom properties por tema, selector UI con preview split-color, persistencia en localStorage, sincronización de colores xterm.js desde tokens CSS).

### Modified Capabilities
- `app-branding`: El popover de preferencias (`#prefs-popover`) se extiende con una nueva sección de selección de tema, y los colores del terminal dejan de estar hardcoded para leerse desde los tokens CSS del tema activo.

## Impact

- **`app/src/styles.css`**: Se reorganiza `:root` como tema default y se crean bloques `[data-theme="<nombre>"]` para cada tema alternativo. Se añaden estilos del selector de tema.
- **`app/src/main.ts`**: Se elimina el objeto `theme` hardcoded de xterm.js; se crea lógica para leer tokens CSS y aplicarlos al terminal. Se añade inicialización y lógica del selector de temas.
- **`app/index.html`**: Se añade la sección de selección de tema dentro del `#prefs-popover`.
- **`docs/design/DESIGN.md`**: Se documenta la arquitectura de theming y las paletas de cada tema conceptual.
