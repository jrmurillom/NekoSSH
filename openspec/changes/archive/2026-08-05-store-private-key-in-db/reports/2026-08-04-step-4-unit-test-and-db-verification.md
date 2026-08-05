# Verification Report: Unit Tests + DB (store-private-key-in-db)

**Date:** 2026-08-04  
**Change:** `store-private-key-in-db`  
**Scope:** Suite unitaria tras rename `key_path` → `private_key`, persistencia PEM y rechazo sin material.

---

## Commands Executed & Results

### 1. Rust (`cargo test --lib`)

```text
running 50 tests
...
test tests::conserva_private_key_pem_en_round_trip ... ok
test tests::rechaza_auth_por_llave_sin_private_key ... ok
test tests::schema_renombra_key_path_a_private_key ... ok
test tests::actualiza_perfil_y_credenciales ... ok
...
test result: ok. 50 passed; 0 failed; 0 ignored
```

### 2. Frontend (`npm run test`)

```text
Test Files  5 passed (5)
Tests  25 passed (25)
```

### 3. Persistencia / DB

- Pruebas usan SQLite **en memoria** (`Connection::open_in_memory`); no se mutó la DB de usuario (`nekossh.db`).
- Restore: N/A — sin mutación de DB persistente.

---

## Tests con valor para este change

| Test | Verdad que dicta |
|------|------------------|
| `conserva_private_key_pem_en_round_trip` | PEM se guarda y sobrevive update |
| `rechaza_auth_por_llave_sin_private_key` | Sin material → error claro antes de TCP |
| `schema_renombra_key_path_a_private_key` | Columna renombrada idempotente |
| `actualiza_perfil_y_credenciales` | Credenciales key usan contenido, no ruta |

---

## Conclusion

Suite verde. No regresiones. DB de app no tocada en este paso.
