# ssh-terminal

## Purpose

Emulación de terminal interactiva con xterm.js, estilo Cyber-Sakura (acentos planos) y sesión SSH bidireccional vía backend Rust.

## Requirements

### Requirement: Emulación de Terminal Visual Cyber-Sakura
El sistema SHALL inicializar una terminal interactiva utilizando `xterm.js` y `xterm-addon-fit` con el estilo visual Cyber-Sakura (fondo oscuro opaco en el viewport de terminal, cursor block sakura parpadeante con acento de color plano, sin neon glow).

#### Scenario: Visualizar y autoajustar la terminal al redimensionar
- **WHEN** el usuario redimensiona la ventana principal del cliente NekoSSH
- **THEN** el sistema calcula las nuevas dimensiones y ajusta dinámicamente las filas y columnas de xterm.js de forma proporcional

### Requirement: Motor SSH en Rust y Flujo de Entrada/Salida
El sistema SHALL establecer una conexión SSH interactiva en el backend de Rust (usando el crate `ssh2`) a partir del perfil seleccionado y mantener un canal bidireccional asíncrono para enviar las pulsaciones de teclado del usuario y recibir la salida del shell remoto. Esa Session del PTY SHALL usarse para la terminal interactiva (y, si aplica, canales auxiliares como SFTP en la misma Session, sin un segundo login TCP). El sistema SHALL aplicar un cambio de directorio solicitado por el explorador (`cd` al PTY). El producto NO requiere sync automático del explorador tras `cd` tipado en la terminal.

#### Scenario: Conectar y emitir comandos con éxito
- **WHEN** el usuario selecciona un perfil en la barra lateral y hace clic en Conectar
- **THEN** el backend de Rust valida las credenciales, inicia la sesión interactiva, abre un canal PTY y transmite la respuesta del shell a xterm.js en el frontend

#### Scenario: Aplicar cd desde el explorador
- **WHEN** el frontend solicita cambiar al path remoto P en la terminal activa
- **THEN** el sistema envía al PTY un `cd` válido hacia P (con escaping adecuado)

#### Scenario: Escribir en la terminal no cierra la sesión
- **WHEN** el usuario escribe caracteres en el PTY con sesión activa
- **THEN** la conexión permanece abierta y los caracteres llegan al shell remoto

### Requirement: Cierre de conexiones con pestaña y aplicación
El sistema SHALL liberar de forma ordenada la Session SSH asociada a un `terminal_id` cuando el usuario cierra esa pestaña de terminal, y SHALL liberar todas las Sessions SSH activas cuando se cierra la ventana o se sale de la aplicación. Al intentar cerrar una pestaña con sesión viva (`isConnected === true`), el sistema MUST solicitar confirmación mediante un diálogo de confirmación antes de proceder a la desconexión. Al ejecutar "Cerrar Todo", si existen conexiones vivas, el sistema MUST solicitar confirmación global una sola vez antes de cerrar todas las sesiones.

#### Scenario: Confirmación al cerrar pestaña individual con sesión viva
- **WHEN** el usuario hace clic en el botón de cerrar (`x`) en una pestaña de terminal que tiene una conexión SSH activa (`isConnected === true`)
- **THEN** el sistema despliega el diálogo de confirmación glass. Si el usuario confirma, la sesión se libera y la pestaña se remueve; si cancela, la terminal permanece abierta

#### Scenario: Cerrar pestaña desconectada sin confirmación
- **WHEN** el usuario hace clic en el botón de cerrar (`x`) en una pestaña que ya está desconectada (`isConnected === false`)
- **THEN** la pestaña se cierra de inmediato sin solicitar confirmación

#### Scenario: Cerrar la aplicación con sesiones activas
- **WHEN** el usuario cierra la ventana de NekoSSH con una o más terminales conectadas
- **THEN** el sistema cierra todas las Sessions SSH activas antes de terminar el proceso

#### Scenario: Confirmación al cerrar todas las terminales
- **WHEN** el usuario activa la acción "Cerrar Todo" teniendo una o más terminales activas conectadas
- **THEN** el sistema presenta un único diálogo de confirmación global antes de cerrar todas las pestañas

### Requirement: Aviso de sesión SSH desconectada
Cuando la Session SSH asociada a una pestaña de terminal termina de forma no solicitada por el usuario (EOF, error de transporte, cierre remoto), el sistema SHALL mostrar en el viewport de esa terminal un mensaje claro de desconexión e indicar que puede reconectar con **Ctrl+R**. El indicador de estado de la pestaña (dot + texto) SHALL pasar a un estado de desconectado (o error, si el cierre fue por fallo de conexión).

#### Scenario: Sesión muerta por el servidor
- **WHEN** el backend emite el evento de cierre/error para un `terminal_id` que estaba conectado
- **THEN** la terminal muestra un aviso de desconexión que incluye la pista de Ctrl+R, marca `isConnected` en falso y actualiza el indicador a desconectado (o error)

#### Scenario: Cierre voluntario de pestaña
- **WHEN** el usuario cierra la pestaña de terminal
- **THEN** el sistema no presenta el flujo de “Ctrl+R para reconectar” como acción disponible (la pestaña deja de existir)

### Requirement: Reconexión manual con Ctrl+R
El sistema SHALL permitir reconectar la misma pestaña de terminal con el mismo perfil de conexión cuando el usuario pulsa **Ctrl+R** y esa pestaña está desconectada. Mientras la sesión esté activa, Ctrl+R NO MUST interceptarse como reconexión (debe poder llegar al shell remoto). La reconexión SHALL reutilizar el `terminal_id` de la pestaña y el perfil asociado a ella. El sistema SHALL evitar lanzar dos reconexiones concurrentes sobre la misma pestaña.

#### Scenario: Reconectar tras desconexión
- **WHEN** la pestaña está desconectada y el usuario pulsa Ctrl+R
- **THEN** el sistema inicia de nuevo la sesión SSH con el perfil de esa pestaña, actualiza el indicador a conectando y, si tiene éxito, a conectado

#### Scenario: Ctrl+R con sesión viva
- **WHEN** la pestaña está conectada y el usuario pulsa Ctrl+R
- **THEN** el cliente no dispara el flujo de reconexión NekoSSH (la pulsación no se consume como reconnect)

#### Scenario: Indicador durante reconexión
- **WHEN** el usuario inicia reconexión con Ctrl+R
- **THEN** el indicador bajo la pestaña refleja estado de conectando hasta el resultado (conectado o error)

### Requirement: Auto-copiar selección en la terminal
Cuando el usuario selecciona texto en el viewport xterm de una sesión, el sistema SHALL copiar automáticamente ese texto al clipboard del sistema vía la API nativa de clipboard de Tauri (plugin clipboard-manager). Una selección vacía NO MUST escribir al clipboard. Ctrl+C NO MUST redefinirse como copiar: MUST seguir disponible para el shell remoto (interrupt). El producto NO MUST depender del prompt de permiso de `navigator.clipboard` del WebView para este gesto.

#### Scenario: Copiar al soltar selección
- **WHEN** el usuario selecciona uno o más caracteres visibles en el emulador xterm
- **THEN** el texto seleccionado queda en el clipboard del sistema

#### Scenario: Selección vacía
- **WHEN** no hay texto seleccionado en el emulador
- **THEN** el sistema no sobrescribe el clipboard por este gesto

#### Scenario: Ctrl+C no es copy de producto
- **WHEN** el usuario pulsa Ctrl+C con sesión SSH activa
- **THEN** el cliente no consume la pulsación como “copiar selección”; el comportamiento hacia el remoto permanece el de interrupt/input normal

#### Scenario: Sin prompt de permiso WebView al copiar
- **WHEN** el usuario selecciona texto en la terminal dentro de la app Tauri
- **THEN** la copia al clipboard del SO ocurre sin diálogo de permiso del WebView por `navigator.clipboard`

### Requirement: Pegar con clic derecho y strip del final
El clic derecho sobre el viewport xterm de una terminal SHALL pegar el contenido del clipboard hacia el PTY (mismo canal de entrada que el teclado), leyendo el clipboard con la API nativa de Tauri (plugin clipboard-manager). Antes de enviar, el sistema MUST eliminar únicamente saltos de línea y/o whitespace **al final** del texto pegado (p. ej. `\n`, `\r`, `\r\n`, espacios/tabs finales). Los saltos de línea en medio del texto MUST conservarse. Este gesto NO MUST abrir un menú contextual en la terminal. El producto NO MUST depender del prompt de permiso de `navigator.clipboard` del WebView para este gesto.

#### Scenario: Pegar con clic derecho
- **WHEN** el usuario hace clic derecho en el canvas de la terminal con texto en el clipboard
- **THEN** ese texto (tras el strip del final) se envía al PTY como input

#### Scenario: Quitar Enter solo al final
- **WHEN** el clipboard contiene `ls -la` seguido de un salto de línea final
- **THEN** al pegar con clic derecho se envía `ls -la` sin el salto final

#### Scenario: Multilínea interna intacta
- **WHEN** el clipboard contiene varias líneas con Enter entre ellas y opcionalmente Enter al final
- **THEN** los Enter entre líneas se conservan y solo se elimina el trailing del final del texto completo

#### Scenario: Sin menú contextual en terminal
- **WHEN** el usuario hace clic derecho en el viewport xterm
- **THEN** no aparece un menú contextual de chrome; se realiza el paste (o nada si el clipboard no se puede leer)

#### Scenario: Sin prompt de permiso WebView al pegar
- **WHEN** el usuario hace clic derecho para pegar en la terminal dentro de la app Tauri
- **THEN** la lectura del clipboard del SO ocurre sin diálogo de permiso del WebView por `navigator.clipboard`
