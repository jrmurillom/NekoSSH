## MODIFIED Requirements

### Requirement: Presentación densa del árbol de carpetas
En el sidebar Servidores, las filas de carpeta SHALL presentarse con **densidad de lista plana**: sin chrome de caja/tarjeta (sin borde visible que forme rectángulo en idle, hover o contexto activo). Las carpetas del árbol MUST iniciar en estado completamente colapsado por defecto al abrir la aplicación. Las filas de carpeta NO MUST mostrar retención de tinte rosa o cambio de fondo permanente tras recibir un clic de selección/activación de contexto (`.is-active-context` background transparente). El hover sutil de la fila de carpeta solo permanecerá activo mientras el cursor sobrevuele el elemento.

#### Scenario: Fila de carpeta sin caja ni fondo persistente al clic
- **WHEN** el usuario hace clic en una fila de carpeta para expandir o colapsar sus elementos
- **THEN** la carpeta alterna su estado visual pero conserva su fondo transparente sin dejar un tinte activo permanente

#### Scenario: Árbol colapsado al iniciar la aplicación
- **WHEN** la aplicación NekoSSH se abre por primera vez o se recarga
- **THEN** todas las carpetas en el panel de Servidores inician colapsadas por defecto
