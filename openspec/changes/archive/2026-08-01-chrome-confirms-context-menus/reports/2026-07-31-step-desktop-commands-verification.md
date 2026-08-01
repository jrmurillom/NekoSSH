# Desktop commands verification

**Change:** `chrome-confirms-context-menus`  
**Fecha:** 2026-07-31  
**Rama:** `feature/chrome-confirms-context-menus`

## Método

Inspección estática de `app/src/main.ts` (invoke sites) tras migrar confirms/alerts y menús contextuales.

## Commands usados (sin cambio de contrato)

| Acción UI | Command | Notas |
|-----------|---------|--------|
| Listar árbol | `list_folders`, `get_profiles` | Sin cambio |
| Crear carpeta | `create_folder` | Sin cambio |
| Renombrar carpeta | `update_folder` | Disparo desde menú → inline |
| Eliminar carpeta | `get_folder_connection_count`, `delete_folder` | Confirm A1 antes |
| Crear/editar perfil | `create_profile`, `update_profile` | Editar vía menú → modal |
| Renombrar conexión | `update_profile` | Solo `name` vía inline |
| Eliminar conexión | `delete_profile` | Confirm A1 antes |

## Resultado

**PASS** — CRUD folders/profiles sigue usando los mismos Tauri commands; solo cambió el chrome de invocación (dialog A1 + context menu B3).
