**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Feature Branch (MANDATORY - FIRST STEP)

- [x] 0.1 Crear y cambiar a la rama `feature/fase2-3-manual-reconnect`
- [x] 0.2 Verificar rama actual y working tree listo para cambios en `app/`

## 1. Estado de pestaña y mensajes

- [x] 1.1 Guardar en `ActiveTerminal` el perfil (id y/o snapshot) usado al abrir la sesión
- [x] 1.2 En `ssh-closed` / `ssh-error`: mensaje PTY con hint Ctrl+R; clases de status `disconnected` / `error`
- [x] 1.3 Estilos CSS del indicador (dot + texto) para connecting / connected / disconnected / error

## 2. Reconexión Ctrl+R

- [x] 2.1 Interceptar Ctrl+R solo si `!isConnected` (y no en reconnect concurrente)
- [x] 2.2 Reusar `terminal_id` + perfil: cleanup idempotente si hace falta + `start_ssh_session` / flujo existente
- [x] 2.3 Actualizar indicador a “Conectando…” y luego connected/error; re-habilitar explorador al `ssh-connected` como hoy

## 3. Review and Update Existing Unit Tests (MANDATORY)

- [x] 3.1 Ajustar/añadir tests solo si hay lógica Rust nueva; si el change es solo frontend, documentar N/A en report

## 4. Run Unit Tests and Verify State (MANDATORY)

- [x] 4.1 Ejecutar `cargo test` y/o `npm run build` según lo tocado
- [x] 4.2 Report `openspec/changes/fase2-3-manual-reconnect/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md` (DB N/A salvo mutación)

## 5. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 5.1 Verificar que reconnect reutiliza commands existentes (`start_ssh_session` / close idempotente) sin regresiones (AGENT MUST EXECUTE)
- [x] 5.2 Report `openspec/changes/fase2-3-manual-reconnect/reports/YYYY-MM-DD-step-desktop-commands-verification.md`

## 6. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Validar aviso + Ctrl+R + indicador (runtime o build + inspección estática documentada) (AGENT MUST EXECUTE)
- [x] 6.2 Report `openspec/changes/fase2-3-manual-reconnect/reports/YYYY-MM-DD-step-desktop-ui-verification.md`

## 7. Update Technical Documentation (MANDATORY)

- [x] 7.1 Actualizar `docs/design/ui-layout-contract.md` (reconnect manual / status de pestaña)
- [x] 7.2 Actualizar `DESIGN.md` o README si hay copy/estados visuales nuevos
