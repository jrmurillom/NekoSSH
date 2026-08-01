# Desktop commands verification — fase2-3-manual-reconnect

- Date: 2026-07-31
- Change: fase2-3-manual-reconnect
- Method: inspección de wiring frontend → commands existentes

## Commands reused

| Command | Uso en reconnect |
|---------|------------------|
| `close_ssh_session` | Cleanup idempotente antes de reabrir mismo `terminalId` |
| `start_ssh_session` | Reabrir sesión con perfil de la pestaña |
| `resize_ssh_pty` / `write_ssh_input` | Sin cambio (flujo post-`ssh-connected`) |

## Notes

- No se añadieron commands Tauri nuevos.
- `reconnectTerminalSession` relee perfil por `id` desde `currentProfiles` si existe.

## Outcome

PASS (contrato IPC sin regresión; reconnect es composición de commands existentes)
