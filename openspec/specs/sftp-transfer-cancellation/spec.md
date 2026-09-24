# sftp-transfer-cancellation Specification

## Purpose
TBD - created by archiving change sftp-transfer-cancellation. Update Purpose after archive.
## Requirements
### Requirement: Cancelación manual con diálogo de confirmación en `#files-status.is-progress`
El sistema SHALL renderizar un botón de cancelación `[ ✕ ]` (`.files-status-progress-cancel`) en el encabezado de `#files-status.is-progress` durante cualquier transferencia activa (`upload`, `download` o `scp`). Al pulsarlo, el sistema SHALL abrir un diálogo de confirmación (`confirmDialog`) sin detener preventivamente el flujo de streaming hasta que el usuario confirme explícitamente.

#### Scenario: Usuario confirma la cancelación de una transferencia (Descarga, Subida o SCP)
- **WHEN** el usuario hace clic en el botón `[ ✕ ]` de `#files-status.is-progress` y confirma `"Sí, cancelar"` en el diálogo de confirmación
- **THEN** el frontend invoca `sftp_cancel_transfer`, el backend aborta en el siguiente bloque de `64 KiB`, elimina el archivo temporal `.nekossh.part` (local o remoto), detiene cualquier archivo pendiente del lote en caso de subida múltiple, y `#files-status` muestra `"Transferencia cancelada"`

#### Scenario: Usuario rechaza la cancelación en el diálogo de confirmación
- **WHEN** el usuario hace clic en `[ ✕ ]` pero selecciona `"Seguir transfiriendo"` (o cierra el diálogo)
- **THEN** la transferencia continúa su ejecución normal en segundo plano hasta completarse al 100% sin pérdida de datos

#### Scenario: La transferencia concluye al 100% mientras el diálogo de confirmación sigue abierto
- **WHEN** el diálogo `"Cancelar transferencia"` está abierto en pantalla y la transferencia subyacente alcanza el `100%` antes de que el usuario confirme
- **THEN** el comando `sftp_cancel_transfer` actúa como no-op seguro (idempotente) sin eliminar el archivo ya finalizado ni sobreescribir el estado de éxito

### Requirement: Cancelación automática al cerrar la sesión SSH (`close_ssh_session`)
El sistema SHALL cancelar automáticamente cualquier transferencia SFTP (`download`, `upload` o `scp`) vinculada a una sesión `terminal_id` en el instante en que dicha sesión se cierre (`close_ssh_session` o `close_all_ssh_connections`).

#### Scenario: Cierre de pestaña de terminal durante una descarga o subida activa
- **WHEN** el usuario cierra la pestaña de la terminal `terminal_id` mientras hay una transferencia activa en esa sesión
- **THEN** `close_ssh_session` activa inmediatamente el token de cancelación para `terminal_id`, el hilo SFTP aborta el bucle de `64 KiB`, elimina el archivo `.nekossh.part` (local o remoto), suelta el `Arc<Mutex<LiveSsh>>` cerrando el socket TCP y el frontend silencia cualquier evento residual del `Channel`

#### Scenario: Cierre de sesión origen o destino durante una copia SCP inter-sesión
- **WHEN** el usuario cierra cualquiera de las dos terminales (`source_terminal_id` o `target_terminal_id`) durante una copia SCP
- **THEN** el sistema cancela inmediatamente la transferencia SCP y limpia el archivo `.nekossh.part` en el servidor destino

### Requirement: Cancelación y barrido síncrono de `.nekossh.part` al cerrar la aplicación
El sistema SHALL cancelar todas las transferencias activas y eliminar síncronamente del disco local cualquier archivo temporal `.nekossh.part` en curso cuando la aplicación reciba el evento de salida (`RunEvent::ExitRequested` o `RunEvent::Exit`).

#### Scenario: Cierre de la aplicación NekoSSH a mitad de una descarga
- **WHEN** el usuario cierra la ventana de NekoSSH mientras se descarga un archivo hacia `.<nombre>.nekossh.part`
- **THEN** el manejador `RunEvent::ExitRequested` / `RunEvent::Exit` activa la cancelación global y ejecuta la eliminación síncrona del `local_part_path` registrado, garantizando que no queden archivos `.nekossh.part` huérfanos en el disco del usuario

