# connection-profiles

## Purpose

Gestión, almacenamiento y lectura de perfiles/conexiones SSH (credenciales y túneles) en SQLite local, organizadas por carpeta.

## Requirements

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

### Requirement: Cajita de conexión en el árbol
En el sidebar, cada conexión SHALL mostrarse como **cajita** con chrome de tarjeta sutil (fondo/borde/radius bajo `.connection-tree .profile-item`): nombre + línea `user@host:port` con acento cyan (sin label de tipo auth/túnel). La cajita NO MUST mostrar botones inline de editar (lápiz) ni eliminar (basurero). Esas acciones MUST estar en el menú contextual. Un control de copiar junto a la línea de endpoint SHALL copiar `user@host` al portapapeles sin abrir sesión. La cajita SHALL abrir la sesión SSH con **doble clic** (fuera del control de copiar y del menú). Renombrar el nombre visible MUST ser inline iniciado desde el menú contextual, no por doble clic. “Editar” en el menú SHALL abrir el formulario/modal de perfil existente. Las carpetas padre MUST permanecer como filas planas de lista (**sin** chrome de caja/tarjeta ni borde contenedor visible); la jerarquía usa indentación + guía vertical. El chrome de caja MUST aplicarse solo a las conexiones hijas, no a la fila de carpeta.

#### Scenario: Sin label de método de auth
- **WHEN** el usuario ve una conexión en el árbol
- **THEN** no aparece el texto `SSH (Contraseña)` ni equivalente de llave/túnel en la cajita

#### Scenario: Sin lápiz ni basurero en la cajita
- **WHEN** el usuario ve una conexión en el árbol
- **THEN** no hay controles inline de editar ni eliminar en la cajita; sí puede haber el icono de copiar endpoint

#### Scenario: Copiar user@host
- **WHEN** el usuario activa el icono de copiar en la línea de endpoint
- **THEN** el sistema copia `username@host` al clipboard y no inicia una sesión SSH

#### Scenario: Conectar con doble clic
- **WHEN** el usuario hace doble clic en la cajita de conexión (fuera del control de copiar)
- **THEN** el sistema inicia la sesión SSH hacia ese perfil

#### Scenario: Acciones desde menú contextual
- **WHEN** el usuario abre el menú contextual de una conexión
- **THEN** puede elegir al menos editar (modal), renombrar (inline) y eliminar (con confirmación glass)

#### Scenario: Renombrar conexión desde menú
- **WHEN** el usuario elige renombrar en el menú contextual de la conexión
- **THEN** el nombre de la cajita entra en modo input inline (Enter guarda, Escape cancela)

#### Scenario: Cajita solo en hijos (carpeta sin caja)
- **WHEN** el usuario ve el árbol en estado de reposo
- **THEN** cada conexión se percibe como cajita (fondo/borde/radius de tarjeta) y cada carpeta padre se percibe como fila plana sin ese chrome de caja

### Requirement: Crear conexión desde el header de zona
El sistema SHALL exponer la acción de crear una nueva conexión como icon-button en el header de zona del panel Servidores (label **Conexiones**), además del `+` en la fila de carpeta. Activar ese icono MUST abrir el mismo flujo de formulario/modal de perfil que el control previo “Nueva conexión”, resolviendo la carpeta destino con las reglas existentes (contexto de carpeta activa / requisitos de `folder_id`). El producto NO MUST depender de un CTA de texto primario “Nueva conexión” en toolbar split para este flujo.

#### Scenario: Abrir flujo desde icono del header
- **WHEN** el usuario activa el icono de crear conexión en el header de zona Conexiones
- **THEN** se abre el flujo de creación de perfil (modal/formulario) con carpeta destino según las reglas existentes

#### Scenario: Plus por carpeta intacto
- **WHEN** el usuario activa el `+` en la fila de una carpeta
- **THEN** se abre el flujo de nueva conexión asociado a esa carpeta, como antes
