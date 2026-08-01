## MODIFIED Requirements

### Requirement: Almacenamiento y Lectura de Perfiles de Servidores
El sistema SHALL permitir a los usuarios crear, leer, actualizar y eliminar (CRUD) perfiles/conexiones SSH que se almacenarán localmente en la base de datos SQLite. Cada conexión MUST pertenecer a una carpeta (`folder_id`). Cada conexión debe contener un nombre identificativo, host (IP o dominio), puerto (por defecto 22), nombre de usuario y un intervalo opcional de keepalive. El listado en el sidebar SHALL presentarse de forma jerárquica (carpeta → conexiones).

#### Scenario: Crear un perfil de servidor exitosamente
- **WHEN** el usuario ingresa el nombre, host, puerto 22, usuario, selecciona o está en contexto de una carpeta destino, y guarda el formulario
- **THEN** el sistema registra la conexión en la base de datos SQLite asociada a esa carpeta y actualiza el árbol visual en el panel izquierdo

#### Scenario: Crear conexión requiere carpeta
- **WHEN** el usuario intenta crear una conexión sin carpeta destino válida
- **THEN** el sistema rechaza la operación o exige elegir/crear carpeta antes de persistir

#### Scenario: Migración de conexiones planas
- **WHEN** existen conexiones previas sin carpeta tras actualizar el esquema
- **THEN** el sistema las asigna a una carpeta por defecto creada en la migración

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

### Requirement: Tarjeta de conexión compacta en el árbol
En el sidebar, cada conexión SHALL mostrarse como tarjeta compacta: nombre + acciones editar/eliminar; línea `user@host:port` con acento cyan (sin label de tipo auth/túnel). La tarjeta SHALL abrir la sesión SSH con **doble clic**. Un control de copiar junto a la línea de endpoint SHALL copiar `user@host` al portapapeles sin abrir sesión.

#### Scenario: Sin label de método de auth
- **WHEN** el usuario ve una conexión en el árbol
- **THEN** no aparece el texto `SSH (Contraseña)` ni equivalente de llave/túnel en la tarjeta

#### Scenario: Copiar user@host
- **WHEN** el usuario activa el icono de copiar en la línea de endpoint
- **THEN** el sistema copia `username@host` al clipboard y no inicia una sesión SSH

#### Scenario: Conectar con doble clic
- **WHEN** el usuario hace doble clic en la tarjeta de conexión (fuera de botones de acción)
- **THEN** el sistema inicia la sesión SSH hacia ese perfil
