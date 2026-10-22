## MODIFIED Requirements

### Requirement: Portapapeles inter-sesión Copiar SCP y Pegar SCP
El sistema SHALL permitir al usuario copiar un archivo desde el explorador SFTP de una sesión activa (origen) y pegarlo en el explorador SFTP de otra sesión activa (destino) mediante las opciones de menú contextual del explorador de archivos.
- Al elegir "copiar scp" sobre un archivo, el sistema SHALL almacenar en el portapapeles de la aplicación el ID de la terminal origen, el nombre del archivo y la ruta remota absoluta.
- La opción "pegar scp" en el menú contextual del árbol o fondo del explorador SHALL habilitarse únicamente si hay un elemento almacenado en el portapapeles, existe una sesión activa como destino y no hay otra transferencia en curso.
- Al activar "pegar scp", el sistema SHALL mostrar un diálogo A1 solicitando confirmación con el nombre del archivo y las rutas absolutas de origen y destino.
- Si el usuario confirma, el sistema SHALL orquestar una transferencia directa punto a punto (P2P) ejecutando en el servidor origen un proceso nativo de transferencia (`rsync` si está disponible en el PATH remoto, o `scp` como alternativa).
- Para la autenticación directa entre servidores, el sistema SHALL inyectar de forma segura en la sesión del servidor origen las credenciales del servidor destino conocidas por NekoSSH (clave privada en memoria segura temporal o en socket de sesión), con permisos restrictivos `0600`.
- El sistema SHALL parsear la salida continua (`stdout`) del comando de transferencia en el servidor origen para emitir telemetría de progreso en tiempo real al overlay flotante `#files-status` (porcentaje, velocidad en MB/s o KB/s y tiempo restante estimado ETA).
- Al finalizar, abortar o cancelarse la transferencia por el usuario, el sistema SHALL eliminar inmediatamente cualquier credencial o archivo temporal remanente en el servidor origen mediante guardias RAII.
- Si el servidor origen y el servidor destino se encuentran en redes físicamente incomunicadas y no existe ruta TCP entre ellos, el sistema SHALL recurrir automáticamente a un transporte por tubería streaming de sockets Unix (`cat` / `tar` pipe sobre SSH) sin fragmentación de bloques SFTP síncronos ni pausas intermedias.

#### Scenario: Copiar SCP de un archivo
- **WHEN** el usuario selecciona "copiar scp" en el menú contextual de un archivo
- **THEN** el sistema registra el archivo y el ID de terminal origen en memoria y la opción "pegar scp" queda disponible para usarse

#### Scenario: Pegar SCP con transferencia directa P2P exitosa
- **WHEN** el usuario hace clic derecho en el explorador destino, selecciona "pegar scp" y confirma en el diálogo A1
- **THEN** el sistema inicia la ejecución en el servidor origen de `rsync` o `scp` hacia el servidor destino utilizando las credenciales inyectadas, muestra en `#files-status` la velocidad real en MB/s y ETA de transferencia directa de datacenter, elimina las credenciales temporales al terminar y refresca la ubicación del explorador destino

#### Scenario: Fallo de red directa activa fallback a tubería streaming
- **WHEN** el servidor origen no tiene ruta de red hacia la IP/puerto del servidor destino (timeout o no route to host)
- **THEN** el sistema conmuta automáticamente a la tubería streaming de sockets directos a través del cliente, completando la transferencia a máxima velocidad de ancho de banda y notificando el modo de transporte utilizado

#### Scenario: Fallo durante transferencia limpia credenciales y temporales
- **WHEN** ocurre un error de proceso o corte de red a mitad de la operación
- **THEN** el sistema destruye de forma segura la clave temporal inyectada en el servidor origen, elimina cualquier archivo parcial en el destino, informa el error en el explorador y mantiene ambas sesiones PTY íntegras y utilizables

#### Scenario: Cancelar Pegar SCP
- **WHEN** el usuario selecciona "pegar scp" y presiona cancelar o escape en el diálogo de confirmación, o pulsa el botón de cancelar durante la transferencia
- **THEN** el comando remoto se interrumpe de inmediato mediante señal SIGTERM/SIGKILL, se purgan las credenciales temporales en el origen y no se altera el sistema de archivos destino
