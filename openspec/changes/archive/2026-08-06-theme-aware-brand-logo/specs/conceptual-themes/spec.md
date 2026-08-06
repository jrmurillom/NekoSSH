## ADDED Requirements

### Requirement: Brand Logo Sync with Theme
Al aplicar o restaurar un tema conceptual, el sistema SHALL actualizar el logo de marca del sidebar (`.brand-logo`) para usar el PNG asociado al id del tema activo, en el mismo ciclo en que se actualizan `data-theme` y los colores de xterm.js.

#### Scenario: Cambio de tema actualiza el logo
- **WHEN** el usuario selecciona un tema distinto en el popover de preferencias
- **THEN** el `src` del logo del sidebar apunta al PNG de ese tema y el título/acentos CSS ya reflejan la misma paleta

#### Scenario: Restauración de tema también restaura el logo
- **WHEN** la aplicación inicia y restaura el tema desde `localStorage`
- **THEN** el logo del sidebar coincide con el tema restaurado sin requerir un clic adicional del usuario
