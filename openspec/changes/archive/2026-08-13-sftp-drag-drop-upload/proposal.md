## Why

El explorador SFTP tiene el cascarón visual del dropzone (`#explorer-dropzone` en HTML y CSS) y el spec `sftp-explorer` ya describe el comportamiento, pero **la lógica de arrastrar y soltar nunca llegó al código**: no existe ningún listener de eventos de arrastre ni invocación de subida desde el frontend. Subir archivos hoy obliga a salir del flujo del explorador.

## What Changes

- Implementar el arrastre y soltado de archivos locales sobre el panel Archivos para subirlos por SFTP.
- Overlay de destino durante el arrastre, **con transparencia y colores derivados de los tokens del tema activo** (sin colores hardcodeados), reemplazando el `rgba()` fijo actual.
- Resolución de destino: carpeta del árbol bajo el cursor, o la ruta actual del explorador si no se apunta a ninguna.
- Resaltado visual de la carpeta destino mientras se arrastra.
- **Confirmación obligatoria antes de subir**, con nombre de archivo (o cantidad) y ruta destino, usando el diálogo A1 existente (`confirmDialog`).
- **Confirmación de sobreescritura** por archivo cuando ya existe en el destino.
- Progreso durante la subida y resumen al terminar, incluyendo archivos fallidos sin abortar el resto.
- Refresco del explorador al completar.
- **Fuera de alcance:** arrastrar carpetas locales completas, subida recursiva, arrastre de remoto a local, y arrastre entre sesiones (ya cubierto por copiar/pegar SCP).

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `sftp-explorer`: refinar el requisito de arrastrar y soltar (overlay basado en tokens del tema, confirmación siempre) y añadir requisitos de confirmación de sobreescritura y de progreso/resultado de la subida.

## Impact

- Frontend: `app/src/main.ts` (listeners de arrastre, resolución de destino, flujo de subida, `dataset` en filas del árbol), `app/src/styles.css` (tokens del overlay), `app/index.html` (icono del overlay si se alinea a Lucide).
- Reutiliza `confirmDialog` de `app/src/overlays.ts` y el command Rust `sftp_upload_file` ya existente en `app/src-tauri/src/external_edit.rs`.
- Backend Rust: sin cambios de API previstos, salvo consulta de existencia remota para la confirmación de sobreescritura.
- Specs: delta en `openspec/specs/sftp-explorer`.
