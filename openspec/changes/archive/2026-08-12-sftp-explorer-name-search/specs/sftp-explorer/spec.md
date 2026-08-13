## ADDED Requirements

### Requirement: Filtro por nombre en el árbol del explorador
El sistema SHALL permitir filtrar por nombre las entradas **ya pintadas** del árbol del explorador SFTP (contenido del directorio actual y nodos hijos de carpetas ya expandidas y cargadas). El filtro SHALL ser por substring sin distinguir mayúsculas y minúsculas. El filtro SHALL NOT disparar listados SFTP adicionales ni búsqueda recursiva remota. Un nodo SHALL mostrarse si su nombre coincide con el filtro **o** si algún descendiente actualmente visible (bajo la expansión cargada) coincide, de modo que se conserve la jerarquía hacia la coincidencia. El panel Archivos SHALL exponer un campo de filtro y un control **×** que limpia el texto del filtro y vuelve a mostrar todas las entradas pintables. Cuando el filtro no deja ninguna fila visible, el explorador SHALL mostrar un estado vacío de sin coincidencias.

#### Scenario: Filtrar entradas del nivel actual
- **WHEN** el usuario escribe un texto en el campo de filtro del explorador y hay entradas listadas en el cwd
- **THEN** solo se muestran las filas cuyo nombre contiene ese texto (sin distinguir mayúsculas) o que anclan un descendiente visible coincidente

#### Scenario: Filtrar hijos de carpeta ya expandida
- **WHEN** una carpeta está expandida con hijos cargados y el usuario aplica un filtro por nombre
- **THEN** los hijos ya pintados también se evalúan contra el filtro; los nodos colapsados o no cargados no se listan ni se buscan en remoto

#### Scenario: Padre sin match con hijo coincidente
- **WHEN** el nombre de una carpeta no coincide con el filtro pero un hijo visible sí coincide
- **THEN** la carpeta permanece visible como contenedor del hijo coincidente

#### Scenario: Limpiar filtro con ×
- **WHEN** el usuario activa el control × del filtro (o deja el campo vacío)
- **THEN** el filtro se limpia y el árbol vuelve a mostrar todas las entradas pintables sin el filtro

#### Scenario: Sin coincidencias
- **WHEN** el filtro no coincide con ninguna fila pintable del árbol
- **THEN** el explorador muestra un estado vacío indicando que no hay coincidencias

#### Scenario: Sin llamadas SFTP por filtrar
- **WHEN** el usuario escribe o limpia el filtro
- **THEN** el sistema no invoca `sftp_list_dir` ni otra operación remota solo por el cambio del filtro
