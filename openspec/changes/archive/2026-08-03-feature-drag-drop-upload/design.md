## Context

Para facilitar la subida de archivos, añadimos soporte para arrastrar archivos locales del sistema de archivos host y soltarlos en el panel lateral de archivos. El Webview de Tauri v2 intercepta drag-drop a nivel nativo y provee coordenadas y paths absolutos del host.

## Goals / Non-Goals

**Goals:**
- Implementar listeners de eventos Tauri nativos (`tauri://drag-enter`, `tauri://drag-over`, `tauri://drag-leave`, `tauri://drag-drop`) en el frontend.
- Renderizar un overlay cyberpunk (`explorer-dropzone`) con bordes neón que difumine el panel lateral cuando el arrastre de archivos esté activo.
- Calcular si las coordenadas del puntero durante el drag-over coinciden con una carpeta en el árbol DOM del explorador y resaltar dicha carpeta como target, actualizando el path destino en tiempo real.
- Mostrar una confirmación al usuario antes de subir.
- Subir los archivos utilizando la API `sftp_upload_file` existente y refrescar el explorador de archivos remotos.

**Non-Goals:**
- Implementar drag and drop de archivos remotos hacia el host local (solo soporta entrada host-to-remote).
- Modificar el sistema de archivos del host local.

## Decisions

### Detección de Destino Basada en Coordenadas
- **Decisión**: Usar `document.elementFromPoint(x, y)` en el handler de `tauri://drag-over` y `tauri://drag-drop`.
- **Lógica**: Buscaremos el nodo de carpeta DOM más cercano con la clase `.files-node-row` y dataset `data-is-dir="true"` para determinar si los archivos se deben depositar en esa subcarpeta. Si no hay carpeta válida debajo del cursor, el destino será la ruta de trabajo actual del explorador (`explorerCwd`).
- **Implementación**:
  ```typescript
  const elem = document.elementFromPoint(pos.x, pos.y);
  const row = elem?.closest(".files-node-row");
  const targetPath = (row && row.getAttribute("data-is-dir") === "true")
    ? row.getAttribute("data-path")
    : explorerCwd;
  ```

### Overlay y Animaciones Cyberpunk
- **Decisión**: Insertar un elemento `#explorer-dropzone` posicionado de manera absoluta dentro de `#panel-files` con `pointer-events: none` para no interferir con la captura de coordenadas de elementos del árbol DOM subyacentes.

## Risks / Trade-offs

- **Riesgo**: Que las coordenadas de `tauri://drag-over` no coincidan perfectamente con la escala del zoom de pantalla en Windows.
- **Mitigación**: `document.elementFromPoint` utiliza coordenadas del cliente webview que coinciden nativamente con el sistema de coordenadas lógicas entregado por los eventos de Tauri.
