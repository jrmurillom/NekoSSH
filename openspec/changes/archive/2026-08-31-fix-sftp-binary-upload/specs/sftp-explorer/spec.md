## MODIFIED Requirements

### Requirement: Descarga y subida de archivo individual por sesión
El sistema SHALL exponer operaciones SFTP de descarga y subida (replace) de un archivo individual asociadas al `terminal_id` de la Session SSH activa, reutilizando el canal SFTP multiplexado (sin segundo login). Las transferencias SHALL NOT degradar de forma permanente el PTY no bloqueante. La subida de archivos SHALL ejecutar un vaciado de buffers (`flush`) obligatorio con reintentos no bloqueantes y bombeo continuo del PTY (`pump_pty`), y SHALL verificar mediante `sftp.stat()` que el tamaño en bytes del archivo remoto destino coincida de forma exacta con el archivo local antes de reportar éxito. Si existe discrepancia de tamaño o fallo de I/O, el sistema SHALL reportar un error de integridad.

#### Scenario: Descargar archivo a ruta local
- **WHEN** el frontend solicita descargar un path remoto válido a una ruta local de sesión de edición
- **THEN** el backend escribe el contenido remoto en esa ruta local y reporta éxito

#### Scenario: Subir replace remoto
- **WHEN** el usuario confirmó la subida y el backend recibe upload del archivo local hacia el path remoto de origen
- **THEN** el sistema reemplaza el archivo remoto con el contenido local, vacía los buffers en el socket SSH, verifica la concordancia de tamaño en bytes mediante `sftp.stat()` y reporta éxito

#### Scenario: Subida de archivo binario completa e íntegra
- **WHEN** el usuario sube un archivo binario o comprimido (ej. `.zip`)
- **THEN** el sistema escribe la totalidad de los bytes, vacía y cierra el handle en el servidor garantizando que la cabecera y el índice final queden íntegros y legibles

#### Scenario: Fallo de integridad por discrepancia de tamaño
- **WHEN** el tamaño reportado por `sftp.stat()` del archivo remoto tras la subida difiere del tamaño local
- **THEN** el sistema reporta un error de integridad y no marca la subida como exitosa

#### Scenario: Fallo de transfer sin tumbar PTY
- **WHEN** una descarga o subida falla (permiso, I/O, path inexistente, discrepancia de tamaño)
- **THEN** el sistema informa el error y la sesión PTY permanece usable
