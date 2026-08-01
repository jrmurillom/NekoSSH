# sftp-explorer

## Purpose

Explorador de archivos remotos en el sidebar (pestaña Archivos): listado SFTP lazy, navegación de árbol y acciones hacia la terminal activa. SSOT de producto Fase 2.

## Requirements

### Requirement: Canal SFTP por sesión de terminal
El sistema SHALL exponer operaciones SFTP asociadas a cada `terminal_id` con sesión SSH activa, usando la **misma** Session TCP/SSH del PTY (canal subsystem SFTP), sin un segundo login que tumbe la sesión interactiva. Las operaciones SFTP SHALL NOT usar `set_blocking(true)` de forma que corrompa el PTY no bloqueante.

#### Scenario: Abrir SFTP con sesión activa
- **WHEN** el usuario tiene una sesión de terminal exitosa y solicita listar el explorador
- **THEN** el backend abre/usa el canal SFTP de esa Session sin degradar el PTY

#### Scenario: Cerrar SFTP al desconectar
- **WHEN** el usuario cierra la terminal o se invoca desconexión
- **THEN** el sistema libera la Session (PTY + SFTP) correspondiente

#### Scenario: Fallo al listar SFTP
- **WHEN** una operación SFTP no puede completarse
- **THEN** el explorador informa el error y la sesión PTY permanece usable cuando el fallo es solo del listado

### Requirement: Explorador de árbol remoto en el sidebar
El sistema SHALL exponer en el panel Archivos del sidebar un árbol navegable de directorios y archivos del servidor remoto, cargado bajo demanda (expansión lazy). El árbol SHALL permitir expandir/colapsar, abrir carpeta como ubicación actual, y subir al padre.

#### Scenario: Listar directorio raíz o home al activar Archivos
- **WHEN** hay una sesión SSH activa y el usuario abre la pestaña Archivos
- **THEN** el sistema muestra las entradas del directorio remoto inicial (home o `/` según disponibilidad) en el panel lateral

#### Scenario: Expandir una carpeta
- **WHEN** el usuario expande un nodo de carpeta en el árbol
- **THEN** el sistema solicita el listado SFTP de esa ruta (si aún no está cargado) y renderiza sus hijos sin perder otros nodos expandidos

#### Scenario: Colapsar una carpeta
- **WHEN** el usuario colapsa un nodo de carpeta expandido
- **THEN** se ocultan sus hijos y el resto del árbol permanece intacto

#### Scenario: Abrir carpeta como ubicación del explorador
- **WHEN** el usuario abre una carpeta (acción de navegación, no solo expand)
- **THEN** la barra de ruta muestra esa path y el árbol lista el contenido de esa path como nivel actual

#### Scenario: Subir al directorio padre
- **WHEN** el usuario activa “Subir” y la ruta actual no es la raíz `/`
- **THEN** el explorador navega al directorio padre

### Requirement: Abrir ruta en la terminal desde el explorador
El sistema SHALL permitir, mediante menú contextual (o acción equivalente) sobre un directorio del explorador, enviar un cambio de directorio (`cd`) a la terminal activa hacia esa ruta remota.

#### Scenario: Abrir en Terminal
- **WHEN** el usuario elige “Abrir en Terminal” sobre un directorio del árbol y existe una terminal activa ligada a esa sesión
- **THEN** el sistema ejecuta `cd` a esa ruta en el PTY y el explorador navega por SFTP a esa path

### Requirement: Barra de ruta editable del explorador
El sistema SHALL permitir al usuario editar manualmente la ruta remota mostrada en el panel Archivos y navegar a ella mediante un control de icono **Ir**. El refresco del listado actual SHALL exponerse como icono **Actualizar** (no solo como texto).

#### Scenario: Ir a ruta escrita
- **WHEN** hay sesión activa, el usuario escribe una ruta remota válida en el campo de ruta y activa el icono Ir
- **THEN** el explorador solicita el listado SFTP de esa ruta y muestra el árbol correspondiente

#### Scenario: Actualizar listado
- **WHEN** el usuario activa el icono Actualizar con una ruta de explorador ya mostrada
- **THEN** el sistema vuelve a listar esa ruta vía SFTP y refresca las entradas visibles

#### Scenario: Ruta inválida al Ir
- **WHEN** el usuario pide Ir a una ruta que SFTP no puede listar
- **THEN** el sistema informa el error sin tumbar la sesión SSH/PTY

### Requirement: Teclado PTY estable durante uso del explorador
El sistema SHALL permitir escribir en el PTY mientras el usuario lista o navega el explorador SFTP, sin que la sesión interactiva se cierre por operaciones de archivos.

#### Scenario: Listar mientras se escribe en la terminal
- **WHEN** hay sesión activa y el usuario escribe en el PTY mientras el explorador lista un directorio
- **THEN** la sesión PTY permanece abierta y la entrada de teclado sigue llegando al shell

### Requirement: Sin sync automático por cd tipado
El producto NO SHALL exigir que el explorador siga automáticamente el `cd` tipado en la terminal. La navegación del árbol es SFTP (Ir / expand / abrir / Actualizar / Abrir en Terminal).

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
