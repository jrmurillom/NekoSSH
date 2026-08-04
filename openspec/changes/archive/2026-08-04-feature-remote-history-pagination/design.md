## Context

Actualmente xterm.js delega el atajo `Ctrl+R` de forma transparente al shell remoto. Sin embargo, esto limita la búsqueda al archivo de historial del shell activo y no provee una interfaz gráfica rica con fechas estructuradas ni paginación controlada. Este diseño introduce una paleta visual cliente-side (`Ctrl+Shift+H`) que consulta dinámicamente y de forma paginada los comandos del archivo de historial en el servidor.

## Goals / Non-Goals

**Goals:**
- Implementar un comando en Rust `sftp_read_remote_history_paged` que ejecute comandos nativos de lectura parcial (`tail` / `head`) vía SSH.
- Crear un modal interactivo con fuzzy search local que reciba la lista de comandos y los muestre en una tabla con fecha legible.
- Integrar autocompletado en el prompt de la terminal al presionar `Enter` y ejecución al presionar `Shift+Enter`.
- Garantizar cero persistencia en local (SQLite/LocalStorage) de los comandos leídos del servidor.

**Non-Goals:**
- No implementar soporte para shells exóticos fuera de Bash y Zsh.
- No guardar historial local en el cliente por razones de seguridad.

## Decisions

### Comando de lectura remota paginada
- **Decisión**: Crear el comando Rust `sftp_read_remote_history_paged(terminal_id, offset, limit)`.
- **Detalle**: En vez de descargar el archivo por SFTP (que podría pesar megabytes), abrimos un canal SSH secundario silencioso para ejecutar:
  ```bash
  tail -n <offset + limit> <history_file> | head -n <limit>
  ```
  Esto devuelve únicamente el lote de 100 líneas deseado.

### Detección automática del archivo de historial
- **Decisión**: El backend detectará el shell del usuario remoto (ej. vía `$SHELL` o probando la existencia de `~/.bash_history` o `~/.zsh_history`).
- **Alternativa considerada**: Exigir al usuario que configure la ruta en el perfil.
- **Razón**: La detección automática brinda una experiencia QoL inmediata "out-of-the-box".

### Parseo de Timestamps
- **Decisión**:
  - Para Zsh: Limpiar el patrón `: <timestamp>:<duration>;<comando>` extrayendo el timestamp Unix para formatear la fecha.
  - Para Bash: Buscar líneas consecutivas que comiencen con `#<timestamp>` si `HISTTIMEFORMAT` está activo.
  - Si no hay timestamp, mostrar `N/D` (No Disponible) en la columna de fecha.

### Atajo de Teclado y UX
- **Decisión**: Mapear `Ctrl+Shift+H` (o un botón de lupa en la UI de terminal) para abrir el modal flotante.
- **Razón**: `Ctrl+H` equivale a Backspace en muchos emuladores de terminal, por lo que usar `Shift` previene colisiones.

### Evitar Cierre e Inyección por Clic en Fila (QoL)
- **Decisión**: El clic simple sobre una fila del historial en el modal **solo seleccionará** la fila para su lectura, sin cerrar el modal ni enviar el texto a la terminal ("no baja con un click").
- **Detalle**: Cada fila de la tabla del historial contará con un botón de acción final con un icono de terminal (`AppIcons.terminal`, análogo al botón de copiar en los snippets). El usuario debe hacer clic explícitamente en ese botón para inyectar el comando en la terminal. El teclado (`Enter` sobre fila seleccionada) mantiene el comportamiento rápido de inyección y cierre.


### Corrección de Ruta (Fix)
- **Decisión**: Para respetar la consistencia estética cyberpunk-sakura y evitar la invención de clases CSS redundantes, se descartan los nuevos estilos específicos `.history-modal-content`, `.history-table`, etc. En su lugar, el modal del historial reutilizará la misma estructura, contenedor de barra de búsqueda y clases visuales del modal de snippets (`.snippets-modal-content`, `.snippets-toolbar`, etc.).
- **Botón e Icono**: Se descarta el uso del icono custom `AppIcons.terminal`. Para realizar la inyección de comandos, el modal de historial remoto reutilizará el mismo botón con el icono de copiar (`AppIcons.copy`) estilizado idénticamente a como está en la lista de snippets.
- **Buscador**: El modal de historial remoto reutilizará el mismo estilo de barra de búsqueda y el input con las clases de snippets. Se compartirán explícitamente las reglas de CSS de `#snippets-search` con `#history-search` (incluyendo los estados de focus y placeholder) para asegurar que se muestren e interactúen de forma idéntica.
- **Acción de Copiado (Clipboard)**: Para que el botón de copiar actúe de acuerdo a su nombre e icono, al hacer clic en él se SHALL copiar el comando al portapapeles de la máquina local (`navigator.clipboard.writeText`) además de inyectarse en la terminal activa.
- **Parser de Historial de Doble Estado (Bash y Zsh)**: Se implementará un parser en un módulo independiente (`remote-history-helper.ts`) que procese las líneas de historial de forma secuencial y stateful. Debe soportar:
  - Formato Zsh extendido (`: timestamp:duration;comando`).
  - Formato Bash extendido (línea de timestamp `#<unix_timestamp>` seguida por la línea del comando en la siguiente iteración).
- **Pruebas Unitarias**: Para asegurar la estabilidad y corrección del parser frente a diferentes formatos de historial, se implementará un archivo de pruebas unitarias (`remote-history-helper.test.ts`) con Vitest para validar todos los escenarios posibles (Bash, Zsh y comandos planos sin fecha).
- **Prueba del Portapapeles (Clipboard Unit Test)**: Se extraerá la acción de copiado a una función testeable `copyCommandToClipboard` en el helper, mockeando la API de portapapeles de Tauri (`@tauri-apps/plugin-clipboard-manager`) en los tests de Vitest para validar técnicamente que el botón copia el comando correcto en tiempo real.

## Risks / Trade-offs

- **Riesgo**: El archivo de historial remoto podría no tener permisos de lectura o estar vacío.
  - **Mitigación**: Si el comando falla, el modal mostrará un estado de error claro indicando que no se pudo leer el historial del servidor, sugiriendo verificar los permisos del archivo `~/.bash_history` o `~/.zsh_history`.
- **Riesgo**: Inyectar comandos en la terminal podría interferir si hay un proceso interactivo corriendo (como `vim` o `nano`).
  - **Mitigación**: El modal se abrirá y funcionará, pero el autocompletado simplemente escribirá los caracteres en el buffer activo de la terminal. Añadiremos un aviso visual indicando que se inyectará en la sesión activa.
