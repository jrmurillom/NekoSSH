# Step 6 — Pruebas unitarias y estado de la base local

**Change:** tab-context-multi-shell
**Fecha:** 2026-08-05
**Rama:** `feature/tab-context-multi-shell`

## Alcance

El cambio es de frontend (modelo de contexto de pestaña, grid de shells, enrutamiento de eventos por `terminal_id`). No toca esquema SQLite ni comandos Rust.

## Revisión de pruebas existentes (Step 5)

- Suites Rust existentes (perfiles, credenciales, snippets, preferencias, edit-session, sftp fake): **no afectadas**; el modelo padre/hijos vive en el frontend y reutiliza los comandos por `terminal_id` sin cambiar su firma.
- Suites frontend existentes (`strip-trailing-paste`, `bg-settings-helper`, `sftp-path-helper`, `connection-tree-helper`, `remote-history-helper`): **no afectadas**.
- **Nueva** suite: `app/src/modules/shell-grid-helper.test.ts` para la lógica pura extraída (límite de hijos, densidad del grid, etiqueta de hijo, foco tras cerrar celda).

## Comandos ejecutados

```
cd app/src-tauri && cargo test
cd app && npm test
cd app && npm run build
```

## Resultados

| Suite | Resultado |
|---|---|
| `cargo test` (app_lib + bin + doc-tests) | 50 passed, 0 failed |
| `vitest run` | 6 archivos, 33 passed, 0 failed (8 nuevos en `shell-grid-helper`) |
| `tsc && vite build` | OK, sin errores de tipos |

## Estado de datos (SQLite)

Base: `%APPDATA%\com.nekossh.app\nekossh.db`

| Momento | SHA256 | Bytes |
|---|---|---|
| Pre | `A31FF6F045770CD73F380E320D48187CD7DA784DAEB7F2DA9BF49B9BB871D6A6` | 77824 |
| Post | `A31FF6F045770CD73F380E320D48187CD7DA784DAEB7F2DA9BF49B9BB871D6A6` | 77824 |

Sin mutación: los tests unitarios usan bases temporales en memoria/archivo propio. No hubo restore necesario.
