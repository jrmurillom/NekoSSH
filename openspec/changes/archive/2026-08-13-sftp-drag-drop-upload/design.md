## Context

El panel Archivos (`#panel-files`) ya tiene el overlay `#explorer-dropzone` en HTML y sus estilos en CSS, y el árbol se pinta en `renderExplorerTree` / `buildExplorerNodeEl` (`app/src/main.ts`). El backend expone `sftp_upload_file(terminal_id, local_path, remote_path)` en `app/src-tauri/src/external_edit.rs`, ya usado por la edición externa. Falta por completo la capa de arrastre en el frontend.

Restricciones del proyecto: diálogos con el A1 existente (`confirmDialog` de `overlays.ts`), y estética derivada de los tokens del tema conceptual activo.

## Goals / Non-Goals

**Goals:**

- Arrastrar uno o varios archivos locales y soltarlos sobre el panel Archivos para subirlos por SFTP.
- Destino claro y visible durante el arrastre; carpeta bajo el cursor resaltada.
- Overlay 100 % basado en tokens del tema (transparencia incluida), sin colores fijos.
- Confirmación siempre antes de subir, y confirmación aparte al sobreescribir.
- Progreso visible y resumen final, tolerante a fallos parciales.

**Non-Goals:**

- Arrastrar carpetas locales o subida recursiva.
- Arrastrar del explorador remoto hacia el escritorio local.
- Transferencia entre sesiones (cubierta por copiar/pegar SCP).
- Cola de transferencias persistente o reanudable.

## Decisions

### 1. Eventos nativos de Tauri, no HTML5

- **Elección:** usar el evento de arrastre del webview de Tauri v2 (`onDragDropEvent` de `@tauri-apps/api/webview`, equivalente a `tauri://drag-enter|drag-over|drag-leave|drag-drop`).
- **Por qué:** el webview de Tauri intercepta el arrastre de archivos del sistema operativo, por lo que los eventos HTML5 (`dragover`/`drop`) no reciben rutas de archivo reales. El evento nativo entrega las rutas locales absolutas, que es justo lo que `sftp_upload_file` necesita.
- **Alternativa descartada:** `dragDropEnabled: false` + HTML5, que daría objetos `File` del navegador sin ruta en disco.

### 2. Resolución del destino por coordenadas

- **Elección:** en cada evento de arrastre, usar `document.elementFromPoint(x, y)` para localizar la fila del árbol bajo el cursor y leer su ruta desde `dataset`.
- **Requisito derivado:** `buildExplorerNodeEl` debe escribir `row.dataset.path` y `row.dataset.isDir` en cada fila.
- **Regla de destino:** fila de carpeta bajo el cursor → esa ruta; fila de archivo → la carpeta que lo contiene; fuera de filas → `explorerCwd`.
- **Nota:** el evento nativo entrega coordenadas físicas; deben convertirse con `devicePixelRatio` antes de `elementFromPoint` para que el zoom de pantalla en Windows no desalinee el objetivo.

### 3. Overlay derivado de tokens

- **Elección:** sustituir el `background: rgba(26, 21, 35, 0.85)` fijo por una composición basada en tokens (p. ej. `color-mix` sobre el color de superficie/acento del tema con la opacidad deseada), manteniendo `backdrop-filter`.
- **Por qué:** el overlay debe cambiar con el tema conceptual como el resto del panel; hoy quedaría desalineado en temas claros o de otro tono.
- **Icono:** alinear el `📥` a un icono Lucide (`upload`) para respetar el spec `ui-icons`.

### 4. Confirmaciones con el A1 existente

- **Elección:** todas las confirmaciones usan `confirmDialog` de `overlays.ts`; ninguna ventana nativa ni componente nuevo.
- **Subida:** título de confirmación con nombre del archivo (o cantidad) y ruta destino.
- **Sobreescritura:** diálogo aparte, por archivo en colisión, con acción de reemplazar o cancelar ese archivo.

### 5. Detección de colisiones sin tocar el backend

- **Elección:** antes de subir, listar el directorio destino con `sftp_list_dir` (ya existente) y comparar nombres para detectar cuáles ya existen.
- **Por qué:** evita añadir un command Rust nuevo y reutiliza el canal SFTP de la sesión.
- **Trade-off:** una llamada extra por operación de soltado; despreciable frente a la transferencia.

### 6. Subida secuencial con estado en el overlay de status

- **Elección:** subir archivo por archivo con `sftp_upload_file`, reportando avance en el overlay de estado del explorador (`setExplorerStatus`), y refrescar el listado al final.
- **Por qué:** secuencial evita saturar el canal SFTP compartido con el PTY; el overlay de status ya existe y no desplaza el árbol.
- **Fallos:** un error no aborta el lote; se acumulan y se resumen al terminar.

### 7. Ligado al shell padre del contexto

- **Elección:** las subidas usan el `terminal_id` del shell padre, igual que el resto del explorador.
- **Por qué:** consistencia con el requisito existente "Explorador ligado al padre del contexto".

## Risks / Trade-offs

- [Coordenadas desalineadas con zoom/DPI en Windows] → Normalizar por `devicePixelRatio` y validar en la verificación de UI.
- [Arrastre sobre otras pestañas del sidebar] → Ignorar el evento salvo que el panel Archivos esté activo y haya sesión conectada.
- [Archivos muy grandes bloqueando la sesión] → Subida secuencial y estado visible; sin límite nuevo en esta iteración.
- [Colisión detectada por listado puede quedar desactualizada] → Ventana de carrera aceptada; el reemplazo es la semántica de `sftp_upload_file`.
- [Overlay con `pointer-events: none`] → Debe permanecer así para no interferir con `elementFromPoint`.

## Migration Plan

- Cambio aditivo de frontend; sin migración de datos ni de esquema.
- Rollback: retirar los listeners de arrastre; el overlay queda oculto como hoy.

## Open Questions

- Ninguna bloqueante.
