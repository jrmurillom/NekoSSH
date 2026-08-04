## ADDED Requirements

### Requirement: Arrastrar y soltar archivos para subir al explorador
El sistema SHALL permitir arrastrar archivos locales desde el host y soltarlos sobre el panel del explorador de archivos SFTP para subirlos al servidor remoto. Al arrastrar archivos sobre el panel del explorador de archivos, el sistema SHALL mostrar un overlay visual con diseño cyberpunk indicando la acción de soltar y el destino actual. Si el cursor se sitúa sobre un nodo de carpeta específico del árbol, el sistema SHALL resaltar visualmente esa carpeta y actualizar el destino en el overlay para reflejar esa subcarpeta. Al soltar los archivos, el sistema SHALL solicitar confirmación al usuario mediante un diálogo A1 indicando el nombre del archivo (o cantidad de archivos) y la ruta remota de destino. Si el usuario confirma, el sistema SHALL subir los archivos correspondientes a la ruta remota y actualizar el listado del explorador.

#### Scenario: Arrastrar sobre el panel general del explorador
- **WHEN** el usuario arrastra archivos sobre el panel del explorador de archivos sin apuntar a una carpeta específica
- **THEN** el sistema muestra el overlay de dropzone con el path actual de navegación (`explorerCwd`) como destino

#### Scenario: Arrastrar apuntando a una carpeta del árbol
- **WHEN** el usuario arrastra archivos apuntando a una carpeta específica del árbol
- **THEN** el sistema resalta esa carpeta y muestra el path de esa carpeta como destino en el overlay de dropzone

#### Scenario: Cancelar subida tras soltar
- **WHEN** el usuario suelta los archivos y cancela el diálogo de confirmación
- **THEN** no se sube ningún archivo y el explorador permanece sin cambios

#### Scenario: Confirmar subida de un archivo
- **WHEN** el usuario suelta un archivo, confirma el diálogo y el upload SFTP es exitoso
- **THEN** el sistema sube el archivo a la ruta de destino remota y refresca el explorador de archivos
