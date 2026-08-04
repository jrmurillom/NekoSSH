## Why

Los mensajes de estado del explorador de archivos (`#files-status`: "Cargando…", "Error al expandir", "Archivo subido al servidor", "Copiado al portapapeles scp", etc.) están posicionados en el flujo vertical entre la barra de ruta y el árbol de archivos. Esto provoca un desplazamiento del árbol (CLS – Cumulative Layout Shift) cada vez que un mensaje aparece o desaparece, generando una experiencia visual inestable y poco profesional.

## What Changes

- Convertir el elemento `#files-status` en un overlay flotante posicionado absolutamente en la parte inferior del panel `#panel-files`, sin ocupar espacio en el flujo vertical.
- Aplicar estética HUD cyberpunk-sakura (glassmorphism, borde neón, backdrop-filter blur, animación de glow sutil).
- Implementar auto-dismiss con fade-out para mensajes de éxito/info (3 segundos), mientras que los mensajes de error persisten hasta la próxima acción.
- Garantizar cero desplazamiento del árbol bajo cualquier circunstancia.

## Capabilities

### New Capabilities
_(ninguna)_

### Modified Capabilities
- `sftp-explorer`: El requisito de feedback visual del explorador cambia para especificar que los mensajes de estado no deben desplazar el contenido del árbol de archivos y deben mostrarse como overlay flotante.

## Impact

- **HTML**: [app/index.html](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html) — Eliminar `style="display: none;"` del `#files-status`.
- **CSS**: [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css) — Rediseño completo de `.files-status` como overlay flotante con glassmorphism y animación.
- **TypeScript**: [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts) — Ajuste de `setExplorerStatus` para manejar visibilidad con clases CSS y auto-dismiss con `setTimeout`.
