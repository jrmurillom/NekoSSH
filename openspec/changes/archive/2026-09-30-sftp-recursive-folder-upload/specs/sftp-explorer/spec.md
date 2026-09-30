## ADDED Requirements

### Requirement: Subida recursiva de carpetas por arrastrar y soltar
El sistema SHALL permitir arrastrar y soltar carpetas locales (o una selección mixta de archivos y directorios) sobre el panel del explorador SFTP para transferirlas al servidor remoto. Al recibir elementos locales, el sistema SHALL inspeccionar si corresponden a archivos regulares o directorios. Si se detectan carpetas, el sistema SHALL recorrer recursivamente su árbol local acumulando la lista de todos los archivos y subdirectorios con sus rutas relativas. El diálogo de confirmación previo SHALL informar la cantidad de carpetas, archivos y volumen en bytes detectados. El sistema SHALL asegurar y crear de forma idempotente en el servidor remoto cada directorio intermedio (`sftp.mkdir`) antes de transmitir sus archivos. La transferencia de cada archivo contenido SHALL realizarse en streaming secuencial de 64 KiB ($O(1)$ de memoria), emitiendo el progreso acumulado por canal IPC y respetando la cancelación cooperativa.

#### Scenario: Subir una carpeta con subdirectorios y archivos
- **WHEN** el usuario arrastra y suelta una carpeta local que contiene subdirectorios y múltiples archivos (ej. un proyecto web con `assets/`, `img/`, `_next/`) sobre el explorador SFTP
- **THEN** el sistema analiza la estructura, presenta el diálogo de confirmación con el desglose de carpetas y archivos, crea los directorios remotos correspondientes y transfiere la totalidad de los archivos respetando su jerarquía original sin arrojar errores de acceso

#### Scenario: Selección mixta de archivos y carpetas
- **WHEN** el usuario arrastra simultáneamente uno o más archivos sueltos y una o más carpetas hacia el explorador
- **THEN** el sistema procesa los archivos en el directorio de destino actual y recorre y crea recursivamente las carpetas, completando la transferencia del conjunto completo

#### Scenario: Creación de jerarquías de directorios remotos anidados
- **WHEN** un archivo local pertenece a una ruta anidada (ej. `techpeople/assets/css/main.css`) cuyas carpetas no existen en el servidor remoto
- **THEN** el sistema crea secuencialmente cada directorio padre ausente en el servidor SFTP antes de proceder con la escritura del archivo `.nekossh.part`
