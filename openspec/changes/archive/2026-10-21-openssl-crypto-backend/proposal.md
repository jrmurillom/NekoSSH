## Why

En sistemas Windows, el crate `ssh2` compila por defecto `libssh2` contra la API nativa de Microsoft (**WinCNG**). Por defecto y por razones de retrocompatibilidad con versiones antiguas de Windows, `libssh2` deshabilita el soporte para intercambio de claves por Curvas Elípticas (`ecdh-sha2-*`). Cuando NekoSSH intentaba conectar a servidores Linux modernos (como Debian 13, Ubuntu 24.04 o instancias AWS EC2 con OpenSSH 9.8+ u OpenSSH 10.0), el servidor rechazaba los métodos Diffie-Hellman tradicionales y exigía exclusivamente Curvas Elípticas, provocando un fallo total en la negociación: `[Session(-5)] Unable to exchange encryption keys`.

Para que NekoSSH alcance compatibilidad universal de nivel industrial (al nivel de MobaXterm o PuTTY), es indispensable activar el soporte nativo de Curvas Elípticas en `libssh2` habilitando el flag `LIBSSH2_ECDSA_WINCNG` en el backend WinCNG, permitiendo la negociación fluida de algoritmos ECDH sin introducir dependencias externas pesadas o frágiles en tiempo de compilación.

## What Changes

- **Activación de Curvas Elípticas Nativas en WinCNG (`LIBSSH2_ECDSA_WINCNG`)**: Configurar las directivas de compilación en `.cargo/config.toml` (`CL = "/DLIBSSH2_ECDSA_WINCNG"`, `CFLAGS = "-DLIBSSH2_ECDSA_WINCNG"`) para que `libssh2-sys` habilite de forma nativa `ecdh-sha2-nistp256`, `ecdh-sha2-nistp384` y `ecdh-sha2-nistp521`.
- **Negociación KEX Exitosa con Servidores Modernos**: Permitir que el cliente NekoSSH ofrezca curvas elípticas estándar NIST P-256/384/521, desbloqueando el handshake con servidores OpenSSH 10.0 (como `54.177.128.80`).
- **Preservación Total de Retrocompatibilidad**: Mantener Diffie-Hellman Group 14/16/GEX y cifrados CBC para conexiones a CentOS / cPanel.
- **Verificación Cero Regresiones**: Comprobar que los tests unitarios de Rust y Frontend continúen pasando limpiamente y certificar el handshake y autenticación con llaves PEM.

## Capabilities

### New Capabilities
<!-- Ninguna nueva; pertenece al motor SSH de ssh-terminal -->

### Modified Capabilities
- `ssh-terminal`: Se actualiza la especificación del motor SSH para exigir soporte de Curvas Elípticas (`ecdh-sha2-*`) mediante el backend WinCNG nativo en Windows.

## Impact

- **Backend Rust (`.cargo/config.toml`)**: Configuración de flags de compilación para `libssh2-sys`.
- **Binario de escritorio**: Cero incremento de tamaño y portabilidad 100% nativa.
- **Base de datos / Frontend**: Cero impacto en esquemas SQLite ni en la UI.
