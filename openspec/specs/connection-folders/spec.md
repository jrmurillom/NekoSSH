# connection-folders

## Purpose

Organización de conexiones SSH en carpetas de un solo nivel (grupos) en el sidebar Servidores.

## Requirements

### Requirement: Carpetas de conexiones en un nivel
El sistema SHALL permitir crear, listar, renombrar y eliminar carpetas (grupos) que organizan conexiones SSH. Las carpetas SHALL formar un único nivel (sin subcarpetas). En la fila de carpeta, el chrome SHALL mostrar acción de agregar conexión (`+`) y NO MUST mostrar basurero inline. Eliminar y renombrar MUST estar disponibles desde menú contextual (clic derecho en la fila). La acción de **agregar carpeta** MUST exponerse como icon-button en el **header de zona** del panel Servidores (junto al icono de crear conexión), no como compañero de un CTA de texto “Nueva conexión” en una toolbar split.

#### Scenario: Crear carpeta desde el header de zona
- **WHEN** el usuario activa el icono de agregar carpeta en el header de zona del panel Servidores
- **THEN** el sistema crea una carpeta con nombre por defecto editable y la muestra en el árbol del sidebar

#### Scenario: Expandir y colapsar carpeta
- **WHEN** el usuario hace clic primario en la fila de una carpeta (chevron, icono o nombre)
- **THEN** el sistema muestra u oculta las conexiones hijas sin afectar otras carpetas; el botón `+` no dispara el toggle

#### Scenario: Eliminar carpeta desde menú contextual
- **WHEN** el usuario abre el menú contextual de una carpeta, elige eliminar y confirma en el dialog glass
- **THEN** el sistema elimina la carpeta y sus conexiones asociadas (cascade)

### Requirement: Renombrado inline de carpeta
El sistema SHALL permitir editar el nombre de una carpeta de forma inline en el árbol (sin modal obligatorio para el rename). El modo inline MUST iniciarse desde el menú contextual (“Cambiar nombre” / “Renombrar”), no por doble clic.

#### Scenario: Iniciar rename desde menú
- **WHEN** el usuario elige renombrar en el menú contextual de la carpeta
- **THEN** la fila entra en modo input inline con el nombre actual seleccionado

#### Scenario: Guardar nombre con Enter
- **WHEN** el usuario edita el nombre de una carpeta inline y confirma con Enter
- **THEN** el sistema persiste el nuevo nombre y actualiza la fila del árbol

#### Scenario: Cancelar con Escape
- **WHEN** el usuario edita inline y pulsa Escape
- **THEN** el sistema descarta el cambio y restaura el nombre anterior

### Requirement: Presentación densa del árbol de carpetas
En el sidebar Servidores, las filas de carpeta SHALL presentarse con **densidad de lista plana**: sin chrome de caja/tarjeta (sin borde visible que forme rectángulo en idle, hover o contexto activo). Las carpetas del árbol MUST iniciar en estado completamente colapsado por defecto al abrir la aplicación. Las filas de carpeta NO MUST mostrar retención de tinte rosa o cambio de fondo permanente tras recibir un clic de selección/activación de contexto (`.is-active-context` background transparente). El hover sutil de la fila de carpeta solo permanecerá activo mientras el cursor sobrevuele el elemento.

#### Scenario: Fila de carpeta sin caja ni fondo persistente al clic
- **WHEN** el usuario hace clic en una fila de carpeta para expandir o colapsar sus elementos
- **THEN** la carpeta alterna su estado visual pero conserva su fondo transparente sin dejar un tinte activo permanente

#### Scenario: Árbol colapsado al iniciar la aplicación
- **WHEN** la aplicación NekoSSH se abre por primera vez o se recarga
- **THEN** todas las carpetas en el panel de Servidores inician colapsadas por defecto

#### Scenario: Guía e indentación de hijos
- **WHEN** una carpeta está expandida y tiene conexiones (o el estado vacío «Sin conexiones»)
- **THEN** el contenido hijo aparece indentado bajo la carpeta con una guía vertical visible que refuerza la relación padre/hijo

#### Scenario: Comportamiento de expand/collapse intacto
- **WHEN** el usuario hace clic primario en la fila de una carpeta (fuera del botón `+`)
- **THEN** el sistema muestra u oculta los hijos como antes; el cambio de estilo NO MUST alterar esa interacción

### Requirement: Header de zona Conexiones en el panel Servidores
El panel Servidores SHALL mostrar, encima del árbol, un **header de zona** con label visible **Conexiones** (español latino) a la izquierda y, a la derecha, dos icon-buttons: crear conexión y crear carpeta. Este header MUST reemplazar la toolbar split que combinaba el CTA de texto “Nueva conexión” con el icono de carpeta. El header NO MUST aparecer en la pestaña Archivos ni en el `sidebar-footer`. El producto NO MUST mostrar el label en inglés “Connections”.

#### Scenario: Header con label e iconos
- **WHEN** el usuario está en la pestaña Servidores
- **THEN** ve el label Conexiones y los dos iconos de crear conexión y crear carpeta encima del árbol

#### Scenario: Sin label en inglés
- **WHEN** el usuario mira el header de zona del árbol
- **THEN** no aparece el texto “Connections” como label visible

#### Scenario: Sin toolbar CTA split
- **WHEN** el usuario mira las acciones del panel Servidores
- **THEN** no hay una fila con botón primario de texto “Nueva conexión” compartiendo espacio con el icono de carpeta
