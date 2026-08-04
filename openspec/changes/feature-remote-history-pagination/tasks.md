## 1. Setup: Crear rama de feature (OBLIGATORIO)

- [x] 1.1 Crear y cambiar a la rama `feature/remote-history-pagination` antes de modificar código de la app.
- [x] 1.2 Verificar que la rama activa en Git sea `feature/remote-history-pagination`.

## 2. Backend Rust: Canal silencioso de lectura de historial

- [x] 2.1 En `app/src-tauri/src/external_edit.rs` (o archivo backend equivalente), implementar el comando `sftp_read_remote_history_paged` que tome `terminal_id`, `offset` (líneas a omitir desde el final) y `limit` (100).
- [x] 2.2 Implementar heurística de detección de shell remoto y ruta del archivo de historial correspondiente (`.bash_history` o `.zsh_history`).
- [x] 2.3 Ejecutar silenciosamente el comando `tail -n <offset + limit> <archivo> | head -n <limit>` sobre la conexión SSH activa de dicho `terminal_id`.
- [x] 2.4 Retornar las líneas leídas en un vector estructurado hacia el frontend.
- [x] 2.5 Registrar e importar el comando en `app/src-tauri/src/lib.rs`.

## 3. Frontend HTML/CSS: Componentes visuales del modal

- [x] 3.1 En [app/index.html](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html), añadir la estructura del modal flotante `#history-modal` con su input de búsqueda fuzzy, la tabla con columnas de fecha y comando, y la botonera inferior de navegación ("Cargar más antiguos" y "Ver más recientes").
- [x] 3.2 En [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css), diseñar los estilos del modal con temática Cyberpunk-Sakura (glassmorphism con blur, bordes de neón sakura, scroll del historial y tablas alienadas).

## 4. Frontend TypeScript: Controlador y atajo

- [x] 4.1 En [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), registrar un event listener global para el atajo `Ctrl+Shift+H` (o `Ctrl+Alt+H`) sobre el contenedor de terminales activas.
- [x] 4.2 Desarrollar el parser de historial en TypeScript para procesar los timestamps de Zsh/Bash y transformarlos en fechas legibles de uso.
- [x] 4.3 Implementar fuzzy search local sobre los resultados cargados en la tabla del modal.
- [x] 4.4 Programar la lógica de paginación (offset acumulativo de 100 en 100) llamando al comando de Rust al interactuar con los botones de paginación.
- [x] 4.5 Renderizar el botón con icono de terminal (`AppIcons.terminal`) al final de cada fila de comando.
- [x] 4.6 Asociar la inyección a la terminal únicamente al clic del botón con icono de terminal o a la tecla `Enter`/`Shift+Enter` desde el teclado. Asegurar que hacer clic en la fila de comando únicamente la seleccione y no la envíe ("no baja con un click").



## 5. Verificación y Pruebas

- [x] 5.1 Ejecutar `cargo test` para asegurar la estabilidad del backend.
- [x] 5.2 Ejecutar `npm run build` para validar que TypeScript compila sin errores.

## 6. Corrección (Refactor): Reutilizar estilos de Snippets y Botón de Copiado

- [x] 6.1 En [app/index.html](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html), modificar `#history-modal` para reutilizar las clases y contenedores visuales de snippets como `.snippets-modal-content` y `.snippets-toolbar`.
- [x] 6.2 En [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css), remover la hoja de estilos duplicada `.history-modal-content` e integrar/reutilizar el diseño de snippets para el historial.
- [x] 6.3 En [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), cambiar la renderización del botón de inyección para usar el icono de copiado/pegar (`AppIcons.copy`) idéntico al de snippets.
- [x] 6.4 Ejecutar `npm run build` para validar que la refactorización compile sin errores.
