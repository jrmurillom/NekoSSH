**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Feature Branch Verification (MANDATORY)

- [x] 0.1 Verificar que el trabajo corre sobre la rama activa `feature/ai-conversational-copilot-v2` sin tocar `main`

## 1. Driver `agy`: Kill-Tree Recursivo y Retorno No Bloqueante (TDD)

- [x] 1.1 Escribir pruebas unitarias que fallen en `app/src-tauri/src/ai/drivers/agy.rs` verificando terminación del árbol hijo y retorno inmediato (< 1s) cuando un proceso shim/subproceso mantiene pipes abiertos al cancelarse o expirar timeout
- [x] 1.2 Implementar `terminate_process_tree` (`taskkill /F /T /PID` en Windows + `child.kill()`/`wait()`) y retorno inmediato sin bloqueo en `join()` dentro de `app/src-tauri/src/ai/drivers/agy.rs`

## 2. `CommandRiskGuard`: Blindaje de Wrappers y Purgas de Bitácora (TDD)

- [x] 2.1 Escribir pruebas unitarias que fallen en `app/src-tauri/src/ai/risk_guard.rs` para `xargs kill -9`, `xargs rm -f`, `timeout 10 systemctl restart nginx`, `nice -n 5 rm -rf /tmp/x`, `ionice -c3 cp a b`, `stdbuf -oL apt-get upgrade -y`, `journalctl --vacuum-time=1d`, `journalctl --rotate`, además de usos benignos (`timeout 5s ping -c 1 8.8.8.8`, `echo a | xargs`, `journalctl -u nginx -n 20`)
- [x] 2.2 Refactorizar `extract_effective_verb_and_tail` y añadir reglas `journalctl` en `app/src-tauri/src/ai/risk_guard.rs`

## 3. `PtyContextTap`: Higiene ANSI en Eco (`zsh`) y Detección de Espera Interactiva / `fish` (TDD)

- [x] 3.1 Escribir pruebas unitarias que fallen en `app/src-tauri/src/ai/pty_tap.rs` para eco coloreado con secuencias CSI (`zsh-syntax-highlighting`) y detección `waiting_input_hint` (`[sudo] password`, `[y/N]`, `fish` `$status`), garantizando emisión única por centinela
- [x] 3.2 Actualizar `strip_echo_plumbing`, agregar `detect_waiting_input_hint` en `app/src-tauri/src/ai/pty_tap.rs` y emitir `"ai-pty-waiting-input"` desde `emit_filtered_ssh_stdout` en `app/src-tauri/src/lib.rs`

## 4. Estado Operativo HITL Indexado por Sesión (`terminal_id`) y UI Reactiva (TDD)

- [x] 4.1 Escribir pruebas unitarias en `app/src-tauri/src/ai/commands.rs` y `app/src/ai-copilot/copilot-state.test.ts` para `AiUiSessionState` (preservación de `pending_action`, `last_failed` y `last_rollback` por `session_id`) y gestor de mapa por `terminal_id` en frontend
- [x] 4.2 Implementar almacenamiento por sesión (`get_ai_session_ui_state_cmd`, `save_ai_session_ui_state_cmd` y limpieza en `clear_ai_conversation_cmd` / `close_ssh_session`) en `app/src-tauri/src/ai/commands.rs` y `app/src-tauri/src/lib.rs`
- [x] 4.3 Actualizar `app/src/ai-copilot/copilot-state.ts`, `app/src/ai-copilot/copilot-main.ts` y `app/src/ai-copilot/copilot.css` para guardar/restaurar estado HITL por `terminal_id` al alternar pestañas o reabrir `"neko-ai"` y mostrar el aviso de espera interactiva en tarjetas activas

## 5. Review and Update Existing Unit Tests (MANDATORY)

- [x] 5.1 Revisar y asegurar que todas las pruebas unitarias existentes de `agy.rs`, `risk_guard.rs`, `pty_tap.rs`, `commands.rs` y `copilot-state.test.ts` sigan aprobando sin atajos

## 6. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 6.1 Ejecutar `cargo test --lib`, `cargo check --bin app`, `npm test` y `npm run build` desde `app/`, verificando ausencia de mutaciones SQLite y generando `openspec/changes/ai-critical-hardening/reports/2026-11-25-step-6-unit-test-and-db-verification.md`

## 7. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 7.1 Verificar comandos IPC nuevos y existentes (`get_ai_session_ui_state_cmd`, `save_ai_session_ui_state_cmd`, `clear_ai_conversation_cmd`, `audit_command_risk_cmd`) mediante pruebas en `app/src-tauri` y registrar evidencia en `openspec/changes/ai-critical-hardening/reports/2026-11-25-step-7-desktop-commands-verification.md`

## 8. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 8.1 Servir `app/dist` y verificar renderizado con Edge headless (`--headless --dump-dom`) más escenarios de estado por `terminal_id`, generando `openspec/changes/ai-critical-hardening/reports/2026-11-25-step-8-desktop-ui-verification.md`

## 9. Update Technical Documentation (MANDATORY)

- [x] 9.1 Actualizar `docs/design/ui-layout-contract.md` si cambia el comportamiento de ciclo de vida/preservación por pestaña y espera interactiva de `"neko-ai"`
