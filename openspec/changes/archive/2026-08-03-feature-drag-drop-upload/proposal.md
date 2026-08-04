## Why

Actualmente, para subir archivos locales al servidor remoto, el usuario debe realizar la edición externa e iniciar un ciclo de guardado, o bien no dispone de un mecanismo directo para simplemente arrastrar archivos desde su sistema operativo. Añadir soporte para arrastrar y soltar (Drag & Drop) directamente en el panel del explorador de archivos remotos mejora significativamente la experiencia del usuario (UX) permitiendo subidas rápidas y directas.

## What Changes

- Soporte para Drag & Drop de archivos desde el sistema host local al explorador de archivos remotos (SFTP).
- Visualización de un overlay cyberpunk intermitente (`explorer-dropzone`) con bordes neón que indique el área donde soltar y la ruta destino actual.
- Resaltado de nodos de carpetas específicas en el árbol del explorador cuando se arrastren archivos directamente sobre ellas, actualizando dinámicamente la ruta destino mostrada en el overlay.
- Flujo de confirmación mediante un diálogo A1 para cada archivo (o lote de archivos) mostrando el nombre del archivo y la ruta destino en el servidor.
- Invocación al comando Tauri `sftp_upload_file` para procesar la subida y refrescar automáticamente el árbol.

## Capabilities

### New Capabilities

### Modified Capabilities
- `sftp-explorer`: Añadir el requerimiento de Drag & Drop para subir archivos individuales o múltiples, indicando el destino y solicitando confirmación al usuario de forma transparente.

## Impact

- Afecta a `app/src/main.ts` (manejadores de drag and drop Tauri, render de dropzone y árbol) e `app/index.html` (añadir contenedor del overlay visual) y `app/src/styles.css` (estilos estéticos cyberpunk del overlay y los estados de hover).
