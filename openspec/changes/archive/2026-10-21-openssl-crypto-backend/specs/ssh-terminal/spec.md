# ssh-terminal

## MODIFIED Requirements

### Requirement: Motor SSH en Rust y Flujo de Entrada/Salida
El sistema SHALL establecer una conexión SSH interactiva en el backend de Rust (usando el crate `ssh2` compilado con soporte nativo de Curvas Elípticas WinCNG `LIBSSH2_ECDSA_WINCNG` en Windows) a partir del perfil seleccionado y mantener un canal bidireccional asíncrono para enviar las pulsaciones de teclado del usuario y recibir la salida del shell remoto. Esa Session del PTY SHALL usarse para la terminal interactiva (y, si aplica, canales auxiliares como SFTP en la misma Session, sin un segundo login TCP). El sistema SHALL aplicar un cambio de directorio solicitado por el explorador (`cd` al PTY). El producto NO requiere sync automático del explorador tras `cd` tipado en la terminal. Cuando el perfil use autenticación por llave, el backend MUST autenticar usando el contenido de `private_key` del perfil (y passphrase si aplica): MUST materializar ese contenido solo en un archivo temporal efímero para la llamada a libssh2 y MUST eliminarlo tras el intento, sin depender de una ruta de llave gestionada por el usuario. Durante la fase de inicialización y handshake SSH, el motor MUST configurar una negociación criptográfica modular que soporte nativamente Curvas Elípticas (`ecdh-sha2-*`), Diffie-Hellman Group 14/16/GEX, Ciphers modernos (AES-GCM, ChaCha20, AES-CTR) y fallback CBC, garantizando compatibilidad universal con servidores ultra-modernos (OpenSSH 10 / Debian 13) y legacy sin fallos prematuros de intercambio de claves (`Session(-5)`).

#### Scenario: Conectar y emitir comandos con éxito
- **WHEN** el usuario selecciona un perfil en la barra lateral y hace clic en Conectar
- **THEN** el backend de Rust valida las credenciales, inicia la sesión interactiva, abre un canal PTY y transmite la respuesta del shell a xterm.js en el frontend

#### Scenario: Aplicar cd desde el explorador
- **WHEN** el frontend solicita cambiar al path remoto P en la terminal activa
- **THEN** el sistema envía al PTY un `cd` válido hacia P (con escaping adecuado)

#### Scenario: Escribir en la terminal no cierra la sesión
- **WHEN** el usuario escribe caracteres en el PTY con sesión activa
- **THEN** la conexión permanece abierta y los caracteres llegan al shell remoto

#### Scenario: Autenticar con llave desde material persistido
- **WHEN** el usuario conecta un perfil con auth por llave y `private_key` poblado
- **THEN** el backend autentica la sesión usando ese contenido (vía archivo temporal efímero) y propaga un error descriptivo si la autenticación falla

#### Scenario: Rechazar conexión sin material de llave
- **WHEN** el perfil indica auth por llave pero `private_key` está vacío o ausente
- **THEN** la conexión MUST abortarse con un error claro indicando que falta la llave privada configurada

#### Scenario: Negociación exitosa con servidor moderno
- **WHEN** el cliente conecta a un servidor SSH moderno con soporte de algoritmos elípticos
- **THEN** el motor negocia prioritariamente Curvas Elípticas (`ecdh-sha2-*`) y AES-CTR/ChaCha20 sin degradar la seguridad

#### Scenario: Negociación exitosa con servidor empresarial o cPanel
- **WHEN** el cliente conecta a un servidor con configuración restringida a Diffie-Hellman Group Exchange o cifrados CBC
- **THEN** el handshake SSH completa con éxito utilizando los algoritmos de compatibilidad ordenados al final de la lista, sin arrojar error `Session(-5)`

#### Scenario: Negociación exitosa con servidor OpenSSH 10 con Curvas Elípticas obligatorias
- **WHEN** el cliente conecta a un servidor moderno (ej. OpenSSH 10 en Debian 13 en `54.177.128.80`) que deshabilitó Diffie-Hellman finito
- **THEN** el backend de libssh2 negocia exitosamente ECDH (`ecdh-sha2-nistp256`) sin arrojar `Session(-5)` y completa la autenticación
