# Reporte de Verificación: Pruebas Unitarias y Regresión (ssh-negotiation-compat)

**Fecha:** 2026-09-28  
**Cambio:** `ssh-negotiation-compat`  
**Superficie:** `desktop-commands`

---

## 1. Alcance de las Pruebas

Se diseñó e implementó una suite de pruebas con **tolerancia cero a fallos** dentro del nuevo módulo `app/src-tauri/src/ssh_negotiation.rs`, además de ejecutar la totalidad de tests de regresión del backend de Rust y del frontend en TypeScript.

---

## 2. Resultados de Pruebas Unitarias del Módulo `ssh_negotiation`

### a) `test_kex_precedence_and_security`
- **Objetivo:** Garantizar que los algoritmos de alta seguridad basados en curvas elípticas (`curve25519-sha256`) tengan precedencia estricta sobre los algoritmos de compatibilidad (`diffie-hellman-group-exchange-sha256` y `diffie-hellman-group14-sha1`).
- **Resultado:** PASS (`ok`)

### b) `test_cipher_precedence_and_security`
- **Objetivo:** Garantizar que los cifrados modernos AEAD/CTR (`aes256-gcm@openssh.com`, `chacha20-poly1305@openssh.com`, `aes256-ctr`) se ubiquen obligatoriamente antes que los cifrados CBC (`aes256-cbc`) y 3DES (`3des-cbc`).
- **Resultado:** PASS (`ok`)

### c) `test_cpanel_centos_mandatory_algorithms_present`
- **Objetivo:** Verificar la presencia innegociable de los algoritmos requeridos para conectar exitosamente a servidores cPanel, CentOS y appliances empresariales:
  - `diffie-hellman-group-exchange-sha256`
  - `diffie-hellman-group-exchange-sha1`
  - `diffie-hellman-group14-sha1`
  - `aes256-cbc`
  - `aes128-cbc`
  - `3des-cbc`
- **Resultado:** PASS (`ok`)

### d) `test_no_duplicate_methods`
- **Objetivo:** Verificar que no existan entradas duplicadas en ninguna de las cadenas de métodos (`KEX`, `CIPHER`, `HOSTKEY`, `MAC`).
- **Resultado:** PASS (`ok`)

### e) `test_configure_session_methods_on_live_session`
- **Objetivo:** Ejecutar `configure_session_methods` sobre una instancia viva de `ssh2::Session` en memoria y comprobar que `libssh2` acepta e inicializa las preferencias sin errores de API en el sistema operativo Windows.
- **Resultado:** PASS (`ok`)

---

## 3. Resumen Global de Ejecución

- **Backend Rust (`cargo test --lib`):**
  - **Total de pruebas:** 71
  - **Aprobadas:** 71
  - **Fallidas:** 0
  - **Warnings:** 0
- **Frontend Vitest (`npm run test`):**
  - **Archivos de prueba:** 13 passed (13)
  - **Total de pruebas:** 94 passed (94)
  - **Fallidas:** 0

---

## 4. Conclusión

La modularización de algoritmos SSH cumple al 100% las especificaciones de seguridad y compatibilidad, certificando cero regresiones en la suite global de NekoSSH.
