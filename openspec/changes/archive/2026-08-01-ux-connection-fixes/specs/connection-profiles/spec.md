## MODIFIED Requirements

### Requirement: Autenticación por Contraseña o Llave Privada
El sistema SHALL permitir configurar para cada perfil una credencial de autenticación que puede ser una contraseña cifrada o una llave privada (con soporte opcional de frase de paso). El formulario de perfil MUST ofrecer un botón "Examinar..." junto al campo de ruta de llave privada que active el selector de archivos nativo del sistema operativo para ubicar archivos de clave (`.pem`, `.key`, `.ppk`, `id_rsa`, etc.) y auto-completar la ruta absoluta en el input.

#### Scenario: Configurar autenticación con selector de archivos del SO
- **WHEN** el usuario hace clic en el botón "Examinar..." en la sección de llave privada
- **THEN** el sistema abre el diálogo nativo de archivos del SO, y al seleccionar un archivo de clave, escribe la ruta absoluta en el campo de la llave privada
