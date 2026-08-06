## Why

Al cambiar de tema conceptual, el título y los tokens CSS se actualizan, pero el logo del sidebar permanece en el PNG rosa fijo. Eso rompe la coherencia visual de marca: el acento del logo debe seguir el tema activo. Ya existen PNG por tema (exportados desde las variantes SVG) listos para usarse en runtime.

## What Changes

- Incorporar al frontend un PNG de logo por cada uno de los 8 temas conceptuales (nombre = id del tema).
- Al aplicar o restaurar un tema, el `.brand-logo` del sidebar cambia su `src` al PNG correspondiente.
- Fallback al logo del tema `nekossh` si el id es desconocido o falta el asset.
- Renombrar/normalizar el asset de Sailor Moon a `sailor-moon.png` (hoy `sailor_moon.png`) para alinear con el id del tema.
- Actualizar requisitos de `app-branding` y `conceptual-themes` para exigir logo temático.

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `app-branding`: el logo del sidebar MUST reflejar el tema conceptual activo, no un asset único fijo.
- `conceptual-themes`: al cambiar o restaurar el tema, el logo de marca MUST sincronizarse junto con CSS y xterm.

## Impact

- Frontend: `app/index.html` (src inicial del logo), `app/src/main.ts` (`applyTheme` / boot), assets bajo `app/src/assets/logos/`.
- Diseño: fuentes en `docs/design/logos/` (SVG/PNG maestros); runtime solo PNG livianos en `app/`.
- Specs: deltas en `app-branding` y `conceptual-themes`.
- Sin cambios de backend Rust, IPC ni BD.
