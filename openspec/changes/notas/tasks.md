## 1. Control de Versiones

- [x] 1.1 Crear e ir a la rama de Git `feature/add-notes-tab`.

## 2. Backend & Base de Datos (Rust / SQLite)

- [x] 2.1 Agregar la creación de la tabla `notes` (`id`, `title`, `content`, `updated_at`) en las migraciones de SQLite de Tauri en backend.
- [x] 2.2 Implementar los comandos Tauri IPC para la gestión de notas: `get_notes`, `create_note`, `update_note`, `delete_note`.
- [x] 2.3 Registrar y exponer los nuevos comandos en [`app/src-tauri/src/lib.rs`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/lib.rs).

## 3. Frontend Layout & Estilos (HTML / CSS)

- [x] 3.1 Agregar el botón del cuarto tab "Notas" en la cabecera del menú lateral en [`app/index.html`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html).
- [x] 3.2 Crear la pestaña y contenedor lateral `#notes-tab-content` con buscador e icono `(+)` en [`app/index.html`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html).
- [x] 3.3 Crear la estructura HTML del modal flotante `#note-editor-modal` con título editable inline, textarea de edición, e icono de papelera en [`app/index.html`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html).
- [x] 3.4 Añadir las clases de diseño Cyber-Sakura para el listado de notas, el título inline y el editor de texto plano en [`app/src/styles.css`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css).

## 4. Frontend Logic (TypeScript)

- [x] 4.1 Registrar e importar el icono de notas de Lucide en [`app/src/icons.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/icons.ts).
- [x] 4.2 Crear el módulo [`app/src/modules/notes-helper.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/modules/notes-helper.ts) e implementar la función `initNotesTab` para inicializar el contenedor, cargar eventos de botones e iconos de la pestaña de Notas.
- [x] 4.3 Programar en `notes-helper.ts` la lógica de refresco y renderizado de la lista de notas en el panel lateral, incluyendo el filtro de búsqueda.
- [x] 4.4 Programar el botón `(+)` para crear una nueva nota llamando a Tauri `create_note` y recargar la lista de notas.
- [x] 4.5 Implementar en `notes-helper.ts` la lógica del modal flotante: abrir la nota seleccionada, habilitar edición inline del título al hacer click y auto-guardado en base de datos en los eventos de cierre (`close`) o cambio de foco (`blur`) del editor.
- [x] 4.6 Conectar el botón de eliminar nota en el modal flotante con confirmación de borrado.
- [x] 4.7 Importar e invocar `initNotesTab()` en la inicialización principal en [`app/src/main.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts).

## 5. Compilación y Validación

- [x] 5.1 Compilar el frontend (`npm run build` en `app`).
- [x] 5.2 Compilar el backend de Tauri (`cargo check` en `app/src-tauri`).
- [ ] 5.3 Validar el funcionamiento del CRUD completo y la persistencia de datos tras cerrar y reabrir la aplicación.
