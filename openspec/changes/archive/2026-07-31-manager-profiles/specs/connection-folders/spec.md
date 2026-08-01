## ADDED Requirements

### Requirement: Carpetas de conexiones en un nivel
El sistema SHALL permitir crear, listar, renombrar y eliminar carpetas (grupos) que organizan conexiones SSH. Las carpetas SHALL formar un único nivel (sin subcarpetas).

#### Scenario: Crear carpeta
- **WHEN** el usuario activa la acción de agregar carpeta (icono de carpeta o control equivalente en el panel de conexiones)
- **THEN** el sistema crea una carpeta con nombre por defecto editable y la muestra en el árbol del sidebar

#### Scenario: Expandir y colapsar carpeta
- **WHEN** el usuario activa el chevron de una carpeta
- **THEN** el sistema muestra u oculta las conexiones hijas sin afectar otras carpetas

#### Scenario: Eliminar carpeta con confirmación
- **WHEN** el usuario elimina una carpeta que contiene N conexiones y confirma
- **THEN** el sistema elimina la carpeta y sus conexiones asociadas (cascade)

### Requirement: Renombrado inline de carpeta
El sistema SHALL permitir editar el nombre de una carpeta de forma inline en el árbol (sin modal obligatorio para el rename).

#### Scenario: Guardar nombre con Enter
- **WHEN** el usuario edita el nombre de una carpeta inline y confirma con Enter
- **THEN** el sistema persiste el nuevo nombre y actualiza la fila del árbol

#### Scenario: Cancelar con Escape
- **WHEN** el usuario edita inline y pulsa Escape
- **THEN** el sistema descarta el cambio y restaura el nombre anterior
