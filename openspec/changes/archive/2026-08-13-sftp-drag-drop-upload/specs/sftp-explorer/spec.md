## MODIFIED Requirements

### Requirement: Arrastrar y soltar archivos para subir al explorador
El sistema SHALL permitir arrastrar archivos locales desde el host y soltarlos sobre el panel del explorador de archivos SFTP para subirlos al servidor remoto. Al arrastrar archivos sobre el panel del explorador de archivos, el sistema SHALL mostrar un overlay visual indicando la acción de soltar y el destino actual. El overlay SHALL derivar su color, transparencia y tipografía de los tokens del tema conceptual activo, sin valores de color fijos, de modo que acompañe al tema como el resto del panel. Si el cursor se sitúa sobre un nodo de carpeta específico del árbol, el sistema SHALL resaltar visualmente esa carpeta y actualizar el destino en el overlay para reflejar esa subcarpeta; si el cursor se sitúa sobre un archivo, el destino SHALL ser la carpeta que lo contiene; si no se apunta a ninguna fila, el destino SHALL ser la ruta actual del explorador. Al soltar los archivos, el sistema SHALL solicitar **siempre** confirmación al usuario mediante un diálogo A1 del marco de trabajo existente, indicando el nombre del archivo (o la cantidad de archivos) y la ruta remota de destino. Si el usuario confirma, el sistema SHALL subir los archivos correspondientes a la ruta remota y actualizar el listado del explorador. Las subidas SHALL usar la sesión del shell padre del contexto activo.

#### Scenario: Arrastrar sobre el panel general del explorador
- **WHEN** el usuario arrastra archivos sobre el panel del explorador de archivos sin apuntar a una carpeta específica
- **THEN** el sistema muestra el overlay de dropzone con el path actual de navegación (`explorerCwd`) como destino

#### Scenario: Arrastrar apuntando a una carpeta del árbol
- **WHEN** el usuario arrastra archivos apuntando a una carpeta específica del árbol
- **THEN** el sistema resalta esa carpeta y muestra el path de esa carpeta como destino en el overlay de dropzone

#### Scenario: Arrastrar apuntando a un archivo del árbol
- **WHEN** el usuario arrastra archivos apuntando a una fila de archivo del árbol
- **THEN** el destino mostrado es la carpeta que contiene ese archivo

#### Scenario: Overlay alineado al tema activo
- **WHEN** el usuario cambia el tema conceptual y vuelve a arrastrar archivos sobre el explorador
- **THEN** el overlay y el resaltado de carpeta se muestran con los colores y la transparencia del tema activo

#### Scenario: Salir del panel durante el arrastre
- **WHEN** el usuario arrastra fuera del panel del explorador o cancela el arrastre
- **THEN** el sistema oculta el overlay y elimina el resaltado de la carpeta destino

#### Scenario: Cancelar subida tras soltar
- **WHEN** el usuario suelta los archivos y cancela el diálogo de confirmación
- **THEN** no se sube ningún archivo y el explorador permanece sin cambios

#### Scenario: Confirmar subida de un archivo
- **WHEN** el usuario suelta un archivo, confirma el diálogo y el upload SFTP es exitoso
- **THEN** el sistema sube el archivo a la ruta de destino remota y refresca el explorador de archivos

#### Scenario: Confirmar subida de varios archivos
- **WHEN** el usuario suelta varios archivos y confirma el diálogo
- **THEN** el diálogo indica la cantidad de archivos y la ruta destino, y el sistema sube todos los archivos confirmados

#### Scenario: Arrastre sin sesión o con otra pestaña activa
- **WHEN** el usuario arrastra archivos y no hay sesión SSH activa, o el panel Archivos no está visible
- **THEN** el sistema no muestra el overlay y no realiza ninguna subida

## ADDED Requirements

### Requirement: Confirmación de sobreescritura al subir
Antes de reemplazar un archivo remoto existente durante una subida por arrastrar y soltar, el sistema SHALL detectar la colisión de nombre en el directorio destino y SHALL solicitar confirmación explícita al usuario mediante un diálogo A1 del marco de trabajo existente, indicando el nombre del archivo y la ruta de destino. Si el usuario no confirma, ese archivo NO SHALL subirse, y el resto del lote SHALL continuar según la decisión del usuario para cada archivo. La detección de colisiones SHALL NOT requerir un segundo login SSH.

#### Scenario: Archivo existente confirmado para reemplazo
- **WHEN** el archivo soltado ya existe en la ruta destino y el usuario confirma el reemplazo
- **THEN** el sistema sube el archivo reemplazando el remoto y refresca el explorador

#### Scenario: Archivo existente no confirmado
- **WHEN** el archivo soltado ya existe en la ruta destino y el usuario cancela la confirmación de reemplazo
- **THEN** ese archivo no se sube y el archivo remoto permanece sin cambios

#### Scenario: Sin colisión de nombres
- **WHEN** ningún archivo soltado existe en la ruta destino
- **THEN** el sistema no muestra diálogo de sobreescritura y procede con la subida ya confirmada

### Requirement: Progreso y resultado de la subida por arrastre
Durante una subida iniciada por arrastrar y soltar, el sistema SHALL mostrar el avance al usuario en el overlay de estado del explorador, sin desplazar el árbol de archivos. Al finalizar, el sistema SHALL informar el resultado. Si una subida individual falla, el sistema SHALL continuar con los archivos restantes y SHALL informar al final cuáles fallaron, manteniendo la sesión PTY utilizable.

#### Scenario: Progreso visible durante la subida
- **WHEN** el sistema está subiendo un lote de archivos confirmado por el usuario
- **THEN** el explorador muestra el avance de la operación como mensaje de estado sin mover el árbol

#### Scenario: Fallo parcial del lote
- **WHEN** uno de los archivos del lote falla al subir y los demás son correctos
- **THEN** el sistema termina el resto del lote, informa cuáles archivos fallaron y la sesión SSH permanece usable

#### Scenario: Refresco al completar
- **WHEN** el lote de subida termina con al menos un archivo subido
- **THEN** el sistema refresca el listado del explorador en la ruta destino
