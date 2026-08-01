# Unit test and DB verification — manager-profiles

- Date: 2026-07-31
- Change: manager-profiles
- Command: `cargo test` (cwd `app/src-tauri`)
- Result: **18 passed**, 0 failed

## Coverage relevant to this change

| Test | Verifica |
|------|----------|
| `crea_lista_y_elimina_perfil` | Perfil con `folder_id` (default General) |
| `actualiza_perfil_y_credenciales` | Update conserva/exige `folder_id` |
| `eliminar_perfil_cascada_credenciales` | Cascade credenciales intacto |
| `carpeta_crud_y_conexion_en_carpeta` | create/rename/list folder; profile in folder; cascade delete folder; delete idempotente |
| `migracion_backfill_profiles_sin_folder` | Legacy profile sin columna → `ensure_connection_folders_schema` backfill a General |

## Schema

- Migración plugin: `002_connection_folders.sql` (versión 2)
- Path rusqlite / tests: `ensure_connection_folders_schema` idempotente (CREATE IF NOT EXISTS + ADD COLUMN si falta + backfill)
- DB de usuario: no mutada en esta verificación (solo in-memory)

## Frontend build

- `npm run build` (tsc + vite) en `app/`: **PASS**

## Outcome

PASS — listo para verificación de commands/UI.
