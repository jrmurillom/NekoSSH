## ADDED Requirements

### Requirement: Pestaña Notas en Barra Lateral
La barra lateral SHALL incluir una cuarta pestaña denominada "Notas" con un icono representativo de libreta o nota.

#### Scenario: Visualización de Pestaña Notas
- **WHEN** la aplicación es iniciada
- **THEN** la barra lateral del sistema muestra la pestaña de notas junto con las demás pestañas

### Requirement: Creación Rápida de Nota
La cabecera de la sección Notas SHALL proveer un botón `(+)` para crear instantáneamente una nota con título autogenerado y contenido vacío.

#### Scenario: Creación de nota vacía exitosa
- **WHEN** el usuario hace clic en el botón `(+)`
- **THEN** el sistema crea una nota en la base de datos local y la añade a la lista lateral de notas

### Requirement: Detalle y Edición de Nota
Al hacer clic en una nota de la lista, se SHALL abrir un modal flotante con diseño glassmorphism adaptado al tema de color de la app que contenga un editor de texto plano para escribir en formato Markdown.

#### Scenario: Abrir nota desde el listado
- **WHEN** el usuario selecciona una nota de la lista lateral
- **THEN** el sistema despliega el modal flotante, carga el título inline y enfoca el editor de texto con el contenido de la nota

### Requirement: Renombrado Inline
El título de la nota SHALL ser un elemento de texto editable que permita renombrar la nota al hacerle clic dentro del modal flotante.

#### Scenario: Editar título inline
- **WHEN** el usuario hace clic en el título de la nota en el modal y edita el texto
- **THEN** el sistema actualiza el título en el listado lateral y en la persistencia local al terminar de editar

### Requirement: Auto Guardado de Notas
El sistema SHALL persistir de manera automática todos los cambios realizados en el contenido de la nota y su título en la base de datos SQLite sin necesidad de confirmación manual del usuario.

#### Scenario: Guardado al cerrar o perder foco
- **WHEN** el usuario edita el texto de la nota y hace clic en cerrar o sale del editor
- **THEN** el sistema guarda el contenido actualizado en la base de datos local SQLite

### Requirement: Eliminación de Nota
El modal flotante SHALL proveer un botón de eliminar con el icono `trash2` de Lucide que borre la nota permanentemente del sistema tras confirmar la acción.

#### Scenario: Eliminar nota exitosa
- **WHEN** el usuario hace clic en el botón de eliminar y confirma la acción
- **THEN** el sistema borra la nota de la base de datos y la remueve de la lista lateral
