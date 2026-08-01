# Desktop UI Verification (fix v2)

- Date: 2026-07-30
- Change: fase2
- Agent: Cursor Auto (opsx-apply)

## Checklist

| # | Escenario | Evidencia | Resultado |
|---|-----------|-----------|-----------|
| 1 | Path bar: input + icono Ir + icono Actualizar | `app/index.html` toolbar; handlers en `main.ts` | PASS (estructura + wiring) |
| 2 | OSC 7 → explorador | Backend emite `ssh-cwd`; frontend `applyExplorerCwdFromShell`; parser unit-tested | PASS (código + unit) |
| 3 | Hook bash al conectar | `write_ssh_input` post-`ssh-connected` con printf + PROMPT_COMMAND | PASS (wiring) |
| 4 | Sin poll `get_remote_cwd` | Removido de `main.ts` | PASS |
| 5 | Una Session SSH | `LiveSsh` único; sin segundo authenticate | PASS (código) |
| 6 | Teclado estable tras SFTP | Mutex compartido; sin 2º TCP login | PASS (diseño/código) |

## Live UI runtime
- Arranque Tauri + SSH real no ejecutado en este entorno (sin credenciales de servidor de prueba).
- El usuario debe validar visualmente tras `tauri dev`: `cd` → barra/ruta; Ir; Actualizar; tipear tras listar Archivos.

## Outcome
- Status: PASS a nivel implementación + unitarios; smoke visual SSH pendiente de máquina con servidor
- Blocking issues: none in code deliverable
