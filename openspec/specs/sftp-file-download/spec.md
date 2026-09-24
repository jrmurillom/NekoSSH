# sftp-file-download Specification

## Purpose
TBD - created by archiving change sftp-file-download-streaming. Update Purpose after archive.
## Requirements
### Requirement: Acción de descarga en el menú contextual de archivos SFTP
El sistema SHALL presentar la opción `"Descargar"` en el menú contextual al hacer clic derecho sobre cualquier nodo de archivo (`!node.isDir`) en el árbol del explorador SFTP, salvo cuando ya exista una transferencia activa (`transferInProgress === true`), manteniendo intacta la opción `"Editar"` (`start_external_edit`) y su comportamiento.

#### Scenario: Menú contextual sobre archivo remoto sin transferencia en curso
- **WHEN** el usuario hace clic derecho sobre un archivo remoto en el árbol SFTP mientras `transferInProgress` es `false`
- **THEN** el menú contextual muestra las opciones `"Editar"`, `"Descargar"` y `"Copiar scp"`

#### Scenario: Menú contextual sobre archivo remoto durante transferencia activa
- **WHEN** el usuario hace clic derecho sobre un archivo remoto mientras otra transferencia (`upload`, `scp` o `download`) está en progreso (`transferInProgress === true`)
- **THEN** el sistema omite o bloquea la ejecución de `"Descargar"` para evitar colisiones de canal SFTP y solapamiento en `#files-status`

### Requirement: Selección de ruta destino mediante diálogo nativo del sistema operativo
El sistema SHALL permitir al usuario elegir la carpeta y el nombre de archivo local mediante un diálogo nativo `"Guardar como…"` del sistema operativo pre-rellenado con el nombre base del archivo remoto antes de iniciar la transferencia SFTP.

#### Scenario: Usuario selecciona ruta destino válida
- **WHEN** el usuario selecciona `"Descargar"` sobre el archivo remoto `/var/backups/prod-db.sql.gz` y confirma la ruta local en el diálogo nativo del sistema operativo
- **THEN** el comando `sftp_pick_download_path` devuelve la ruta absoluta local elegida e inicia inmediatamente la descarga en streaming

#### Scenario: Usuario cancela el diálogo de selección de ruta
- **WHEN** el usuario selecciona `"Descargar"` y cierra o cancela el diálogo nativo `"Guardar como…"`
- **THEN** `sftp_pick_download_path` devuelve `null`, el sistema no abre ningún canal SFTP ni altera el estado de `transferInProgress`, y no muestra error en `#files-status`

### Requirement: Descarga SFTP en streaming O(1) sin límite de tamaño y con staging atómico local
El backend Rust SHALL descargar el archivo remoto en bloques fijos de `64 KiB` (`[0u8; 65536]`) en memoria constante $O(1)$ sin imponer el límite de `10 MiB` de edición externa, escribiendo inicialmente en un archivo temporal oculto `.<nombre>.nekossh.part` en el mismo directorio local elegido, verificando que los bytes descargados coincidan con el tamaño reportado por `sftp.stat()`, y renombrándolo atómicamente al nombre definitivo al concluir.

#### Scenario: Descarga exitosa de archivo grande (>4 GiB) o vacío (0 bytes)
- **WHEN** se ejecuta `sftp_download_file_with_progress` para un archivo remoto (desde `0 bytes` hasta múltiples gigabytes)
- **THEN** el sistema escribe los bloques en `.<nombre>.nekossh.part`, bombea el PTY (`pump_pty`), verifica `bytes_written == total_bytes`, renombra atómicamente `.<nombre>.nekossh.part` a la ruta destino final y emite `100%`

#### Scenario: Fallo de red o disco durante la descarga limpia el archivo parcial local
- **WHEN** ocurre una desconexión SSH, timeout de `WouldBlock` o error de escritura en disco local antes de completar el 100% de la descarga
- **THEN** el sistema cierra y elimina inmediatamente el archivo temporal local `.<nombre>.nekossh.part` (`std::fs::remove_file`) garantizando que no queden archivos truncados en la ruta elegida por el usuario, y muestra un mensaje de error persistente (`❌ ...`) en `#files-status`

### Requirement: Integración con la barra de progreso en tiempo real en `#files-status`
El sistema SHALL conectar la descarga al motor de progreso unificado (`TransferProgressPayload` con `operation: "download"` y `createExplorerStatusController`), mostrando el encabezado `"Descargando <archivo>"`, el porcentaje (`0%..100%`), la barra de progreso con acento plano (`var(--color-accent-primary)`) y las métricas de tamaño/velocidad actualizadas cada `100ms`.

#### Scenario: Visualización de progreso y cierre exitoso de descarga
- **WHEN** la descarga está en curso y posteriormente finaliza con éxito
- **THEN** `#files-status` muestra `"Descargando <archivo>"` con barra de progreso y métricas en tiempo real durante la transferencia, transiciona a `"✅ Descarga completa: <archivo>"` al finalizar y se auto-oculta tras `3000ms`

