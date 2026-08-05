# snippets-manager

## Purpose

Gestor de snippets de comandos con categorías de un solo nivel, persistencia SQLite local, modal glass en el shell y copia al portapapeles (sin inserción PTY).

## Requirements

### Requirement: Persistencia de categorías y snippets
El sistema SHALL almacenar categorías de snippet de un solo nivel y snippets asociados en SQLite local (`nekossh.db`). Cada snippet MUST pertenecer a exactamente una categoría y MUST tener título y cuerpo de texto. Eliminar una categoría MUST eliminar en cascada sus snippets. El producto NO MUST persistir snippets solo en `localStorage` ni como blob opaco en `app_preferences`.

#### Scenario: Crear categoría y snippet
- **WHEN** el usuario crea una categoría con nombre válido y luego un snippet con título y cuerpo en esa categoría
- **THEN** ambos quedan persistidos en SQLite y aparecen al reabrir el gestor

#### Scenario: Cascade al eliminar categoría
- **WHEN** el usuario confirma eliminar una categoría que tiene snippets
- **THEN** la categoría y todos sus snippets dejan de existir en la base de datos

#### Scenario: Editar snippet
- **WHEN** el usuario modifica el título o el cuerpo de un snippet existente y guarda
- **THEN** los cambios persisten y se reflejan en la lista del modal

### Requirement: Seed demo en base vacía
Al primer uso con tablas de snippets vacías, el sistema SHALL sembrar categorías demo **Apache**, **Tomcat** y **Permisos**, cada una con al menos dos snippets de ejemplo. El seed MUST ser idempotente respecto a una base ya poblada (no duplicar). Los datos sembrados MUST ser editables y eliminables como cualquier dato de usuario.

#### Scenario: Primer arranque vacío
- **WHEN** no existen categorías de snippet y el usuario abre el gestor (o se invoca el seed)
- **THEN** aparecen las tres categorías demo con snippets de ejemplo

#### Scenario: No re-sembrar si ya hay datos
- **WHEN** ya existe al menos una categoría o snippet y se vuelve a invocar el seed
- **THEN** el sistema no inserta duplicados de las categorías demo

### Requirement: Apertura por botón en sidebar-footer
El producto SHALL exponer un control en `sidebar-footer` (franja inferior del sidebar) que abre el modal del gestor de snippets. Este slice NO MUST registrar un atajo de teclado global para abrir el gestor.

#### Scenario: Abrir con el botón
- **WHEN** el usuario activa el botón de Snippets en el footer del sidebar
- **THEN** se muestra el modal del gestor de snippets

#### Scenario: Sin atajo en este slice
- **WHEN** el usuario pulsa combinaciones de teclado habituales sin haber definido un shortcut de snippets
- **THEN** el gestor no se abre por teclado como parte de este alcance

### Requirement: Botón Snippets con fill primario del tema
El control de apertura de Snippets en `sidebar-footer` (`.snippets-footer-btn`) SHALL usar el **fill primario** del tema Cyber-Sakura — el mismo gradiente/fondo y contraste de texto que el botón primario de producto (`.btn-primary`: `linear-gradient` con `--color-sakura-neon` y texto blanco), no un estilo ghost/outline con fondo oscuro. Este requisito es solo de look del control existente; el comportamiento de apertura del modal MUST permanecer igual. El engrane de preferencias NO MUST adoptar ese fill primario.

#### Scenario: Fill alineado al primario
- **WHEN** el usuario ve el botón Snippets en el footer del sidebar
- **THEN** el botón se percibe con relleno rosa/sakura sólido (gradiente primario del tema) y texto/icono en contraste claro, no como outline sobre fondo oscuro

#### Scenario: Apertura intacta
- **WHEN** el usuario activa el botón Snippets
- **THEN** se abre el modal del gestor como antes

### Requirement: Modal glass con lista plana, chips y búsqueda
El gestor SHALL presentarse en un modal con chrome glass alineado al modal de perfil (overlay + panel), no como dialog de confirmación A1. La UI MUST incluir: chips de categoría (incluida opción “Todas”), lista plana de snippets (sin árbol expandible) y un campo de búsqueda que filtre por título y/o cuerpo. Las categorías MUST ser de un solo nivel (sin anidación).

#### Scenario: Filtrar por chip
- **WHEN** el usuario selecciona la chip de una categoría
- **THEN** la lista muestra solo los snippets de esa categoría

#### Scenario: Buscar por texto
- **WHEN** el usuario escribe un término presente en el título o cuerpo de un snippet
- **THEN** la lista muestra coincidencias y oculta el resto (respetando el filtro de chip activo)

#### Scenario: Sin árbol anidado
- **WHEN** el usuario ve el gestor
- **THEN** no hay nodos de categoría expandibles ni categorías hijas anidadas

### Requirement: CRUD in-modal
El modal SHALL permitir crear categorías, crear snippets, editar título/cuerpo de un snippet existente y eliminar categorías o snippets, sin navegar fuera del shell. Los formularios de alta/edición MUST vivir en el mismo modal (inline o panel interno).

#### Scenario: Alta de snippet
- **WHEN** el usuario elige “Nuevo snippet”, completa categoría, título y cuerpo, y guarda
- **THEN** el snippet aparece en la lista filtrable sin cerrar la app ni abrir otra ventana

#### Scenario: Cancelar edición
- **WHEN** el usuario inicia editar/crear y cancela o cierra el modal sin guardar
- **THEN** no se persisten los cambios del draft

### Requirement: Copiar snippet al portapapeles
Cada fila de snippet SHALL ofrecer una acción Copiar que coloca el **cuerpo** del snippet en el portapapeles del sistema. Esta acción NO MUST escribir al PTY SSH ni invocar inserción de terminal.

#### Scenario: Copiar cuerpo
- **WHEN** el usuario activa Copiar en un snippet
- **THEN** el cuerpo del snippet queda en el portapapeles y no se envía input a ninguna sesión SSH

#### Scenario: Fallo de clipboard
- **WHEN** el entorno deniega el acceso al portapapeles
- **THEN** el sistema informa el error al usuario (dialog/aviso de producto) y no simula éxito

### Requirement: Eliminar con confirmación A1
Eliminar un snippet o una categoría SHALL requerir confirmación mediante el dialog glass centrado (`confirmDialog` / patrón A1). Escape o Cancelar MUST abortar el borrado. El copy de eliminación de categoría con snippets MUST dejar claro el impacto (borrado en cascada de snippets asociados).

#### Scenario: Confirmar borrado de snippet
- **WHEN** el usuario elige Eliminar en un snippet y confirma en el dialog A1
- **THEN** el snippet se elimina de SQLite y desaparece de la lista

#### Scenario: Cancelar borrado
- **WHEN** el dialog A1 de eliminación está abierto y el usuario pulsa Escape o Cancelar
- **THEN** no se elimina el recurso

### Requirement: Presentación alineada al tema Cyber-Sakura
El modal de snippets MUST alinearse visualmente a los tokens del modal de perfil (`DESIGN.md`): campo de búsqueda y control “+ Snippet” con chrome temático (no controles nativos/desconectados). La lista MUST separar filas de forma visible. Cada fila MUST mostrar título y cuerpo/comando **sin** repetir el nombre de la categoría. Crear categoría MUST usar un panel/form **dentro del mismo modal**; el producto NO MUST usar `window.prompt` ni un diálogo del SO para ese alta.

#### Scenario: Controles temáticos
- **WHEN** el usuario ve el toolbar del gestor
- **THEN** búsqueda y “+ Snippet” comparten el look Cyber-Sakura del modal de perfil

#### Scenario: Fila sin categoría redundante
- **WHEN** el usuario ve una fila de snippet
- **THEN** aparece el título y el comando/cuerpo, no un prefijo “Categoría · …”

#### Scenario: Alta de categoría in-modal
- **WHEN** el usuario activa “+” de nueva categoría
- **THEN** se abre un formulario dentro del modal de snippets (no un prompt del sistema)
