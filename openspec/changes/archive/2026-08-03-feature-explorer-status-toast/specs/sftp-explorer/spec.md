## MODIFIED Requirements

### Requirement: Explorador de árbol remoto en el sidebar
El sistema SHALL exponer en el panel Archivos del sidebar un árbol navegable de directorios y archivos del servidor remoto, cargado bajo demanda (expansión lazy). El árbol SHALL permitir expandir/colapsar, abrir carpeta como ubicación actual, y subir al padre. Los mensajes de estado del explorador (cargando, error, confirmación de acciones) SHALL mostrarse como un overlay flotante superpuesto en la parte inferior del panel de archivos, sin desplazar ni alterar la posición del árbol de archivos bajo ninguna circunstancia. Los mensajes informativos SHALL desaparecer automáticamente tras unos segundos; los mensajes de error SHALL persistir hasta la próxima acción del usuario.

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
- **WHEN** el usuario activa "Subir" y la ruta actual no es la raíz `/`
- **THEN** el explorador navega al directorio padre

#### Scenario: Mensajes de estado sin desplazamiento del árbol
- **WHEN** el explorador muestra un mensaje de estado (cargando, error, confirmación)
- **THEN** el mensaje aparece como overlay flotante en la parte inferior del panel sin mover ni desplazar el contenido del árbol de archivos

#### Scenario: Auto-dismiss de mensajes informativos
- **WHEN** el explorador muestra un mensaje informativo (no error)
- **THEN** el mensaje desaparece automáticamente tras unos segundos con una transición suave
