## Why

Se detectaron fallos de Usabilidad (UX) e Interacción en el árbol de conexiones, la gestión de sesiones vivas de la terminal y los eventos de ratón en la interfaz. El árbol de conexiones iniciaba expandido, las categorías padres mostraban una retención visual de selección al hacer clic, las terminales vivas se cerraban sin confirmar, el clic derecho mostraba menús nativos del navegador en zonas no deseadas, y la selección de claves SSH (.ppk, .pem, .key) requería escribir la ruta manualmente.

## What Changes

- **Estado del Árbol de Conexiones**: El árbol de servidores/conexiones iniciará completamente colapsado por defecto al abrir la aplicación.
- **Fondo de Categorías Padres**: Se remueve el tinte de fondo de contexto activo (`.is-active-context`) en las filas de las carpetas para evitar la retención visual de color rosa al hacer clic.
- **Confirmación al Cerrar Conexiones Vivas**: Se solicita confirmación mediante un diálogo antes de cerrar cualquier pestaña de terminal con conexión SSH activa (`isConnected === true`), así como al invocar "Cerrar Todo" si existen sesiones vivas.
- **Bloqueo de Menú Contextual Nativo del Navegador**: Se inhabilita el clic derecho nativo del motor de renderizado (browser context menu) en áreas neutras de la aplicación.
- **Explorador Nativo del SO para Claves Privadas**: Se añade un botón "Examinar..." con selector de archivos del SO en el modal de perfil de conexión para seleccionar claves SSH (`.ppk`, `.pem`, `.key`, `id_rsa`, etc.) y auto-completar su ruta absoluta.

## Capabilities

### New Capabilities
- Ninguna.

### Modified Capabilities
- `connection-folders`: Ajuste de estado colapsado inicial y estilo de fila padre plano sin retención de fondo.
- `connection-profiles`: Integración de explorador nativo de archivos del sistema operativo para claves SSH.
- `ssh-terminal`: Incorporación de confirmación previa antes de cerrar sesiones SSH activas (individual o masivo).
- `ui-overlays`: Inhabilitación del menú contextual nativo del navegador en todo el documento.

## Impact

- **Archivos afectados**:
  - `app/src/main.ts` (lógica de estado de árbol, handlers de cierre de sesión, selector de archivos y prevención global de `contextmenu`).
  - `app/src/styles.css` (estilos de fila de carpeta y botón de explorador de archivos).
  - `app/index.html` (botón "Examinar..." en modal de conexión).
- **Backend Rust (Tauri)**: Uso del plugin/diálogo de archivos nativo o handler de selección de archivos.
