### Requirement: Autenticación por Contraseña o Llave Privada
El sistema SHALL permitir configurar para cada perfil una credencial de autenticación que puede ser una contraseña cifrada o una llave privada (con soporte opcional de frase de paso). El formulario de perfil MUST ofrecer un botón "Examinar..." junto al campo de ruta de llave privada que active el selector de archivos nativo del sistema operativo para ubicar archivos de clave (`.pem`, `.key`, `.ppk`, `id_rsa`, etc.) y auto-completar la ruta absoluta en el input. La ruta de la llave MUST ser normalizada convirtiendo todas las barras invertidas '\' a barras diagonales '/' antes de guardarse.

#### Scenario: Configurar autenticación con archivo de llave privada
- **WHEN** el usuario selecciona la opción de llave privada en el perfil, introduce la ruta de la llave `.pem` y una frase de paso, y guarda
- **THEN** el sistema almacena la ruta de la llave y la frase de paso cifrada en la base de datos asociada a ese perfil

#### Scenario: Configurar autenticación con selector de archivos del SO
- **WHEN** el usuario hace clic en el botón "Examinar..." en la sección de llave privada
- **THEN** el sistema abre el diálogo nativo de archivos del SO, y al seleccionar un archivo de clave, escribe la ruta absoluta en el campo de la llave privada

#### Scenario: Normalización de la ruta de la llave privada
- **WHEN** el usuario guarda un perfil con una ruta de llave que contiene barras invertidas '\'
- **THEN** el sistema MUST normalizar la ruta convirtiendo todas las barras invertidas '\' a barras diagonales '/' antes de guardar la información en la base de datos
