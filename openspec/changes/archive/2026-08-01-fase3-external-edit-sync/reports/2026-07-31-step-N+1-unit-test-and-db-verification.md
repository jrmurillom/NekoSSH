# Step N+1 — Unit tests y DB

- Date: 2026-07-31
- Change: `fase3-external-edit-sync`
- Branch: `feature/fase3-external-edit-sync`
- Agent: Cursor Auto (opsx-apply)

## Commands Executed

- `cargo test --manifest-path app/src-tauri/Cargo.toml --lib`
- `npm run build --prefix app`

## Unit Test Results

- **34 passed**, 0 failed
- Cobertura relevante:
  - `preferences`: default vacío + round-trip `preferred_external_editor`
  - `edit_util`: límite 10 MiB, heurística NUL, paths `edit-sessions/<id>/`, sweep huérfanos
  - `fake_sftp`: download/upload local, rechazo tamaño, binario
  - `edit_session`: reuse, coalesce confirm, debounce/fingerprint, preserve temp en disconnect/upload
  - CRUD perfiles/folders + path_util + osc7 (regresión)
- Frontend: `tsc && vite build` **PASS**

## Database

- Migración nueva `003_app_preferences.sql` (tabla `app_preferences` key/value).
- Verificación unitaria con SQLite **in-memory** (`ensure_app_preferences_schema`); sin mutar `nekossh.db` de usuario en esta corrida.
- Baseline/restore de DB de app: **N/A** (no se abrió la DB de producción del usuario).

## Lab SSH: cero mutaciones

- Tests de transfer/edit usan **solo** `FakeSftpStore` (memoria + temp local) y helpers puros.
- **No** se abrió cliente SSH ni se ejecutó upload/replace/delete contra el host de pruebas compartido.
- Sandbox remoto desechable: **no provisionado**; N/A.

## Outcome

- Unit/build: **PASS**
- Listo para desktop-commands (harness local) y desktop-ui (estructura + mock posture).
