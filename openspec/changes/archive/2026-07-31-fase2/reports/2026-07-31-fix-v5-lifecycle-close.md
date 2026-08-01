# Fix v5 — Lifecycle close (2026-07-31)

## Objetivo
Cerrar Sessions SSH al cerrar pestaña(s) o al salir de la app.

## Cambios
- `remove_and_shutdown_ssh` / `shutdown_live_ssh`: `channel.close` + `session.disconnect`; idempotente
- `close_all_ssh_connections` en `RunEvent::ExitRequested` / `Exit`
- Reader PTY: no emite `ssh-closed` si el mapa ya fue limpiado por close explícito
- Frontend: `isConnected=false` antes de invoke; limpia explorador bound; `closeAllTerminals` await secuencial

## Tests

| Prueba | Resultado |
|--------|-----------|
| `cargo test` (16) | PASS |
| `close_all` empty map + remove missing idempotent | PASS |
| `smoke_close_lifecycle` (VPS profile id=1) | PASS |
| UI Tauri E2E cerrar ventana | no corrida aquí (hook nativo cubierto en código) |

## Spec
Delta `specs/ssh-terminal`: requisito “Cierre de conexiones con pestaña y aplicación” (ya en change).
Docs: README + `ui-layout-contract.md`.
