## Context

NekoSSH requiere resolver 5 puntos de interacción de usuario (UX) para mejorar la consistencia visual y la seguridad operativa durante las sesiones de conexión.

## Goals / Non-Goals

**Goals:**
- Asegurar que el árbol de conexiones permanezca colapsado al iniciar la aplicación.
- Garantizar que las carpetas del árbol de conexiones no conserven ninguna tonalidad ni tintero de fondo activo al hacer clic.
- Exigir confirmación explícita mediante un cuadro de diálogo antes de cerrar cualquier terminal activa (individual o masivo).
- Bloquear la aparición del menú contextual nativo del navegador en todo el documento.
- Proveer un selector nativo de archivos del sistema operativo para ubicar la ruta de llaves privadas SSH (.ppk, .pem, .key, etc.) e insertarla directamente en el formulario de perfil.

**Non-Goals:**
- Modificar el backend en Rust de persistencia de SQLite para perfiles ni la estructura de datos.
- Alterar la lógica interna de emulación de terminal de xterm.js.

## Decisions

### 1. Inicialización Colapsada del Árbol de Conexiones
- **Decisión:** En `app/src/main.ts`, en la función `loadProfiles()`, omitir la inicialización automática que agregaba todas las carpetas a `expandedFolderIds`.
- **Justificación:** `expandedFolderIds` permanecerá como un `Set<number>()` vacío por defecto al iniciar la app. Las carpetas solo se expandirán cuando el usuario haga clic explícito en cada una.

### 2. Estilos de Fila de Carpeta Padre Planos (Sin Fondo de Selección)
- **Decisión:** En `app/src/styles.css`, modificar la regla `.connection-tree .folder-row.is-active-context` para establecer `background: transparent;`.
- **Justificación:** La fila mantiene su comportamiento funcional de asignación de `activeFolderId` sin proyectar un tintero rosa translúcido sobre la categoría. El estado `:hover` se mantiene únicamente mientras el puntero permanezca sobre la fila.

### 3. Diálogo de Confirmación para Cierre de Conexiones Vivas
- **Decisión:** 
  - En `closeTerminalSession(terminalId: string, skipConfirm = false)`: Si `activeTerm.isConnected === true` y `skipConfirm` es `false`, llamar a `confirmDialog({ title: "¿Cerrar sesión activa?", message: "La conexión SSH sigue activa. ¿Deseas desconectarte y cerrar?", confirmLabel: "Desconectar", danger: true })`. Si el usuario cancela, se detiene el cierre.
  - En `closeAllTerminals()`: Verificar si existe al menos una terminal activa (`activeTerminals`). Si alguna tiene `isConnected === true`, mostrar un único `confirmDialog` masivo antes de llamar a `closeTerminalSession(id, true)`.

### 4. Bloqueo del Menú Contextual Nativo del Navegador
- **Decisión:** Agregar un listener global en `app/src/main.ts`: `document.addEventListener("contextmenu", (ev) => ev.preventDefault());`.
- **Justificación:** Dado que todos los elementos con menús contextuales personalizados (carpetas, perfiles, explorador de archivos, terminal) manejan el evento en sus nodos específicos con `ev.stopPropagation()`, la regla a nivel de `document` actúa como salvaguarda impidiendo que el motor WebView2 despliegue el menú del browser en zonas neutras.

### 5. Buscador Nativo del Sistema Operativo para Llaves Privadas
- **Decisión:** En `app/index.html`, agregar un botón de acción `<button type="button" id="btn-browse-key" class="btn-secondary">Examinar...</button>` dentro de `#auth-key-group`. En `app/src/main.ts`, adjuntar un handler que dispare un `<input type="file">` oculto en la Webview (o llamada a la API nativa de Tauri si está disponible) para capturar la ruta absoluta del archivo `.pem`, `.key`, `.ppk` o `id_rsa` seleccionado y escribirla en `#prof-key-path`.

## Risks / Trade-offs

- **[Riesgo]** Un usuario podría querer cerrar rápidamente múltiples pestañas desconectadas sin diálogos.
  - *Mitigación:* La confirmación solo se activa si `isConnected === true`. Las pestañas desconectadas o cerradas por el servidor se cierran con 1 solo clic de forma inmediata.
