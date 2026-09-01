## MODIFIED Requirements

### Requirement: Sesión de edición externa con archivo temporal
El sistema SHALL, al solicitar iniciar o reabrir una edición externa sobre un archivo remoto, descargar obligatoriamente la versión más reciente del archivo desde el servidor a un directorio temporal aislado de la aplicación (por `edit_id`). Si ya existía una sesión o watcher previo para el mismo `terminal_id` y `remote_path`, el sistema SHALL detener el watcher previo, reemplazar la sesión con una nueva línea base (`baseline_fingerprint`), abrir el archivo local con el editor configurado y comenzar una vigilancia activa con un nuevo watcher sobre el archivo descargado.

#### Scenario: Primera edición de un archivo
- **WHEN** el usuario inicia edición sobre un archivo remoto que aún no tiene sesión activa
- **THEN** el sistema crea temp aislado, descarga el remoto, abre el editor y comienza a vigilar el archivo local

#### Scenario: Reabrir y forzar descarga fresca
- **WHEN** el usuario vuelve a pedir Editar o hace doble clic sobre un path remoto en la misma terminal que ya tenía una sesión previa
- **THEN** el sistema detiene el watcher anterior, descarga obligatoriamente la versión fresca desde el servidor remoto, abre el editor con el nuevo contenido y re-establece la vigilancia activa