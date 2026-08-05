# Verification Report: Desktop Commands (store-private-key-in-db)

**Date:** 2026-08-04  
**Change:** `store-private-key-in-db`  
**Surface:** desktop-commands

---

## Scenarios

### 5.1 Save/update con `private_key` y persistencia SQLite

**Ejecución:** `cargo test --lib conserva_private_key_pem_en_round_trip` (mismo código de create/update/list que usan los commands Tauri `create_profile` / `update_profile` / `get_profiles`).

**Resultado:** PASS  
- Inserta PEM en `auth_credentials.private_key`  
- Listado lo recupera intacto  
- Update de nombre conserva el PEM  

### 5.2 Connect sin material / con material vacío

**Ejecución:** `cargo test --lib rechaza_auth_por_llave_sin_private_key`

**Resultado:** PASS  
- `authenticate_session_once(..., auth_type=key, private_key=None)` → `"Llave privada no configurada: falta el material en el perfil"`  
- Idem con `Some("   ")`  
- Falla **antes** de TCP (sin Session(-1) genérico)

**Connect con material válido:** cubierto en verificación UI (paso 6) contra servidor real; no se inventa un SSH mock en este harness.

---

## Conclusion

Commands de persistencia y gate de auth verificados vía las mismas funciones internas que invoca IPC. DB de usuario no mutada (solo in-memory en tests).
