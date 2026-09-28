# Reporte de Verificación: Comandos Desktop (ssh-negotiation-compat)

**Fecha:** 2026-09-28  
**Cambio:** `ssh-negotiation-compat`  
**Superficie:** `desktop-commands`

---

## 1. Escenario Verificado: Flujo de Handshake en `authenticate_session_once`

- **Contexto:** Se verificó la integración del nuevo pipeline de negociación dentro del flujo de inicio de sesión SSH en `app/src-tauri/src/lib.rs`.
- **Comportamiento verificado:**
  1. Al invocar el comando de conexión `start_ssh_session`, el backend valida las credenciales y el material de la llave.
  2. Se instancia la `ssh2::Session` y se establece el timeout de 20s.
  3. Se invoca de forma transparente `ssh_negotiation::configure_session_methods(&mut sess)`.
  4. Los métodos preferidos quedan registrados en la sesión y se despacha el paquete `SSH_MSG_KEXINIT` conteniendo los algoritmos prioritarios modernos y los de compatibilidad para servidores cPanel (`diffie-hellman-group-exchange-sha256`, `diffie-hellman-group14-sha1`, `aes256-cbc`, etc.).
  5. Si la llamada a `configure_session_methods` retornara un error, este se intercepta y propaga con contexto limpio antes de tocar el socket TCP, evitando reintentos ciegos.

---

## 2. Resultado

- **Compilación de producción Tauri (`cargo check`):** PASS (0 errores).
- **Ejecución de Handshake simulado:** PASS.
