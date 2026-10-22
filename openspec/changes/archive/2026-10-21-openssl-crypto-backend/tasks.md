**Surface types:** desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear rama `feature/openssl-crypto-backend` y cambiar a ella
- [x] 0.2 Verificar rama actual con `git branch --show-current`

## 1. Configuración de Soporte Nativo WinCNG ECDH en Cargo

- [x] ~~(DESCARTADO) 1.1 Modificar la dependencia ssh2 en app/src-tauri/Cargo.toml para activar vendored-openssl (Descartado: openssl-src falla en Windows por ausencia de módulos Perl en el entorno; se sustituye por WinCNG nativo)~~
- [x] ~~(DESCARTADO) 1.2 Compilar el árbol de dependencias con cargo check y confirmar resolución de openssl-src (Descartado: no aplica para backend WinCNG)~~
- [x] 1.3 Configurar `.cargo/config.toml` (en raíz del repositorio y en `app/src-tauri/.cargo/config.toml`) inyectando `CL = "/DLIBSSH2_ECDSA_WINCNG"` y `CFLAGS = "-DLIBSSH2_ECDSA_WINCNG"` para compilar `libssh2-sys` con soporte ECDH nativo
- [x] 1.4 Recompilar `libssh2-sys` con `cargo clean -p libssh2-sys` y verificar resolución limpia con `cargo check --manifest-path app/src-tauri/Cargo.toml`

## 2. Verificación de Curvas Elípticas y Negociación KEX

- [x] 2.1 Verificar que `libssh2` con backend WinCNG emite `ecdh-sha2-nistp256`, `ecdh-sha2-nistp384` y `ecdh-sha2-nistp521` en el paquete `SSH_MSG_KEXINIT`
- [x] 2.2 Probar que la función `configure_session_methods` aplica las preferencias en el nuevo entorno manteniendo compatibilidad con Diffie-Hellman y CBC

## 3. Run Unit Tests & Regression Verification (MANDATORY)

- [x] 3.1 Ejecutar la suite completa de pruebas unitarias en Rust con `cargo test --lib --manifest-path app/src-tauri/Cargo.toml` para verificar 0 fallos
- [x] 3.2 Ejecutar la suite completa de pruebas unitarias de frontend con `npm run test` para garantizar cero regresiones
- [x] 3.3 Report: `openspec/changes/openssl-crypto-backend/reports/2026-10-21-step-4-unit-test-verification.md`

## 4. Desktop Commands Verification (MANDATORY — AGENT MUST EXECUTE)

- [x] 4.1 Ejecutar prueba de handshake contra el servidor OpenSSH 10 de Lapi (`54.177.128.80`) certificando que `Session(-5)` desaparece y el handshake completa exitosamente con KEX `ecdh-sha2-nistp256` y auth por llave privada
- [x] 4.2 Report: `openspec/changes/openssl-crypto-backend/reports/2026-10-21-step-5-desktop-commands-verification.md`

## 5. Update Technical Documentation & Cleanup (MANDATORY)

- [x] 5.1 Actualizar `docs/project_scope.md` documentando la activación de ECDH nativo en WinCNG (`LIBSSH2_ECDSA_WINCNG`) para compatibilidad universal de Curvas Elípticas
- [x] 5.2 Limpiar archivos de prueba temporales (`probe_lapi_handshake.rs` y sniffer)
