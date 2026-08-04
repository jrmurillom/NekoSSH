## 1. Setup: Create Feature Branch (MANDATORY)

- [x] 1.1 Crear y cambiar a la rama `feature/scp-copy-paste` antes de modificar el código de la app.
- [x] 1.2 Verificar que la rama activa en Git sea `feature/scp-copy-paste`.

## 2. Backend: Comando de Streaming en Rust

- [x] 2.1 En [app/src-tauri/src/external_edit.rs](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/external_edit.rs), implementar la función `sftp_copy_between_sessions` que abre SFTP sobre origen y destino, lee el archivo origen en bloques de 64 KiB y los escribe directamente en el destino.
- [x] 2.2 En [app/src-tauri/src/lib.rs](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/lib.rs), importar y registrar el nuevo comando `sftp_copy_between_sessions` en el builder de Tauri.

## 3. Frontend: Portapapeles y Menú Contextual (TypeScript)

- [x] 3.1 En [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), declarar una variable global `scpClipboard` para almacenar el archivo origen copiado.
- [x] 3.2 En la misma clase, en los manejadores de menú contextual del explorador de archivos, agregar la opción "copiar scp" cuando se hace clic derecho sobre un nodo archivo.
- [x] 3.3 Agregar la opción "pegar scp" en el menú contextual del fondo del explorador y en las filas de las carpetas, mostrándose deshabilitada o visible únicamente cuando `scpClipboard` no es nulo y la sesión actual es diferente a la de origen.
- [x] 3.4 Implementar el comportamiento de "copiar scp", guardando el path y terminalId origen.
- [x] 3.5 Implementar la acción "pegar scp", desplegando el diálogo de confirmación `confirmDialog` con las rutas origen/destino. Al confirmar, invocar el comando `sftp_copy_between_sessions`, mostrar estado de carga y refrescar el árbol de archivos destino al finalizar.

## 4. Pruebas y Verificación

- [x] 4.1 Ejecutar `cargo test` para garantizar estabilidad de las pruebas unitarias.
- [x] 4.2 Compilar el frontend con `npm run build`.

## 5. Corrección de Ruta (Fix): Conexión Explorer On-Focus

- [x] 5.1 Extender la interfaz `ActiveTerminal` en [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts) con propiedades `explorerCwd` y `explorerRoot`.
- [x] 5.2 Modificar `switchActiveTerminal` para hacer backup del estado actual de `explorerCwd` y `explorerRoot` al portapapeles de la terminal saliente y restaurar el estado desde la terminal activa entrante.
- [x] 5.3 Asegurar que solo se realice la consulta SFTP (`refreshExplorerForActiveTerminal`) si el panel de archivos (`#panel-files`) está visible/activo.
- [x] 5.4 Ejecutar `npm run build` para validar.
