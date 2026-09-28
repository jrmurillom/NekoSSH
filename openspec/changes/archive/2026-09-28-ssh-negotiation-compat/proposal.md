## Why

Al intentar conectar a ciertos servidores empresariales, appliances o paneles de hosting como cPanel/CentOS, la conexión SSH falla inmediatamente durante la fase de handshake con el error `[Session(-5)] Unable to exchange encryption keys`. Esto ocurre porque NekoSSH restringe de forma estricta los métodos de intercambio de claves (KEX) y cifrados (Ciphers) en `libssh2`, excluyendo algoritmos ampliamente usados en entornos empresariales (como Diffie-Hellman Group Exchange y modos de cifrado CBC).

Se requiere modularizar y extender la matriz de negociación criptográfica para ofrecer compatibilidad alineada con herramientas de la industria (como MobaXterm o PuTTY), garantizando que los servidores modernos continúen usando algoritmos de alta seguridad primero, mientras que los servidores legacy o cPanel puedan conectar sin fallos de KEX.

## What Changes

- **Nuevo módulo de negociación SSH (`app/src-tauri/src/ssh_negotiation.rs`)**: Desacoplar y modularizar la definición de algoritmos preferidos (KEX, Ciphers, HostKey y MAC) fuera de la lógica principal de sesión en `lib.rs`.
- **Ampliación de KEX (Key Exchange)**: Incorporar `diffie-hellman-group-exchange-sha256`, `diffie-hellman-group-exchange-sha1` y mantener fallback a `diffie-hellman-group14-sha1` al final de la lista de preferencias.
- **Ampliación de Cifrados (Ciphers)**: Agregar soporte de fallback para modos CBC (`aes256-cbc`, `aes192-cbc`, `aes128-cbc`, `3des-cbc`) después de los modos prioritarios modernos (`aes-ctr`, `chacha20-poly1305`, `aes-gcm`).
- **Configuración de MAC (Integridad)**: Configurar explícitamente métodos de integridad priorizando SHA-2 y permitiendo fallback (`hmac-sha2-256,hmac-sha2-512,hmac-sha1`).
- **Prioridad Estricta de Seguridad**: Los algoritmos modernos (Curve25519, ChaCha20, AES-GCM, AES-CTR) siempre van al inicio de cada lista para que cualquier servidor moderno negocie la máxima seguridad disponible.
- **Suite de Pruebas Unitarias Robustas**: Batería de tests en Rust que verifiquen de forma determinista la precedencia de seguridad, integridad de cadenas, presencia de algoritmos requeridos y ausencia de fallos en llamadas a `libssh2::Session::method_pref`.

## Capabilities

### New Capabilities
<!-- Ninguna capability nueva; la funcionalidad pertenece al motor SSH existente -->

### Modified Capabilities
- `ssh-terminal`: Se amplía el requisito del motor SSH en Rust para exigir una negociación criptográfica modular con soporte de compatibilidad amplia (KEX DH-GEX y Ciphers CBC como fallback ordenado).

## Impact

- **Backend Rust (`app/src-tauri/src/ssh_negotiation.rs`)**: Nuevo archivo modular con la lógica de preferencias de métodos y sus pruebas unitarias asociadas.
- **Backend Rust (`app/src-tauri/src/lib.rs`)**: Reemplazo de las cadenas hardcodeadas por llamadas al módulo `ssh_negotiation`.
- **Dependencias**: Cero dependencias nuevas (utiliza las capacidades nativas ya compiladas en `libssh2`).
- **Base de datos / Frontend**: Cero cambios en SQLite ni en la interfaz gráfica (la negociación ocurre automáticamente por protocolo SSH en el paquete KEXINIT).
