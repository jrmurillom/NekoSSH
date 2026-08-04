## ADDED Requirements

### Requirement: Consulta y paginación del historial remoto
El sistema SHALL permitir al usuario consultar el historial de comandos ejecutados en el servidor remoto de forma paginada (en lotes de 100 comandos) directamente desde su archivo de historial nativo (`~/.bash_history`, `~/.zsh_history`, etc.) sin almacenar dichos comandos en base de datos local en el cliente.
- El sistema SHALL abrir un canal SSH secundario para ejecutar comandos silenciosos de `tail` y `head` sobre el archivo de historial.
- El sistema SHALL soportar retroceder o avanzar en los lotes de historial mediante offset dinámico.
- Si el archivo de historial contiene timestamps estructurados (Zsh/Bash extendido), el sistema SHALL extraer y mostrar la fecha y hora legible. En caso contrario, SHALL mostrar un indicador de "No disponible".

#### Scenario: Cargar primera página de historial
- **WHEN** el usuario abre la paleta de historial remoto en una sesión activa
- **THEN** el sistema ejecuta silenciosamente `tail -n 100` sobre el archivo de historial remoto y muestra los comandos en la interfaz

#### Scenario: Cargar página previa de historial
- **WHEN** el usuario navega a la página anterior (más antiguos) desde el modal
- **THEN** el sistema ejecuta `tail -n <offset_acumulado> | head -n 100` para traer el lote correspondiente de comandos antiguos y refresca la lista

#### Scenario: Parseo de timestamps con fecha legible
- **WHEN** se leen comandos de un historial de Zsh con timestamps (ej. `: 1627999999:0;ls`)
- **THEN** el sistema remueve los metadatos y muestra la fecha formateada en la columna de fecha de uso

### Requirement: Interfaz interactiva de Paleta de Historial (Fuzzy Search & Inyección)
El sistema SHALL mostrar un modal flotante HUD cyberpunk para buscar y filtrar en tiempo real los comandos obtenidos.
- El modal SHALL soportar fuzzy search local sobre la página de historial cargada.
- Cada fila del historial en el modal SHALL contar con un botón o icono de inyección independiente (con el icono `AppIcons.terminal` o similar).
- El sistema SHALL requerir un clic en el botón de inyección de la fila para enviar el comando a la terminal; hacer clic en cualquier otra parte de la fila SHALL únicamente seleccionarla o resaltarla visualmente, sin cerrar el modal ni pegar el comando.
- Al presionar `Enter` con el teclado sobre un comando seleccionado, el sistema SHALL inyectar el texto del comando en el prompt de la terminal activa y cerrar el modal.
- Al presionar `Shift+Enter` con el teclado, el sistema SHALL inyectar el comando, ejecutarlo inmediatamente en la terminal activa y cerrar el modal.

#### Scenario: Inyectar comando con el teclado para edición
- **WHEN** el usuario tiene seleccionado un comando en el modal mediante teclado y presiona `Enter`
- **THEN** el modal se cierra y el texto del comando aparece en la terminal activa manteniendo el cursor al final de la línea listo para ejecutarse

#### Scenario: Ejecución directa desde teclado con Shift+Enter
- **WHEN** el usuario tiene seleccionado un comando en el modal mediante teclado y presiona `Shift+Enter`
- **THEN** el modal se cierra, el comando se envía a la terminal con un retorno de carro (`\r`) y se ejecuta inmediatamente en el servidor

#### Scenario: Clic en la fila del historial sin inyectar
- **WHEN** el usuario hace un clic simple sobre la fila de un comando en el historial
- **THEN** la fila se resalta visualmente como seleccionada, pero el modal no se cierra y no se envía ningún comando a la terminal

#### Scenario: Clic en el botón de inyección
- **WHEN** el usuario hace clic en el botón de icono de terminal de una fila del historial
- **THEN** el modal se cierra y el texto del comando se inyecta en el prompt de la terminal activa sin ejecutarlo
