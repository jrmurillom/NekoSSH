## ADDED Requirements

### Requirement: Cuadrícula interna del panel unificado
El `.terminal-panel` de la pestaña activa SHALL contener una cuadrícula de celdas de terminal (hasta cuatro) sin romper la fusión visual pestaña–panel ni el glow sakura del contenedor unificado. El padding de seguridad del panel SHALL seguir evitando que el border-radius recorte el texto de cualquier celda visible.

#### Scenario: Panel con varias celdas mantiene glow unificado
- **WHEN** el contexto muestra dos o más shells en el grid
- **THEN** el resplandor y el borde redondeado envuelven el bloque completo del panel, no cada celda por separado

#### Scenario: Texto visible en celdas del grid
- **WHEN** hay output cerca del borde de una celda en layout 2×2
- **THEN** el texto permanece legible dentro del área útil (sin recorte por el radius del panel)
