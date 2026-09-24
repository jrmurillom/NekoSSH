## MODIFIED Requirements

### Requirement: Descarga y subida de archivo individual por sesión
El sistema SHALL exponer operaciones SFTP de descarga y subida (replace) de un archivo individual asociadas al `terminal_id` de la Session SSH activa, reutilizando el canal SFTP multiplexado (sin segundo login). Las transferencias SHALL NOT degradar de forma permanente el PTY no bloqueante. La subida de archivos SHALL ejecutarse en streaming desde disco con un búfer fijo de 64 KiB en memoria constante $O(1)$ (sin cargar el archivo completo en RAM), escribiendo inicialmente sobre un archivo temporal oculto (`.<nombre>.nekossh.part`) en el directorio destino, aplicando un límite determinista de reintentos en estados `WouldBlock` durante la escritura para evitar bucles infinitos, ejecutando un vaciado de buffers (`flush`) obligatorio con bombeo continuo del PTY (`pump_pty`), verificando mediante `sftp.stat()` que el tamaño en bytes del archivo temporal coincida de forma exacta con el archivo local, y renombrando atómicamente al destino final. Si existe discrepancia de tamaño, timeout o fallo de I/O, el sistema SHALL eliminar el archivo `.nekossh.part` residual y reportar un error sin corromper el archivo destino previo. Adicionalmente, cuando se proporcione un canal `tauri::ipc::Channel`, el backend SHALL emitir eventos de progreso con *throttling* de `100ms` (incluyendo porcentaje, bytes transferidos, bytes totales y velocidad).

#### Scenario: Descargar archivo a ruta local
- **WHEN** el frontend solicita descargar un path remoto válido a una ruta local de sesión de edición
- **THEN** el backend escribe el contenido remoto en esa ruta local y reporta éxito

#### Scenario: Subir replace remoto
- **WHEN** el usuario confirmó la subida y el backend recibe upload del archivo local hacia el path remoto de origen
- **THEN** el sistema transmite el archivo en bloques de 64 KiB hacia el archivo temporal `.nekossh.part`, vacía los buffers en el socket SSH, verifica la concordancia de tamaño en bytes mediante `sftp.stat()`, renombra al archivo destino final y reporta éxito

#### Scenario: Subida de archivo binario completa e íntegra
- **WHEN** el usuario sube un archivo binario o comprimido (ej. `.zip` o imagen `.iso`)
- **THEN** el sistema escribe la totalidad de los bytes manteniendo un consumo de memoria constante de 64 KiB, vacía y cierra el handle en el servidor garantizando que la cabecera y el índice final queden íntegros y legibles

#### Scenario: Fallo de integridad por discrepancia de tamaño
- **WHEN** ocurre un error de escritura, timeout de `WouldBlock` o el tamaño reportado por `sftp.stat()` del archivo remoto tras la subida difiere del tamaño local esperado
- **THEN** el sistema elimina el archivo `.nekossh.part` incompleto en el servidor, preserva intacto cualquier archivo destino preexistente y reporta el error

#### Scenario: Emisión de progreso de subida por canal IPC
- **WHEN** se invoca `sftp_upload_file` con un canal de progreso activo
- **THEN** el backend emite el estado inicial (`0%`), actualizaciones periódicas con *throttling* de `100ms` y el estado final (`100%`) con bytes enviados, bytes totales y velocidad en bytes por segundo

#### Scenario: Fallo de transfer sin tumbar PTY
- **WHEN** una descarga o subida falla (permiso, I/O, path inexistente, discrepancia de tamaño o timeout)
- **THEN** el sistema informa el error y la sesión PTY permanece usable

### Requirement: Progreso y resultado de la subida por arrastre
Durante una subida iniciada por arrastrar y soltar, el sistema SHALL mostrar el avance en tiempo real al usuario dentro del overlay flotante de estado del explorador (`#files-status`), sin desplazar el árbol de archivos (`#files-tree`). El indicador visual SHALL presentar el nombre del archivo actual, el contador de lote `(i/N)`, una barra de progreso visual estilizada con los tokens del tema activo, el porcentaje numérico de transferencia (`0%` a `100%`), los bytes transferidos sobre el total (`MB / MB`) y la velocidad actual (`KB/s` o `MB/s`). Mientras una transferencia esté en curso, el sistema SHALL bloquear el inicio de nuevas subidas o copias SCP concurrentes. Al finalizar, el sistema SHALL informar el resultado. Si una subida individual falla, el sistema SHALL continuar con los archivos restantes y SHALL informar al final cuáles fallaron, manteniendo la sesión PTY utilizable.

#### Scenario: Progreso visible durante la subida
- **WHEN** el sistema está subiendo uno o varios archivos confirmados por el usuario
- **THEN** el overlay de estado del explorador muestra la barra de progreso rellenándose dinámicamente, el porcentaje exacto, el índice del archivo `(i/N)`, el volumen transferido y la velocidad sin mover el árbol de archivos

#### Scenario: Subida de archivo de cero bytes
- **WHEN** el usuario sube un archivo vacío (`0 bytes`)
- **THEN** el sistema calcula el progreso como `100%` sin producir errores de división por cero y completa la creación del archivo remoto

#### Scenario: Bloqueo de concurrencia durante transferencia activa
- **WHEN** hay una subida en curso y el usuario intenta iniciar otra subida o un "Pegar scp"
- **THEN** el sistema impide la ejecución simultánea hasta que concluya la transferencia activa

#### Scenario: Fallo parcial del lote
- **WHEN** uno de los archivos del lote falla al subir y los demás son correctos
- **THEN** el sistema termina el resto del lote, informa cuáles archivos fallaron y la sesión SSH permanece usable

#### Scenario: Refresco al completar
- **WHEN** el lote de subida termina con al menos un archivo subido
- **THEN** el sistema refresca el listado del explorador en la ruta destino

### Requirement: Portapapeles inter-sesión Copiar SCP y Pegar SCP
El sistema SHALL permitir al usuario copiar un archivo desde el explorador SFTP de una sesión activa (origen) y pegarlo en el explorador SFTP de otra sesión activa (destino) mediante las opciones de menú contextual denominadas exactamente "copiar scp" y "pegar scp".
- Al elegir "copiar scp" sobre un archivo, el sistema SHALL almacenar en el portapapeles de la aplicación el ID de la terminal origen y la ruta remota absoluta.
- La opción "pegar scp" en el menú contextual del árbol o fondo del explorador SHALL habilitarse únicamente si hay un elemento almacenado en el portapapeles, existe una sesión activa como destino y no hay otra transferencia en curso.
- Al activar "pegar scp", el sistema SHALL mostrar un diálogo A1 solicitando confirmación con el nombre del archivo y las rutas absolutas de origen y destino.
- Si el usuario confirma, el sistema SHALL consultar el tamaño del archivo origen mediante `sftp.stat()`, transferir los datos en streaming de 64 KiB hacia un archivo temporal `.nekossh.part` en el destino con límite de reintentos en `WouldBlock`, emitir el progreso en tiempo real hacia el mismo loader visual del explorador (barra de progreso, porcentaje, bytes transferidos y velocidad), validar el tamaño final, renombrar al nombre definitivo y refrescar la ubicación destino.

#### Scenario: Copiar SCP de un archivo
- **WHEN** el usuario selecciona "copiar scp" en el menú contextual de un archivo
- **THEN** el sistema registra el archivo y el ID de terminal origen en memoria y la opción "pegar scp" queda disponible para usarse

#### Scenario: Pegar SCP con confirmación
- **WHEN** el usuario hace clic derecho en el explorador destino, selecciona "pegar scp" y confirma en el diálogo A1
- **THEN** el sistema inicia el streaming directo de datos en bloques de 64 KiB mostrando en el overlay del explorador la barra de progreso, porcentaje (`%`), bytes transferidos y velocidad, valida e instala atómicamente el archivo en el servidor destino, y refresca la ubicación del explorador al completar con éxito

#### Scenario: Fallo durante Pegar SCP limpia archivo temporal
- **WHEN** ocurre un corte de red o fallo de escritura a mitad de una operación "pegar scp"
- **THEN** el sistema elimina el archivo `.nekossh.part` en el servidor destino, informa el error en el explorador y mantiene ambas sesiones PTY utilizables

#### Scenario: Cancelar Pegar SCP
- **WHEN** el usuario selecciona "pegar scp" y presiona cancelar o escape en el diálogo de confirmación
- **THEN** la transferencia se aborta y no se crea ni modifica ningún archivo en el destino
