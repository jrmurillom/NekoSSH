## ADDED Requirements

### Requirement: Presentación densa del árbol de carpetas
En el sidebar Servidores, las filas de carpeta y sus conexiones hijas SHALL presentarse con **densidad de lista** (no tarjetas decorativas). El bloque de carpeta MUST indicar jerarquía padre/hijo mediante indentación de los hijos y una guía vertical (p. ej. borde izquierdo sutil). La fila de carpeta SHALL conservar el chrome funcional existente (chevron + icono carpeta + nombre + `+`) sin basurero inline. La presentación visual MUST alinearse al SSOT visual del change (`docs/design/preview-connection-tree-dense.html`).

#### Scenario: Fila de carpeta compacta
- **WHEN** el usuario ve una carpeta en el árbol
- **THEN** la fila se lee como ítem de lista densa (altura/padding compactos), no como tarjeta decorativa gruesa

#### Scenario: Guía e indentación de hijos
- **WHEN** una carpeta está expandida y tiene conexiones (o el estado vacío «Sin conexiones»)
- **THEN** el contenido hijo aparece indentado bajo la carpeta con una guía vertical visible que refuerza la relación padre/hijo

#### Scenario: Comportamiento de expand/collapse intacto
- **WHEN** el usuario hace clic primario en la fila de una carpeta (fuera del botón `+`)
- **THEN** el sistema muestra u oculta los hijos como antes; el cambio de estilo NO MUST alterar esa interacción
