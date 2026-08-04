## Why

Actualmente, no existe un mecanismo para transferir archivos directamente entre diferentes sesiones SSH activas en la aplicación. Para copiar un archivo de un servidor a otro, el usuario debe descargarlo localmente de manera manual y luego subirlo al destino. Agregar la acción de "copiar scp" y "pegar scp" en el explorador de archivos permite copiar archivos en streaming directo entre servidores a través de la memoria local de forma rápida y segura.

## What Changes

- Opción de menú contextual "copiar scp" en archivos y carpetas del explorador SFTP.
- Opción de menú contextual "pegar scp" en el explorador de archivos cuando existe un archivo o carpeta en el portapapeles de NekoSSH.
- Transferencia por streaming en memoria (en chunks de 64 KiB) entre las dos conexiones SSH/SFTP mediante el backend.
- Flujo de confirmación A1 antes de iniciar la transferencia de datos.

## Capabilities

### New Capabilities

### Modified Capabilities
- `sftp-explorer`: Agregar los requerimientos de interacción de portapapeles remoto "copiar scp" y "pegar scp", el flujo de confirmación y el streaming en memoria del backend.

## Impact

- Afecta a `app/src/main.ts` (menús contextuales de archivos y lógica de pegado/invocación) y `app/src-tauri/src/external_edit.rs` o `lib.rs` (añadir comando Tauri `sftp_copy_between_sessions`).
