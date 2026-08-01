# Desktop UI Verification (fix v3 — árbol navegable)

- Date: 2026-07-30
- Change: fase2
- Agent: Cursor Auto (opsx-apply)

## Cambios implementados
- OSC 7 con el **mismo path** no fuerza `loadExplorerAt` (preserva expand/collapse).
- Chevron ▶/▼ = expand/collapse lazy (SFTP); click en carpeta = abrir (path + listado).
- Botón **↑** Subir; estados Cargando / vacío / error.
- CSS: panel Archivos con flex + scroll en `.files-tree`.
- Hook OSC 7 con `stty -echo` para reducir eco en el PTY.

## Checklist (código + build)

| # | Escenario | Resultado |
|---|-----------|-----------|
| 1 | Expand no depende de click en toda la fila | PASS (handler en chevron) |
| 2 | Abrir carpeta cambia path y lista | PASS (`openExplorerFolder`) |
| 3 | Subir al padre | PASS (`parentRemotePath` + botón) |
| 4 | OSC mismo path no relista | PASS (`pathsEqual` early return) |
| 5 | Layout scroll | PASS (CSS min-height:0 + overflow) |
| 6 | Unit `parent_remote_path` | PASS (cargo test) |

## Live SSH
- Smoke visual en servidor remoto: no ejecutado en este entorno (sin credenciales). Validar tras `tauri dev`: expand, collapse, abrir, subir, Ir.

## Outcome
- Status: PASS implementación; smoke SSH pendiente de máquina del usuario
