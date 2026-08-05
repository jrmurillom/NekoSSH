## MODIFIED Requirements

### Requirement: Autenticación por Contraseña o Llave Privada
El sistema SHALL permitir configurar para cada perfil una credencial de autenticación que puede ser una contraseña cifrada o una llave privada (con soporte opcional de frase de paso). El material de la llave privada MUST persistirse en la columna `private_key` de `auth_credentials` como contenido de archivo (PEM/texto), no como ruta de sistema de archivos. El formulario de perfil MUST ofrecer un botón "Examinar..." que active el selector de archivos del sistema operativo; al seleccionar un archivo de clave (`.pem`, `.key`, `id_rsa`, etc.), el sistema MUST leer el contenido del archivo y asociarlo al perfil al guardar. El formulario MUST NOT mostrar el contenido PEM al usuario: MUST indicar únicamente que la autenticación por llave está configurada cuando exista material guardado, y permitir reemplazarlo vía "Examinar...".

#### Scenario: Configurar autenticación con archivo de llave privada
- **WHEN** el usuario selecciona la opción de llave privada, elige un archivo de llave con "Examinar...", opcionalmente introduce una frase de paso, y guarda
- **THEN** el sistema almacena el contenido de la llave en `private_key` y la frase de paso asociada a ese perfil

#### Scenario: Configurar autenticación con selector de archivos del SO
- **WHEN** el usuario hace clic en "Examinar..." en la sección de llave privada y selecciona un archivo
- **THEN** el sistema lee el contenido del archivo seleccionado y deja el perfil listo para guardarlo con ese material (sin depender de una ruta absoluta visible)

#### Scenario: Formulario oculta el PEM
- **WHEN** el usuario abre el formulario de un perfil con llave privada ya configurada
- **THEN** la UI muestra un indicador de que la llave está configurada y no revela el texto PEM

#### Scenario: Conservar llave al editar sin reemplazar
- **WHEN** el usuario edita y guarda un perfil con auth por llave sin seleccionar un archivo nuevo
- **THEN** el sistema conserva el `private_key` ya persistido
