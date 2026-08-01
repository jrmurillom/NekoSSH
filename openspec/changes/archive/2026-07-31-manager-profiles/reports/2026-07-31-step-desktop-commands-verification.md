# Desktop commands verification — manager-profiles

- Date: 2026-07-31
- Change: manager-profiles
- Method: capa DB compartida por los Tauri commands (rusqlite helpers) vía `cargo test` + registro en `invoke_handler`

## Commands registrados

| Command | Handler | Verificación |
|---------|---------|--------------|
| `list_folders` | `list_folders_from_db` | PASS (`carpeta_crud_y_conexion_en_carpeta`) |
| `create_folder` | `create_folder_in_db` | PASS |
| `update_folder` | `update_folder_in_db` | PASS (rename) |
| `delete_folder` | `delete_folder_in_db` | PASS (cascade + idempotente) |
| `get_folder_connection_count` | `count_profiles_in_folder` | PASS (conteo en test de carpeta) |
| `get_profiles` | `list_profiles_from_db` (+ `folder_id`) | PASS |
| `create_profile` / `update_profile` | exige/resuelve `folder_id` | PASS |
| `delete_profile` | sin regresión | PASS |

## Notas

- Los commands Tauri son thin wrappers sobre `get_db_conn` → mismos helpers probados.
- Migración `version: 2` registrada en `run()` junto a `001`.
- Invocación IPC en app viva no ejecutada en este paso (requiere `tauri dev`); contratos de argumentos: snake_case en payload (`folder_id`, `sort_order` vía serde / camelCase opcional en create_folder `sortOrder`).

## Outcome

PASS a nivel de implementación + unit tests de la capa de commands.
