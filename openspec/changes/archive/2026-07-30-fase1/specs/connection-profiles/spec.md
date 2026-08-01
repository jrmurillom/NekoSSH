## ADDED Requirements

### Requirement: Almacenamiento y Lectura de Perfiles de Servidores
El sistema SHALL permitir a los usuarios crear, leer, actualizar y eliminar (CRUD) perfiles de servidores SSH que se almacenarán localmente en la base de datos SQLite. Cada perfil debe contener un nombre identificativo, host (IP o dominio), puerto (por defecto 22), nombre de usuario y un intervalo opcional de keepalive.

#### Scenario: Crear un perfil de servidor exitosamente
- **WHEN** el usuario ingresa el nombre, host, puerto 22, usuario y guarda el formulario
- **THEN** el sistema registra el perfil en la base de datos SQLite y actualiza la lista visual de servidores en el panel izquierdo

### Requirement: Autenticación por Contraseña o Llave Privada
El sistema SHALL permitir configurar para cada perfil una credencial de autenticación que puede ser una contraseña cifrada o una llave privada (con soporte opcional de frase de paso).

#### Scenario: Configurar autenticación con archivo de llave privada
- **WHEN** el usuario selecciona la opción de llave privada en el perfil, introduce la ruta de la llave `.pem` y una frase de paso, y guarda
- **THEN** el sistema almacena la ruta de la llave y la frase de paso cifrada en la base de datos asociada a ese perfil

### Requirement: Redirección de Puertos (Túneles SSH)
El sistema SHALL permitir configurar túneles SSH de tipo Local o Dinámico (SOCKS Proxy) asociados a un perfil de conexión.

#### Scenario: Añadir un túnel dinámico SOCKS a un perfil
- **WHEN** el usuario agrega un túnel de tipo 'dynamic' en el puerto local 8080 y guarda el perfil
- **THEN** el sistema almacena la especificación del túnel en la base de datos asociada al perfil del servidor
