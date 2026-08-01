## ADDED Requirements

### Requirement: Canal SFTP dedicado por sesión de terminal
El sistema SHALL mantener, por cada sesión de terminal SSH activa, una **segunda conexión SSH dedicada a SFTP** (separada de la Session del PTY), autenticada con el mismo perfil, viva mientras dure la terminal. El PTY SHALL NOT compartir Session con operaciones SFTP ni alterar `set_blocking` de la Session del PTY por causa del explorador.

#### Scenario: Abrir SFTP al conectar
- **WHEN** el usuario establece una sesión de terminal exitosa con un perfil
- **THEN** el backend abre (o deja lista) la conexión SFTP asociada a ese `terminal_id` sin degradar el PTY

#### Scenario: Cerrar SFTP al desconectar
- **WHEN** el usuario cierra la terminal o se invoca desconexión
- **THEN** el sistema cierra y libera la conexión SFTP correspondiente junto con el PTY

#### Scenario: Fallo al abrir SFTP (p. ej. MaxSessions)
- **WHEN** la segunda conexión SFTP no puede autenticarse o abrirse
- **THEN** el explorador informa el error y la sesión PTY permanece usable

### Requirement: Explorador de árbol remoto en el sidebar
El sistema SHALL exponer en el panel Archivos del sidebar un árbol navegable de directorios y archivos del servidor remoto, cargado bajo demanda (expansión lazy), usando la conexión SFTP dedicada. El árbol SHALL permitir expandir/colapsar, abrir carpeta como ubicación actual, y subir al padre.

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
