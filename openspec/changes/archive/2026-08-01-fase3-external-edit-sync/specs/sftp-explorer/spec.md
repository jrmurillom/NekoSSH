## ADDED Requirements

### Requirement: Inicio de edición externa desde el explorador
El sistema SHALL permitir iniciar una sesión de edición externa sobre un **archivo** remoto desde el explorador SFTP mediante doble clic en el archivo o mediante el ítem de menú contextual **“Editar”**. El doble clic sobre un directorio MUST conservar el comportamiento de navegación existente (no iniciar edición).

#### Scenario: Doble clic en archivo
- **WHEN** el usuario hace doble clic en un nodo archivo del árbol SFTP con sesión activa
- **THEN** el sistema inicia el flujo de edición externa para esa ruta remota (download a temp + abrir editor)

#### Scenario: Menú Editar
- **WHEN** el usuario elige “Editar” en el menú contextual de un archivo del explorador
- **THEN** el sistema inicia el mismo flujo de edición externa que el doble clic

#### Scenario: Doble clic en carpeta sin edición
- **WHEN** el usuario hace doble clic en un nodo carpeta
- **THEN** el explorador navega/abre la carpeta y NO inicia una sesión de edición externa

### Requirement: Descarga y subida de archivo individual por sesión
El sistema SHALL exponer operaciones SFTP de descarga y subida (replace) de un archivo individual asociadas al `terminal_id` de la Session SSH activa, reutilizando el canal SFTP multiplexado (sin segundo login). Las transferencias SHALL NOT degradar de forma permanente el PTY no bloqueante.

#### Scenario: Descargar archivo a ruta local
- **WHEN** el frontend solicita descargar un path remoto válido a una ruta local de sesión de edición
- **THEN** el backend escribe el contenido remoto en esa ruta local y reporta éxito

#### Scenario: Subir replace remoto
- **WHEN** el usuario confirmó la subida y el backend recibe upload del archivo local hacia el path remoto de origen
- **THEN** el sistema reemplaza el archivo remoto con el contenido local

#### Scenario: Fallo de transfer sin tumbar PTY
- **WHEN** una descarga o subida falla (permiso, I/O, path inexistente)
- **THEN** el sistema informa el error y la sesión PTY permanece usable

### Requirement: Verificación de transfer sin mutar el lab SSH compartido
Las pruebas automatizadas y la verificación por agente de download/upload MUST NOT mutar archivos en el host SSH de pruebas compartido. La evidencia de replace remoto en CI/agente SHALL obtenerse con mock o fake SFTP local, salvo sandbox remoto desechable explícitamente provisionado por el usuario. El command de upload de producto permanece disponible para uso real tras confirm del usuario.

#### Scenario: Harness de transfer local/mock
- **WHEN** se verifica `sftp_download_file` / `sftp_upload_file` en tests o harness de agente
- **THEN** no se escribe en paths arbitrarios del servidor de pruebas compartido; se usa mock, fixture local o sandbox disposable documentado
