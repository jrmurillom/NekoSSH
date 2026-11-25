# Reporte de Verificación de Comandos Desktop (Step 7)

- **Change:** `ai-guided-execution` (Hito 4)
- **Fecha:** 2026-11-24
- **Superficie:** `desktop-commands`
- **Harness de Prueba:** Validación IPC en `app_lib` (`cargo test ai::`) y compilación de registro en `cargo check --bin app`

## 1. Comandos IPC de Fase 3 Registrados en `src/lib.rs`

Se implementaron y registraron en `tauri::generate_handler!`:

1. `execute_ai_plan_step_cmd`: valida estado `GuidedExecution`, aplica edición inline opcional re-auditando severidad con `CommandRiskGuard`, bloquea colisiones si ya existe un centinela activo, arma `SentinelPurpose::PlanStep` e inyecta el wrapper POSIX inmune a eco TTY en el PTY.
2. `interrupt_ai_pty_step_cmd`: escribe `\x03` (`SIGINT`) en el PTY, desarma inmediatamente el centinela en `PtyTapRegistry` (`take_sentinel`) y transiciona la FSM a `HaltedOnError` con `exit_code = 130`.
3. `retry_halted_step_cmd`: re-audita y aplica el comando corregido mediante `resume_from_error`, restaurando `GuidedExecution` en el `step_index` detenido.
4. `execute_ai_rollback_step_cmd`: valida el índice solicitado (`validate_rollback_request` — tanto paso fallido como pasos previos completados no revertidos), arma `SentinelPurpose::StepRollback` e inyecta el comando de reversión con centinela POSIX.

## 2. Sincronización Atómica en Hilo Lector SSH (`src/lib.rs`)

- En `emit_filtered_ssh_stdout`: al completarse cualquier centinela (`res.sentinel_completed`), el backend sanitiza `captured_output` con `SecretRedactor`, ejecuta `apply_sentinel_completion_to_fsm` directamente sobre `HarnessRegistry` (`complete_current_step`, `halt_on_error` o `mark_step_rolled_back`) y adjunta `updated_phase` en el evento `ai-pty-command-finished`.
- En el cierre EOF/desconexión del lector PTY: si existía un `SentinelPurpose::PlanStep` en vuelo sin completar, el backend registra primero `HaltedOnError` (`exit_code = -1`) antes de envolver en `PausedOnDisconnect` y emite `ai-harness-phase-changed`.

## 3. Evidencia de Ejecución

- `ai::commands::tests::fase_3_dispatch_ctrl_c_reintento_y_rollback_quirurgico` ... `ok`
- `ai::pty_tap::tests::wrapper_inmune_a_eco_tty_y_comentarios_en_mismo_paquete_tcp` ... `ok`
- `ai::fsm::tests::edicion_inline_rollback_idempotente_y_reemplazo_de_cola_pendiente` ... `ok`
- `cargo check --bin app` ... `Finished dev profile in 6.26s`

## 4. Conclusión
Todos los comandos IPC y mecanismos de sincronización atómica de Fase 3 operan deterministamente y pasan validación completa.
