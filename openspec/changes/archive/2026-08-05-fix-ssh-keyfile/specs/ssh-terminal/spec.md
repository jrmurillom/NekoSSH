### Requirement: Motor SSH en Rust y Flujo de Entrada/Salida
El sistema SHALL establecer una conexión SSH interactiva en el backend de Rust (usando el crate `ssh2`) a partir del perfil seleccionado y mantener un canal bidireccional asíncrono para enviar las pulsaciones de teclado del usuario y recibir la salida del shell remoto. Esa Session del PTY SHALL usarse para la terminal interactiva (y, si aplica, canales auxiliares como SFTP en la misma Session, sin un segundo login TCP). El sistema SHALL aplicar un cambio de directorio solicitado por el explorador (`cd` al PTY). El producto NO requiere sync automático del explorador tras `cd` tipado en la terminal. Además, el backend MUST validar la existencia del archivo de llave privada antes de intentar la conexión SSH. Para la autenticación con llave pública, el backend MUST usar autenticación basada en memoria (`userauth_pubkey_memory`) leyendo los contenidos de la llave desde el sistema de archivos, utilizando el archivo de llave pública (`.pub`) si está presente.

#### Scenario: Conectar y emitir comandos con éxito
- **WHEN** el usuario selecciona un perfil en la barra lateral y hace clic en Conectar
- **THEN** el backend de Rust valida las credenciales, inicia la sesión interactiva, abre un canal PTY y transmite la respuesta del shell a xterm.js en el frontend

#### Scenario: Aplicar cd desde el explorador
- **WHEN** el frontend solicita cambiar al path remoto P en la terminal activa
- **THEN** el sistema envía al PTY un `cd` válido hacia P (con escaping adecuado)

#### Scenario: Escribir en la terminal no cierra la sesión
- **WHEN** el usuario escribe caracteres en el PTY con sesión activa
- **THEN** la conexión permanece abierta y los caracteres llegan al shell remoto

#### Scenario: Conectar con un archivo de llave privada inexistente
- **WHEN** el backend de Rust intenta conectar a un perfil donde el archivo de llave privada no existe en el sistema de archivos
- **THEN** la conexión MUST ser abortada y retornar un error claro indicando: "El archivo de llave privada no existe..."

#### Scenario: Autenticar exitosamente usando la carga de llave en memoria
- **WHEN** el backend de Rust lee el archivo de llave privada (y su respectiva llave pública si existe)
- **THEN** la sesión SSH MUST autenticar al usuario exitosamente utilizando la función `userauth_pubkey_memory`
