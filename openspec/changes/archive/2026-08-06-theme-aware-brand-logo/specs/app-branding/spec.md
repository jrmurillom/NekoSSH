## MODIFIED Requirements

### Requirement: Visual App Logo
La interfaz del cliente NekoSSH SHALL mostrar el logo oficial al lado izquierdo del título en la barra lateral, asegurando una presentación limpia y estéticamente alineada. El asset del logo SHALL corresponder al tema conceptual activo (un PNG por id de tema); si el id es desconocido o falta el asset, el sistema MUST usar el logo del tema `nekossh`.

#### Scenario: Visualización del Logo en Sidebar
- **WHEN** la aplicación es cargada en el cliente
- **THEN** se debe renderizar el elemento del logo junto al título "NekoSSH" manteniendo el espaciado de la estética del tema activo

#### Scenario: Logo alineado al tema al iniciar
- **WHEN** la aplicación arranca con un tema guardado distinto de `nekossh` (p. ej. `hatsune-miku`)
- **THEN** el logo del sidebar muestra el PNG de ese tema, no el logo rosa por defecto

#### Scenario: Fallback de logo
- **WHEN** el tema activo no tiene un PNG asociado o el id no está en el catálogo
- **THEN** el sidebar muestra el logo del tema `nekossh`
