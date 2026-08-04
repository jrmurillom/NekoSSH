## 1. Setup: Create Feature Branch (MANDATORY)

- [x] 1.1 Crear y cambiar a la rama `feature/drag-drop-upload` antes de modificar el código de la app.
- [x] 1.2 Verificar que la rama activa en Git sea `feature/drag-drop-upload`.

## 2. Cambios en la Estructura HTML y Estilos CSS

- [x] 2.1 En [app/index.html](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html), agregar el div de overlay `#explorer-dropzone` dentro de `#panel-files`.
- [x] 2.2 En [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css), definir la clase `.explorer-dropzone` y sus elementos hijos, aplicando diseño glassmorphism y bordes dashed cyberpunk con animación de pulso.
- [x] 2.3 En el mismo archivo CSS, definir la clase `.files-node-row.drag-over` para iluminar las carpetas que sirvan como destino del arrastre.

## 3. Atributos Dataset en el Árbol del Explorador

- [x] 3.1 En [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), en la función `buildExplorerNodeEl`, establecer `row.dataset.path = node.path` y `row.dataset.isDir = String(node.isDir)` sobre cada elemento del árbol para que sea identificable por el parser de coordenadas.

## 4. Implementación de Drag & Drop y Subida (TypeScript)

- [x] 4.1 En [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), inicializar listeners para los eventos de ventana nativos de Tauri: `tauri://drag-enter`, `tauri://drag-over`, `tauri://drag-leave` y `tauri://drag-drop`.
- [x] 4.2 En el listener de `tauri://drag-over`, verificar si el panel de archivos está activo. Si lo está, mostrar el overlay, resolver la carpeta destino mediante `document.elementFromPoint`, resaltar la fila hover y actualizar el texto del destino en el overlay.
- [x] 4.3 En el listener de `tauri://drag-leave`, ocultar el overlay y eliminar los resaltados de carpeta.
- [x] 4.4 En el listener de `tauri://drag-drop`, ocultar el overlay, recuperar la ruta remota destino final, mostrar el diálogo de confirmación A1 (`confirmDialog`), e invocar en bucle `sftp_upload_file` para subir los archivos confirmados.
- [x] 4.5 Al finalizar el bucle de subida con éxito, invocar `refreshExplorerAtCurrentPath()` para refrescar el árbol y mostrar los nuevos archivos.

## 5. Pruebas y Verificación

- [x] 5.1 Ejecutar `cargo test` para asegurar estabilidad del compilador y backend.
- [x] 5.2 Compilar el frontend con `npm run build` para asegurar compatibilidad de tipos TypeScript.
