## Why

Actualmente la funcionalidad para configurar una imagen de fondo personalizada en NekoSSH falla o no se renderiza. Existen 3 causas principales:
1. **Conflicto de apilamiento Z-Index y capas CSS**: La capa base de fondo oscuro (`.bg-overlay-layer` con color `#0c060d` opaco) posee `z-index: -1`, mientras que la capa de imagen (`.bg-image-layer`) usaba `z-index: -2` (o `-1`), quedando totalmente tapada e invisible bajo el color negro base de la app.
2. **Falta de resolución de protocolos de archivos locales**: La ruta local escrita o seleccionada no utilizaba la función `convertFileSrc` de Tauri (`@tauri-apps/api/core`), provocando que el navegador (WebView2) bloquee las rutas locales de disco (`file://` o paths de Windows) por políticas de seguridad.
3. **Falta de controles de exploración y limpieza (CRUD)**: El formulario no cuenta con un selector de archivos nativo del SO ("Examinar...") para elegir imágenes de fondo ni con un botón explícito de eliminación/limpieza ("Quitar") para restaurar el fondo predeterminado.

## What Changes

- **Resolución de capas CSS**: Invertir la jerarquía z-index para que `.bg-image-layer` (`z-index: -1`) se posicione por encima de la capa base `.bg-overlay-layer` (`z-index: -2`), permitiendo que la imagen se fusione transparentemente con la opacidad configurada.
- **Conversión de rutas locales con `convertFileSrc`**: En `applyBackgroundSettings()`, procesar las rutas mediante `convertFileSrc()` de `@tauri-apps/api/core` para transformar rutas del disco local en URLs válidas del protocolo de activos de Tauri (`asset://`).
- **Controles de UI y CRUD para el Fondo**:
  - Añadir botón "Examinar..." con selector nativo de imágenes (`input type="file" accept="image/*"`) en el popover de preferencias para elegir la imagen directamente del explorador de archivos.
  - Añadir botón "Quitar" para borrar la imagen de fondo de `localStorage` y restaurar la vista sin fondo.
  - Mantener la sincronización en tiempo real del control deslizable de opacidad.

## Capabilities

### New Capabilities
- Ninguna.

### Modified Capabilities
- `app-branding`: Gestión completa (CRUD) y renderizado funcional del fondo personalizado sobre la capa base Cyber-Sakura.

## Impact

- **Archivos afectados**:
  - `app/src/main.ts` (import de `convertFileSrc`, lógica de `applyBackgroundSettings`, handlers para `btn-browse-bg`, `file-input-bg` y `btn-clear-bg`).
  - `app/src/styles.css` (corrección de `z-index` en `.bg-image-layer` y `.bg-overlay-layer`, alineación del formulario de fondo).
  - `app/index.html` (adición de botones "Examinar..." y "Quitar" en `#prefs-popover`).
