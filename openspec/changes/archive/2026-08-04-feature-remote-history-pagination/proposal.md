## Why

Los usuarios necesitan una forma rápida de buscar y reutilizar comandos ejecutados anteriormente en sus servidores remotos sin la necesidad de almacenar localmente en el cliente datos sensibles de historial. El atajo nativo `Ctrl+R` es muy limitado y texto plano en consola, por lo que una paleta visual tipo modal con búsqueda interactiva (fuzzy search), paginación bajo demanda y extracción de fecha de uso (timestamp) mejora significativamente la productividad y la calidad de vida de los administradores de sistemas.

## What Changes

- Crear un atajo de teclado global en el terminal (`Ctrl+Shift+H` / `Ctrl+Alt+H` o similar) para abrir la paleta de historial remoto.
- Implementar un modal flotante con estética cyberpunk-sakura para la búsqueda interactiva con fuzzy filtering.
- Incorporar un botón/icono de inyección al lado de cada comando (similar al botón de copiar de los snippets) para enviarlo a la terminal, evitando que el modal se cierre o envíe el comando con un simple clic sobre la fila del historial ("no baja con un click").
- Añadir paginación dinámica sin estado que lea por lotes de 100 líneas directamente desde el servidor remoto usando comandos de consola optimizados (`tail` y `head`).
- Extraer y procesar los timestamps de los archivos de historial nativos de Linux (Bash/Zsh) para mostrar la columna "Fecha de uso" si está disponible.
- Permitir inyectar/autocompletar el comando seleccionado en la línea del terminal activo cuando se presiona `Enter`, y ejecutarlo inmediatamente al presionar `Shift+Enter`.

## Capabilities

### New Capabilities
- `remote-history`: Capacidad de consultar, paginar y buscar el historial de comandos directamente desde el archivo de historial de la shell remota activa sin almacenamiento local persistente en el cliente.

### Modified Capabilities
_(ninguna)_

## Impact

- **Frontend**: `main.ts`, `styles.css` (para agregar el atajo, el componente visual del modal flotante, y las llamadas para ejecutar la lectura remota).
- **Backend Rust**: `lib.rs` y un nuevo comando en Tauri (`sftp_read_remote_history_paged`) para abrir canales secundarios silenciosos que ejecuten `tail` y `head` sobre los archivos de historial del usuario remoto.
