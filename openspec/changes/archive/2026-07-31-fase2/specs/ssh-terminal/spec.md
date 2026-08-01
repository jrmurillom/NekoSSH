## MODIFIED Requirements

### Requirement: Motor SSH en Rust y Flujo de Entrada/Salida
El sistema SHALL establecer una conexión SSH interactiva en el backend de Rust (usando el crate `ssh2`) a partir del perfil seleccionado y mantener un canal bidireccional asíncrono para enviar las pulsaciones de teclado del usuario y recibir la salida del shell remoto. Esa Session del PTY SHALL usarse solo para la terminal interactiva. El sistema SHALL aplicar un cambio de directorio solicitado por el explorador (`cd` al PTY). El producto NO requiere sync automático del explorador tras `cd` tipado en la terminal.

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
El sistema SHALL liberar de forma ordenada la Session SSH asociada a un `terminal_id` cuando el usuario cierra esa pestaña de terminal, y SHALL liberar todas las Sessions SSH activas cuando se cierra la ventana o se sale de la aplicación. El cierre SHALL ser idempotente (cerrar dos veces no falla de forma ruidosa) y no SHALL dejar entradas huérfanas en el estado de conexiones del backend.

#### Scenario: Cerrar pestaña de terminal
- **WHEN** el usuario cierra la pestaña de una terminal conectada
- **THEN** el backend elimina esa sesión del estado, cierra el canal PTY y desconecta la Session SSH de ese `terminal_id`

#### Scenario: Cerrar la aplicación con sesiones activas
- **WHEN** el usuario cierra la ventana de NekoSSH con una o más terminales conectadas
- **THEN** el sistema cierra todas las Sessions SSH activas antes de terminar el proceso

#### Scenario: Cerrar todas las terminales
- **WHEN** el usuario elige cerrar todas las terminales
- **THEN** cada `terminal_id` activo recibe el mismo cleanup que al cerrar una pestaña individual
