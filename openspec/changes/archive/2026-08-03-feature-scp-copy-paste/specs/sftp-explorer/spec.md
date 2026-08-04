## ADDED Requirements

### Requirement: Portapapeles inter-sesión Copiar SCP y Pegar SCP
El sistema SHALL permitir al usuario copiar un archivo o carpeta desde el explorador SFTP de una sesión activa (origen) y pegarlo en el explorador SFTP de otra sesión activa (destino) mediante las opciones de menú contextual denominadas exactamente "copiar scp" y "pegar scp".
- Al elegir "copiar scp" sobre un archivo, el sistema SHALL almacenar en el portapapeles de la aplicación el ID de la terminal origen y la ruta remota absoluta.
- La opción "pegar scp" en el menú contextual del árbol o fondo del explorador SHALL habilitarse únicamente si hay un elemento almacenado en el portapapeles y existe una sesión activa como destino.
- Al activar "pegar scp", el sistema SHALL mostrar un diálogo A1 solicitando confirmación con el nombre del archivo y las rutas absolutas de origen y destino.
- Si el usuario confirma, el sistema SHALL transferir los datos en streaming a través de la memoria local y refrescar la ubicación destino.

#### Scenario: Copiar SCP de un archivo
- **WHEN** el usuario selecciona "copiar scp" en el menú contextual de un archivo
- **THEN** el sistema registra el archivo y el ID de terminal origen en memoria y la opción "pegar scp" queda disponible para usarse

#### Scenario: Pegar SCP con confirmación
- **WHEN** el usuario hace clic derecho en el explorador destino, selecciona "pegar scp" y confirma en el diálogo A1
- **THEN** el sistema inicia el streaming directo de datos, crea el archivo en el servidor destino, y refresca la ubicación del explorador al completar con éxito

#### Scenario: Cancelar Pegar SCP
- **WHEN** el usuario selecciona "pegar scp" y presiona cancelar o escape en el diálogo de confirmación
- **THEN** la transferencia se aborta y no se crea ni modifica ningún archivo en el destino
