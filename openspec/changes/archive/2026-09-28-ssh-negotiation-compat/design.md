## Context

Actualmente, en `app/src-tauri/src/lib.rs`, dentro de la función `authenticate_session_once`, las preferencias de métodos criptográficos para `libssh2` se configuran mediante cadenas fijas hardcodeadas:

```rust
let _ = sess.method_pref(MethodType::Kex, "curve25519-sha256,...");
let _ = sess.method_pref(MethodType::CryptCs, "aes256-ctr,...");
let _ = sess.method_pref(MethodType::CryptSc, "aes256-ctr,...");
let _ = sess.method_pref(MethodType::HostKey, "ssh-ed25519,...");
```

Esta implementación presenta tres limitaciones graves:
1. **Falta de modularidad**: La lógica de negociación criptográfica está acoplada dentro de la función de conexión TCP/PTY, impidiendo pruebas unitarias aisladas sin levantar sockets.
2. **Exclusión de algoritmos comunes**: Excluye los métodos de intercambio Diffie-Hellman Group Exchange (`diffie-hellman-group-exchange-sha256`, `diffie-hellman-group-exchange-sha1`) y modos de cifrado CBC (`aes256-cbc`, `aes128-cbc`, `3des-cbc`), provocando que servidores ampliamente extendidos (como CentOS, RedHat Enterprise, appliances y paneles cPanel) fallen durante el handshake con `[Session(-5)] Unable to exchange encryption keys`.
3. **Omisión de MAC**: No se configuran explícitamente métodos de integridad (MAC), dejando el comportamiento sujeto a defaults de la plataforma.

## Goals / Non-Goals

**Goals:**
- Modularizar la arquitectura de negociación SSH en un nuevo módulo `app/src-tauri/src/ssh_negotiation.rs`.
- Proporcionar una matriz de negociación completa y compatible con herramientas líderes como MobaXterm y PuTTY.
- Mantener una estricta jerarquía de seguridad: los algoritmos modernos y robustos siempre van al inicio; los algoritmos legacy solo actúan como fallback al final de la lista.
- Diseñar una suite de pruebas unitarias robustas, deterministas y con **tolerancia cero a fallos** que certifiquen el orden de precedencia y la presencia de algoritmos requeridos.

**Non-Goals:**
- No se agregarán librerías externas en Rust ni en Node/Tauri (`libssh2` ya incluye estos algoritmos internamente).
- No se guardará el algoritmo negociado en la base de datos SQLite (el protocolo SSH RFC 4253 exige el paquete `SSH_MSG_KEXINIT` en cada sesión; guardar el algoritmo en SQLite agregaría deuda técnica y riesgo de desincronización).
- No se alterará la interfaz visual ni el esquema de perfiles de conexión.

## Decisions

### 1. Creación del módulo desacoplado `ssh_negotiation.rs`
* **Decisión:** Extraer toda la configuración criptográfica a `app/src-tauri/src/ssh_negotiation.rs`.
* **Racional:** Permite evaluar y certificar las listas de algoritmos con pruebas unitarias en memoria, sin depender de conexiones de red activas ni mocks complejos.
* **Alternativa descartada:** Mantener las cadenas en `lib.rs` (rechazada: viola el principio de modularidad y dificulta la verificación formal).

### 2. Estrategia de Lista Priorizada Transparente (Estilo MobaXterm)
* **Decisión:** Enviar una única lista por cada `MethodType` que priorice algoritmos modernos y concluya con algoritmos de compatibilidad legacy:
  - **KEX:** `curve25519-sha256`, `curve25519-sha256@libssh.org`, `ecdh-sha2-nistp256`, `ecdh-sha2-nistp384`, `ecdh-sha2-nistp521`, `diffie-hellman-group-exchange-sha256`, `diffie-hellman-group16-sha512`, `diffie-hellman-group14-sha256`, `diffie-hellman-group14-sha1`, `diffie-hellman-group-exchange-sha1`.
  - **Cifrado (Ciphers):** `aes256-gcm@openssh.com`, `chacha20-poly1305@openssh.com`, `aes256-ctr`, `aes192-ctr`, `aes128-ctr`, `aes128-gcm@openssh.com`, `aes256-cbc`, `aes192-cbc`, `aes128-cbc`, `3des-cbc`.
  - **HostKey:** `ssh-ed25519`, `ecdsa-sha2-nistp256`, `ecdsa-sha2-nistp384`, `ecdsa-sha2-nistp521`, `rsa-sha2-512`, `rsa-sha2-256`, `ssh-rsa`.
  - **MAC:** `hmac-sha2-256-etm@openssh.com`, `hmac-sha2-512-etm@openssh.com`, `hmac-sha2-256`, `hmac-sha2-512`, `hmac-sha1`.
* **Racional:** El protocolo SSH selecciona el primer algoritmo de la lista del cliente que también soporte el servidor. Un servidor moderno siempre seleccionará Curve25519 + AES-GCM/CTR; un servidor cPanel o CentOS antiguo tomará DH-GEX o AES-CBC. Todo en un solo paquete inicial de <0.02ms.

## Risks / Trade-offs

- **[Riesgo] Cifrados CBC con vulnerabilidades conocidas (ej. Plaintext recovery en redes hostiles)**
  - *Mitigación:* Los modos CBC están estrictamente relegados al final de la lista. Solo se seleccionan si el servidor remoto explícitamente no ofrece ninguna alternativa CTR, GCM o ChaCha20.
- **[Riesgo] Diffie-Hellman Group Exchange en Windows/WinCNG**
  - *Mitigación:* Se priorizan `curve25519` y `diffie-hellman-group14/16` antes de `group-exchange`. Si el servidor remoto soporta Group 14/16 (el estándar moderno), se usará ese antes de caer en Group Exchange.

## Migration Plan

- No requiere migración de datos ni cambios en esquema de base de datos.
- Despliegue directo al compilar el backend de Tauri.
