## ADDED Requirements

### Requirement: Modularidad y Cobertura de Pruebas Unitarias de Dominio
El sistema NekoSSH MUST contar con módulos desacoplados para la lógica pura de conexiones y rutas SFTP, y MUST mantener una suite de pruebas unitarias automatizadas que validen las transiciones de la máquina de estados de edición, la sanitización de comandos elevados y el ordenamiento de perfiles.

#### Scenario: Ejecución de pruebas unitarias valiosas
- **WHEN** se ejecutan los comandos de prueba `cargo test` y `npm run test`
- **THEN** la suite valida exitosamente la lógica pura de la máquina de estados, sanitización de comillas y manipulación de rutas SFTP sin errores.
