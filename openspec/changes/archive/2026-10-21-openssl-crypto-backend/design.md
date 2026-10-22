## Context

NekoSSH utiliza el crate de Rust `ssh2` (v0.9.6), que a su vez depende de `libssh2-sys`. En plataformas Windows, `libssh2-sys` se compila por defecto vinculándose a la API criptográfica nativa del sistema operativo (**WinCNG**).

A través de un análisis de tráfico SSH a nivel de paquetes en este entorno, se demostró empíricamente que:
1. La implementación por defecto de WinCNG en `libssh2` carece de Curvas Elípticas activas si no se define explícitamente el flag de compilación `LIBSSH2_ECDSA_WINCNG`. En consecuencia, el cliente NekoSSH en Windows sólo emitía métodos Diffie-Hellman finitos tradicionales (`diffie-hellman-group*`).
2. Los servidores Linux modernos (como instancias AWS EC2 con OpenSSH 10.0 en Debian 13 o Ubuntu 24.04) han deshabilitado Diffie-Hellman finito por motivos de seguridad y requieren estrictamente Curvas Elípticas o algoritmos post-cuánticos.
3. Al conectar a estos servidores, la intersección de métodos KEX entre cliente y servidor era nula (0 algoritmos coincidentes), resultando invariablemente en `[Session(-5)] Unable to exchange encryption keys`.

## Goals / Non-Goals

**Goals:**
- Dotar al binario de NekoSSH de soporte nativo para Curvas Elípticas (`ecdh-sha2-nistp256`, `ecdh-sha2-nistp384`, `ecdh-sha2-nistp521`) en Windows sin dependencias externas pesadas.
- Mantener la compatibilidad existente con servidores cPanel/CentOS lograda en `ssh_negotiation.rs`.
- Probar que el handshake con servidores modernos que exigen Curvas Elípticas (como OpenSSH 10 en `54.177.128.80`) completa sin fallos de KEX y autentica llaves PEM exitosamente.

**Non-Goals:**
- No alterar la arquitectura modular de `ssh_negotiation.rs` ni la lógica de sesión en `lib.rs`.
- No requerir DLLs ni instalaciones externas de software (OpenSSL, Strawberry Perl, etc.) en la máquina del desarrollador ni del usuario final.
- No modificar el esquema de base de datos ni el frontend.

## Decisions

### 1. Soporte Criptográfico en Windows
* **Decisión:** Activar el soporte ECDH nativo de Windows WinCNG definiendo `LIBSSH2_ECDSA_WINCNG` a través de la configuración de Cargo (`.cargo/config.toml`).
* **Racional:** WinCNG en Windows 10/11 implementa de forma nativa e hiperoptimizada por hardware las curvas NIST P-256, P-384 y P-521. Al definir este flag en la compilación de `libssh2-sys`, NekoSSH adquiere compatibilidad total con servidores modernos sin inflar el binario ni requerir cadenas de herramientas de compilación externas como Perl o C++ vcpkg.
* **Alternativa descartada:** `vendored-openssl` (rechazada: `openssl-src` falla en entornos Windows estándar debido a scripts de configuración de Perl incompletos, añadiendo fragilidad en CI/CD y dependencias externas innecesarias).
* **Alternativa descartada:** Pedir al usuario que reconfigure el servidor remoto para admitir Diffie-Hellman antiguo (rechazada: un cliente SSH comercial de calidad superior debe ser compatible de fábrica con cualquier servidor, igual que MobaXterm).

### Corrección de Ruta (Fix): Pivote a Soporte Nativo WinCNG ECDH (`LIBSSH2_ECDSA_WINCNG`)

* **Razón del Pivote:** Durante la prueba empírica de `vendored-openssl`, la compilación estática de `openssl-src` falló en sistemas Windows debido a que la versión de Perl disponible en el entorno carece del módulo `Locale::Maketext::Simple` requerido por los scripts de configuración de OpenSSL. Forzar la instalación de dependencias externas violaría el principio de portabilidad y autosuficiencia del proyecto.
* **Descubrimiento Técnico:** La auditoría profunda del código fuente de `libssh2-sys-0.3.2` (`libssh2/src/wincng.h`, `wincng.c` y `kex.c`) reveló que la API criptográfica nativa de Windows (WinCNG) ya implementa completamente los algoritmos ECDH (`ecdh-sha2-nistp256`, `ecdh-sha2-nistp384`, `ecdh-sha2-nistp521`). `libssh2` deshabilita estas curvas por defecto para mantener retrocompatibilidad con Windows 7, pero expone el flag de compilación `LIBSSH2_ECDSA_WINCNG`.
* **Solución Adoptada:** Activar el flag `/DLIBSSH2_ECDSA_WINCNG` en la configuración de Cargo (`.cargo/config.toml` tanto a nivel raíz como en `app/src-tauri/.cargo/config.toml`) mediante las variables de entorno de compilación `CL` y `CFLAGS`.
* **Evidencia Empírica de Validación:**
  - El sniffer local (`scratch/sniff_client_kex.py`) confirmó que el cliente emite inmediatamente `ecdh-sha2-nistp256`, `ecdh-sha2-nistp384` y `ecdh-sha2-nistp521` al inicio de la lista KEX.
  - La prueba en vivo contra el servidor OpenSSH 10 de Lapi (`54.177.128.80:22`) negoció exitosamente KEX `ecdh-sha2-nistp256`, HostKey `ecdsa-sha2-nistp256` y Cipher `chacha20-poly1305@openssh.com`.
  - La autenticación por llave privada con `lapi-prd-key.pem` completó con éxito total (`Authenticated: true`).

## Risks / Trade-offs

- **[Riesgo] Alcance en versiones de Windows anteriores a Windows 10**
  - *Mitigación:* WinCNG implementa `BCRYPT_KDF_RAW_SECRET` a partir de Windows 10. Toda la base instalada de NekoSSH opera en Windows 10 y Windows 11 modernos.
- **[Trade-off] `curve25519-sha256` en WinCNG**
  - *Mitigación:* Los servidores SSH modernos (OpenSSH 9.8+ y 10.0) aceptan universalmente `ecdh-sha2-nistp256`, satisfaciendo la negociación sin fricción.

## Migration Plan

- Crear `.cargo/config.toml` con las variables de compilación `CL` y `CFLAGS` para `LIBSSH2_ECDSA_WINCNG`.
- Recompilar `libssh2-sys` con `cargo clean -p libssh2-sys`.
- Cero migraciones de base de datos ni pasos de despliegue manual.
