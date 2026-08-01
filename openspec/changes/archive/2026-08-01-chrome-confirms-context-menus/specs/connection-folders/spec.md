## MODIFIED Requirements

### Requirement: Carpetas de conexiones en un nivel
El sistema SHALL permitir crear, listar, renombrar y eliminar carpetas (grupos) que organizan conexiones SSH. Las carpetas SHALL formar un único nivel (sin subcarpetas). En la fila de carpeta, el chrome SHALL mostrar acción de agregar conexión (`+`) y NO MUST mostrar basurero inline. Eliminar y renombrar MUST estar disponibles desde menú contextual (clic derecho en la fila).

#### Scenario: Crear carpeta
- **WHEN** el usuario activa la acción de agregar carpeta (icono de carpeta o control equivalente en el panel de conexiones)
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
