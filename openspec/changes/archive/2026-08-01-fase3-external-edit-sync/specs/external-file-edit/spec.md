## ADDED Requirements

### Requirement: Sesión de edición externa con archivo temporal
El sistema SHALL, al iniciar una edición externa, descargar el archivo remoto a un directorio temporal aislado de la aplicación (por `edit_id`), registrar la asociación `terminal_id` + `remote_path` + `local_path`, y abrir el archivo local con el editor configurado. Si ya existe una sesión activa para el mismo `terminal_id` y `remote_path`, el sistema SHALL reutilizarla (reabrir el editor) en lugar de duplicar el temporal.

#### Scenario: Primera edición de un archivo
- **WHEN** el usuario inicia edición sobre un archivo remoto que aún no tiene sesión activa
- **THEN** el sistema crea temp aislado, descarga el remoto, abre el editor y comienza a vigilar el archivo local

#### Scenario: Reabrir el mismo remoto
- **WHEN** el usuario vuelve a pedir Editar/doble clic sobre el mismo path remoto en la misma terminal con sesión de edición ya activa
- **THEN** el sistema no crea un segundo temp y reabre el editor sobre el local existente

### Requirement: Editor externo preferido con fallback OS
El sistema SHALL persistir una preferencia de usuario `preferred_external_editor` (ruta de ejecutable). Si la preferencia es no vacía y el ejecutable es usable, el sistema MUST abrir el archivo local con ese editor. Si la preferencia está vacía o no es usable, el sistema MUST abrir el archivo con la asociación por defecto del sistema operativo. El producto SHALL exponer UI en español para ver y cambiar esa ruta (patrón Settings / appearance).

#### Scenario: Abrir con editor preferido
- **WHEN** existe una ruta de editor preferido válida y el usuario inicia una edición
- **THEN** el sistema lanza ese ejecutable con el path del archivo temporal

#### Scenario: Fallback a asociación del OS
- **WHEN** no hay editor preferido configurado (o la ruta no es usable) y el usuario inicia una edición
- **THEN** el sistema abre el archivo temporal con la asociación por defecto del OS

#### Scenario: Guardar preferencia de editor
- **WHEN** el usuario guarda una nueva ruta de editor en preferencias
- **THEN** el valor queda persistido y se usa en las siguientes aperturas

### Requirement: Vigilancia y confirmación A1 antes de subir
El sistema SHALL vigilar el archivo temporal de cada sesión de edición y, tras detectar un cambio real de contenido (con debounce y sin apilar diálogos), mostrar un dialog glass A1 preguntando si se debe subir al servidor. El producto MUST NOT subir en silencio. Escape o Cancelar MUST dejar el remoto intacto y MAY dejar la vigilancia activa para futuros cambios. El dialog de subida MUST mostrar por defecto solo el **filename** (basename del path remoto), con un control colapsable “ver ruta completa” que revela el path remoto completo en un textarea readonly (wrap / seleccionable) sin desbordar el panel.

#### Scenario: Cambio detectado pide confirmación
- **WHEN** el archivo temporal cambia de forma real tras el debounce y no hay confirm pendiente para esa sesión
- **THEN** el sistema muestra el dialog A1 con la pregunta de subir al servidor y el filename remoto (sin forzar el path completo inline)

#### Scenario: Ver ruta completa en confirm de subida
- **WHEN** el dialog A1 de subida está abierto y el usuario activa “ver ruta completa”
- **THEN** el sistema muestra el path remoto completo en un textarea readonly seleccionable sin overflow horizontal del dialog

#### Scenario: Confirmar subida
- **WHEN** el usuario confirma en el dialog A1
- **THEN** el sistema sube/reemplaza el archivo remoto de origen y actualiza la baseline de vigilancia

#### Scenario: Cancelar subida
- **WHEN** el dialog A1 de subida está abierto y el usuario cancela o pulsa Escape
- **THEN** no se modifica el archivo remoto y la sesión de edición puede seguir vigilando

#### Scenario: Sin auto-upload
- **WHEN** el archivo temporal cambia
- **THEN** el sistema no sube al servidor hasta que el usuario confirma en el dialog A1

### Requirement: Limpieza de temporales de edición
El sistema SHALL detener el watcher y eliminar best-effort el directorio temporal de una sesión de edición cuando la sesión se cierra de forma ordenada (cierre de terminal/sesión SSH asociada, o cierre explícito de la edición). Al arrancar la aplicación, el sistema SHALL barrer temporales huérfanos según la política de TTL acordada en el design. El sistema MUST NOT borrar el temp mientras un upload o un dialog de subida de esa sesión está en curso.

#### Scenario: Cleanup al cerrar terminal
- **WHEN** el usuario cierra la terminal (o se libera la Session) con sesiones de edición asociadas
- **THEN** el sistema deja de vigilar y elimina best-effort los temps de esas sesiones tras resolver diálogos abiertos sin subir

#### Scenario: No borrar durante upload
- **WHEN** hay un upload en curso para una sesión de edición
- **THEN** el sistema no elimina el archivo temporal de esa sesión hasta terminar el intento

### Requirement: Desconexión durante edición
Si la sesión SSH se desconecta o cierra mientras hay ediciones activas, el sistema SHALL detener watchers, no completar una subida pendiente, cerrar cualquier dialog A1 de subida sin aplicar cambios remotos, e informar al usuario. El archivo local temporal SHOULD conservarse de forma temporal para no perder trabajo del usuario; el reattach automático de la sesión de edición tras reconnect manual NO es obligatorio en este change.

#### Scenario: Disconnect con confirm abierto
- **WHEN** se desconecta la sesión SSH con un dialog “¿Subir al servidor?” abierto
- **THEN** el dialog se cierra sin subir y el usuario recibe aviso de que no se pudo subir

#### Scenario: Trabajo local tras disconnect
- **WHEN** ocurre desconexión mid-edit
- **THEN** el sistema deja de intentar subir automáticamente y conserva el temp local según la política de cleanup (no borrado inmediato silencioso del trabajo del usuario)

### Requirement: Política de archivos grandes y binarios
El sistema SHALL rechazar el inicio de edición externa cuando el tamaño del archivo remoto supera el límite por defecto de **10 MiB**, con mensaje claro en español. Ante archivos que parezcan binarios (heurística), el sistema SHALL pedir confirmación A1 antes de descargar/abrir; si el usuario cancela, no inicia la sesión de edición.

#### Scenario: Archivo demasiado grande
- **WHEN** el usuario intenta editar un archivo remoto mayor a 10 MiB
- **THEN** el sistema no descarga ni abre el editor y muestra un mensaje de rechazo amable

#### Scenario: Posible binario con confirm
- **WHEN** el archivo parece binario según la heurística y el usuario inicia edición
- **THEN** el sistema muestra un dialog A1 de aviso y solo continúa si el usuario confirma

#### Scenario: Texto o config habitual
- **WHEN** el usuario edita un archivo de texto/config dentro del límite de tamaño
- **THEN** el sistema inicia la sesión de edición sin el rechazo de tamaño

### Requirement: Verificación sin mutar hosts SSH de prueba compartidos
La verificación automatizada y las corridas de agente de este capability MUST NOT escribir, sobrescribir ni borrar archivos en hosts SSH de prueba compartidos fuera de un sandbox remoto desechable explícitamente provisionado y documentado por el usuario. Los tests SHALL usar mocks, fixtures locales o fake SFTP in-process para cubrir download/upload y el ciclo de confirmación. El producto SHALL seguir subiendo/reemplazando el path remoto de origen cuando el usuario final confirma en uso real (esta restricción no elimina el upload de producto).

#### Scenario: Tests sin writes al lab host
- **WHEN** se ejecutan unit/integration tests del ciclo de edición externa
- **THEN** no se realizan upload/replace/delete contra el host SSH de pruebas compartido; la evidencia usa mock o fixture local

#### Scenario: Verificación de agente sin mutar el lab
- **WHEN** un agente ejecuta desktop-commands o desktop-ui verification sin sandbox remoto disposable documentado
- **THEN** no escribe en paths remotos del lab; valida con mocks/fixtures locales y/o solo lectura (download a temp local) y documenta N/A para writes remotos

#### Scenario: Upload de producto tras confirm del usuario
- **WHEN** un usuario final confirma “¿Subir al servidor?” en uso real de la app
- **THEN** el sistema sube/reemplaza el archivo remoto de origen según el flujo FileZilla (la restricción de lab no aplica a ese uso)
