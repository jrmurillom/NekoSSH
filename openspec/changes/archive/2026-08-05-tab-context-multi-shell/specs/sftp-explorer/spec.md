## ADDED Requirements

### Requirement: Explorador ligado al padre del contexto
El explorador SFTP del sidebar SHALL asociar listados, navegación, clipboard SCP y “Abrir en Terminal” al `terminal_id` del **shell padre** del contexto de pestaña activo, no al shell hijo enfocado.

#### Scenario: Listar con hijo enfocado
- **WHEN** el usuario tiene foco en un shell hijo y abre o refresca Archivos
- **THEN** las operaciones SFTP usan la Session del padre del contexto

#### Scenario: Abrir en Terminal desde explorador
- **WHEN** el usuario elige “Abrir en Terminal” sobre un directorio
- **THEN** el `cd` se envía al PTY del padre, no al hijo enfocado

#### Scenario: Cambiar de pestaña cambia SFTP
- **WHEN** el usuario cambia a otra pestaña/contexto
- **THEN** el explorador se rebinda al padre de ese contexto (cwd/caché del contexto)
