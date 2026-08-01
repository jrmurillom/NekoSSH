## Why

La UI de NekoSSH aún usa emojis, glifos Unicode (`↑` `→` `↻` `▶`) y texto (`DIR`/`FIL`) como “iconos”. Eso no escala con temas, no es consistente y no sigue el lenguaje visual Cyber-Sakura. Integrar [Lucide](https://lucide.dev/icons/) (variante de **contorno**) permite iconografía uniforme cuyo color hereda tokens CSS (`currentColor` / variables de tema).

## What Changes

- Añadir dependencia Lucide en el frontend (`app/`) y un helper de iconos de contorno (stroke, no filled/sólidos).
- Reemplazar iconos/emojis/glifos de chrome UI: explorador (subir, ir, actualizar, expand/collapse, carpeta/archivo), perfiles (editar, eliminar), pestañas (cerrar), y otros botones icónicos existentes.
- Colorear iconos vía tokens de tema (`--color-*` / `currentColor`); sin fills fijos ni colores hardcodeados en SVG.
- Actualizar `docs/design/DESIGN.md` / layout contract con la convención de iconos Lucide outline + tema.
- **No** cambiar lógica SSH/SFTP; solo presentación de iconos.

## Capabilities

### New Capabilities
- `ui-icons`: Sistema de iconos Lucide de contorno en la UI desktop; herencia de color por tema; mapa de iconos por acción/superficie.

### Modified Capabilities
- *(ninguno a nivel de requisito de producto SSH/SFTP; el cambio es de presentación UI.)*

## Impact

- Código: `app/package.json`, `app/src/` (TypeScript/HTML/CSS), helper de iconos nuevo.
- Dependencia: `lucide` (oficial, compatible con Vite + TS vanilla).
- Docs: `docs/design/DESIGN.md`, `docs/design/ui-layout-contract.md`, `README.md` si aplica.
- Fuera de alcance: iconos del SO, favicon de instalador, glifos dentro del stream PTY/xterm.
