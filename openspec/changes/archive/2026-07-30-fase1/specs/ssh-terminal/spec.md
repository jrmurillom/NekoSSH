## ADDED Requirements

### Requirement: Emulación de Terminal Visual Cyber-Sakura
El sistema SHALL inicializar una terminal interactiva utilizando `xterm.js` y `xterm-addon-fit` con el estilo visual Cyber-Sakura (fondo translúcido oscuro, cursor block de neón sakura parpadeante con resplandor glow).

#### Scenario: Visualizar y autoajustar la terminal al redimensionar
- **WHEN** el usuario redimensiona la ventana principal del cliente NekoSSH
- **THEN** el sistema calcula las nuevas dimensiones y ajusta dinámicamente las filas y columnas de xterm.js de forma proporcional

### Requirement: Motor SSH en Rust y Flujo de Entrada/Salida
El sistema SHALL establecer una conexión SSH interactiva en el backend de Rust (usando el crate `ssh2` o `russh`) a partir del perfil seleccionado y mantener un canal bidireccional asíncrono para enviar las pulsaciones de teclado del usuario y recibir la salida del shell remoto.

#### Scenario: Conectar y emitir comandos con éxito
- **WHEN** el usuario selecciona un perfil en la barra lateral y hace clic en Conectar
- **THEN** el backend de Rust valida las credenciales, inicia la sesión interactiva, abre un canal PTY y transmite la respuesta del shell a xterm.js en el frontend
