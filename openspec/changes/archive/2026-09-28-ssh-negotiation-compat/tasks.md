**Surface types:** desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear rama `feature/ssh-negotiation-compat` y cambiar a ella
- [x] 0.2 Verificar rama actual con `git branch --show-current`

## 1. Módulo Desacoplado de Negociación SSH (`ssh_negotiation.rs`)

- [x] 1.1 Crear el módulo `app/src-tauri/src/ssh_negotiation.rs` con constantes fuertemente tipadas y ordenadas por nivel de seguridad para `KEX_PREFERENCES`, `CIPHER_PREFERENCES`, `HOSTKEY_PREFERENCES` y `MAC_PREFERENCES`
- [x] 1.2 Incluir en `KEX_PREFERENCES` algoritmos de compatibilidad legacy al final: `diffie-hellman-group-exchange-sha256`, `diffie-hellman-group14-sha1`, `diffie-hellman-group-exchange-sha1`
- [x] 1.3 Incluir en `CIPHER_PREFERENCES` algoritmos CBC al final como fallback: `aes256-cbc`, `aes192-cbc`, `aes128-cbc`, `3des-cbc`
- [x] 1.4 Definir e implementar la función pública `configure_session_methods(sess: &mut ssh2::Session) -> Result<(), String>` que aplique todas las preferencias a la sesión de `libssh2` con manejo de errores descriptivo
- [x] 1.5 Declarar el submódulo `pub mod ssh_negotiation;` en `app/src-tauri/src/lib.rs`

## 2. Integración en el Flujo de Handshake en `lib.rs`

- [x] 2.1 Reemplazar las llamadas hardcodeadas a `sess.method_pref` en `authenticate_session_once` de `app/src-tauri/src/lib.rs` por la invocación modular a `ssh_negotiation::configure_session_methods(&mut sess)`
- [x] 2.2 Garantizar que los errores de configuración de métodos se propaguen con contexto claro antes de iniciar `sess.handshake()`

## 3. Pruebas Unitarias Robustas con Tolerancia Cero a Fallos

- [x] 3.1 Implementar en `app/src-tauri/src/ssh_negotiation.rs` pruebas que verifiquen que los algoritmos prioritarios de máxima seguridad (Curve25519, ChaCha20, AES-GCM, AES-CTR) preceden estrictamente a los algoritmos legacy (DH-GEX, CBC) en cada lista
- [x] 3.2 Implementar prueba unitaria que valide la presencia obligatoria de los algoritmos requeridos por cPanel/CentOS (`diffie-hellman-group-exchange-sha256`, `aes256-cbc`, `3des-cbc`)
- [x] 3.3 Implementar prueba unitaria que certifique que `configure_session_methods` se ejecuta exitosamente sobre una instancia viva de `ssh2::Session` sin errores de API

## 4. Run Unit Tests & Regression Verification (MANDATORY)

- [x] 4.1 Ejecutar la suite completa de pruebas unitarias en Rust con `cargo test` para verificar 0 fallos
- [x] 4.2 Ejecutar la suite de pruebas unitarias de frontend con `npm run test` para garantizar cero regresiones
- [x] 4.3 Report: `openspec/changes/ssh-negotiation-compat/reports/2026-09-28-step-4-unit-test-verification.md`

## 5. Desktop Commands Verification (MANDATORY — AGENT MUST EXECUTE)

- [x] 5.1 Verificar la inicialización del handshake y configuración de métodos SSH mediante prueba de comando o sesión de test
- [x] 5.2 Report: `openspec/changes/ssh-negotiation-compat/reports/2026-09-28-step-5-desktop-commands-verification.md`

## 6. Update Technical Documentation (MANDATORY)

- [x] 6.1 Actualizar `docs/design/DESIGN.md` o documentación técnica de arquitectura SSH si corresponde documentar el módulo `ssh_negotiation`
