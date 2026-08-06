## ADDED Requirements

### Requirement: Wallpaper Sync with Theme
Al aplicar o restaurar un tema conceptual, el sistema SHALL cargar y aplicar el wallpaper (imagen + opacidad) persistido para ese id de tema, en el mismo ciclo en que se actualizan `data-theme`, los colores de xterm.js y el logo de marca.

#### Scenario: Cambio de tema restaura su fondo
- **WHEN** el usuario selecciona el tema B y B tenía un fondo guardado
- **THEN** el panel de terminal muestra el fondo (y opacidad) de B, no el de A

#### Scenario: Volver al tema A recupera su fondo
- **WHEN** el usuario vuelve al tema A tras haber estado en B
- **THEN** se reaplica el wallpaper previamente configurado en A

#### Scenario: Restauración al boot
- **WHEN** la aplicación inicia y restaura el tema desde `localStorage`
- **THEN** el wallpaper del tema restaurado se aplica sin acción adicional del usuario
