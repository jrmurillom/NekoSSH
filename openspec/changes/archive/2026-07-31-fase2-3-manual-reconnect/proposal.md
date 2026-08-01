## Why

Cuando la sesión SSH muere (EOF, error de transporte, cierre remoto), NekoSSH ya marca la pestaña como desconectada y escribe un aviso en el PTY, pero no ofrece un camino claro tipo Terminus/MobaXterm para **reconectar a mano** desde la misma pestaña. Antes de Fase 3 (editor), hace falta esa higiene de UX: avisar bien + `Ctrl+R` + indicador de estado coherente — sin auto-reconnect.

## What Changes

- Mensaje en la terminal al morir la sesión: desconectado + hint **Ctrl+R para reconectar**.
- Atajo **Ctrl+R** (solo con pestaña desconectada) reabre SSH con el **mismo perfil** de esa pestaña.
- Indicador bajo la pestaña (dot + texto) refleja estados claros: conectando / conectado / desconectado / error.
- Guardar en el estado de la pestaña la referencia al perfil (o snapshot mínimo) necesario para reconectar.
- Sin auto-reconnect en loop; sin rehidratar cwd; sin cambios de keepalive (ya existe).

## Capabilities

### New Capabilities

_(ninguna — se extiende el comportamiento de terminal SSH existente)_

### Modified Capabilities

- `ssh-terminal`: aviso de sesión muerta, reconexión manual con Ctrl+R, y estados del indicador de la pestaña.

## Impact

- Frontend: `app/src/main.ts` (handlers `ssh-closed` / `ssh-error`, keybinding, `ActiveTerminal`), estilos del status indicator.
- Backend: probablemente sin commands nuevos (reusar `start_ssh_session` / flujo actual); solo si hace falta limpiar estado antes de reabrir el mismo `terminal_id`.
- Docs: `ui-layout-contract.md` / `DESIGN.md` si se documenta el patrón de reconnect.
- Fuera de alcance: túneles runtime, host keys, auto-reconnect, Fase 3 Monaco.
