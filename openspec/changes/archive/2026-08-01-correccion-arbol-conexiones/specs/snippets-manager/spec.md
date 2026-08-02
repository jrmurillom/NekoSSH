## ADDED Requirements

### Requirement: Botón Snippets con fill primario del tema
El control de apertura de Snippets en `sidebar-footer` (`.snippets-footer-btn`) SHALL usar el **fill primario** del tema Cyber-Sakura — el mismo gradiente/fondo y contraste de texto que el botón primario de producto (`.btn-primary`: `linear-gradient` con `--color-sakura-neon` y texto blanco), no un estilo ghost/outline con fondo oscuro. Este requisito es solo de look del control existente; el comportamiento de apertura del modal MUST permanecer igual. El engrane de preferencias NO MUST adoptar ese fill primario.

#### Scenario: Fill alineado al primario
- **WHEN** el usuario ve el botón Snippets en el footer del sidebar
- **THEN** el botón se percibe con relleno rosa/sakura sólido (gradiente primario del tema) y texto/icono en contraste claro, no como outline sobre fondo oscuro

#### Scenario: Apertura intacta
- **WHEN** el usuario activa el botón Snippets
- **THEN** se abre el modal del gestor como antes
