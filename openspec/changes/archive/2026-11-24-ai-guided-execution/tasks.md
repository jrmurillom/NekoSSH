**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)
- [x] 0.1 Crear y posicionarse en la rama de feature `feature/ai-guided-execution` (`git checkout -b feature/ai-guided-execution`)
- [x] 0.2 Verificar que el árbol de trabajo esté posicionado en la rama creada

## 1. Blindaje de Centinela POSIX, FSM de Fase 3 y Rollback en Backend (TDD Rust)
- [x] 1.1 Extender `AgentTaskProposal` en `src/ai/schema.rs` con `#[serde(default)] pub is_rolled_back: bool` manteniendo compatibilidad completa con el JSON Schema de salida del proveedor
- [x] 1.2 Extender `AgentHarnessFsm` en `src/ai/fsm.rs` con edición y re-auditoría del comando actual (`update_current_step_command`), marcado idempotente de reversión por índice (`mark_step_rolled_back`) y actualización de cola pendiente desde `HaltedOnError` (`replace_remaining_plan_from_error`) con pruebas TDD
- [x] 1.3 Evolucionar `src/ai/pty_tap.rs` al wrapper agrupado inmune a eco TTY y comentarios (`{ <cmd>\n} ; __neko_rc=$? ; printf '\n%s_%s:%d__\n' '__NEKO_END' '<nonce>' "$__neko_rc"`), agregar `SentinelPurpose` (`ReadOnlyRecon`, `PlanStep`, `StepRollback`), bloqueo de colisiones `try_arm_sentinel` e iteración multi-ocurrencia en el mismo paquete TCP con pruebas TDD
- [x] 1.4 Actualizar `HarnessOrchestrator` en `src/ai/orchestrator.rs` para inyectar el contexto del paso fallido (`step_index`, `failed_command`, `exit_code`, `stderr`) en `HaltedOnError` y soportar reemplazo de pasos restantes cuando el modelo proponga un plan ajustado

## 2. Sincronización Atómica en Hilo Lector SSH y Comandos IPC de Fase 3
- [x] 2.1 Conectar en `src/lib.rs` (`emit_filtered_ssh_stdout` y cierre EOF del PTY) la actualización atómica de `HarnessRegistry` al completarse un centinela (`PlanStep` -> `complete_current_step` / `halt_on_error`, `StepRollback` -> `mark_step_rolled_back`) y el paso automático a `HaltedOnError` (`exit_code = -1`) dentro de `PausedOnDisconnect` si cae el socket SSH con un `PlanStep` en vuelo
- [x] 2.2 Implementar y registrar en `src/ai/commands.rs` y `src/lib.rs` los comandos IPC de Fase 3 (`execute_ai_plan_step_cmd`, `interrupt_ai_pty_step_cmd`, `retry_halted_step_cmd`, `execute_ai_rollback_step_cmd`) con re-auditoría `CommandRiskGuard` y pruebas unitarias

## 3. Helpers Puros de Estado para Fase 3 y Rollback LIFO (`vitest`)
- [x] 3.1 Extender `app/src/ai-copilot/copilot-state.ts` y `app/src/ai-copilot/copilot-state.test.ts` con helpers puros para prohibición universal de auto-ejecución en Fase 3, cálculo del candidato recomendado de rollback en orden inverso (LIFO), estados visuales de checklist (`pending`, `active`, `done`, `rolled_back`, `halted`) y validación de comandos editados

## 4. Interfaz Completa de Fase 3, Freno de Error y Rollback en `"neko-ai"`
- [x] 4.1 Actualizar `app/src/icons.ts` (si falta algún icono Lucide outline con `stroke="currentColor"`), `app/ai-copilot.html` y `app/src/ai-copilot/copilot.css` para el checklist de progreso `[i/N]`, editor inline sin *neon glow* (`autocomplete="off"`), estados de emergencia y reversión
- [x] 4.2 Implementar en `app/src/ai-copilot/copilot-main.ts` el flujo interactivo completo de `GuidedExecution` (autorización manual obligatoria en cada paso, edición inline con re-auditoría en vivo `audit_command_risk_cmd` y botón `Ctrl+C`), `HaltedOnError` (diagnóstico con IA, edición/reintento y ejecución supervisada de `rollback_cmd`), reversión quirúrgica de pasos previos completados y tarjeta final `Completed`

## 5. Review and Update Existing Unit Tests (MANDATORY)
- [x] 5.1 Revisar y actualizar las suites existentes en `cargo test` y `npm test` (`vitest`) asegurando cero regresiones tras el nuevo formato de centinela y contratos extendidos

## 6. Run Unit Tests and Verify Local DB (MANDATORY)
- [x] 6.1 Ejecutar `cargo test`, `npm test` y `npm run build` validando ambos bundles HTML y las pruebas en memoria de SQLite y arnés
- [x] 6.2 Generar reporte en `openspec/changes/ai-guided-execution/reports/2026-11-24-step-6-unit-test-and-db-verification.md`

## 7. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)
- [x] 7.1 Verificar la ejecución programática de los comandos IPC de Fase 3 (`execute_ai_plan_step_cmd`, `interrupt_ai_pty_step_cmd`, `retry_halted_step_cmd`, `execute_ai_rollback_step_cmd`), centinela inmune a eco TTY y commit atómico de FSM
- [x] 7.2 Generar reporte en `openspec/changes/ai-guided-execution/reports/2026-11-24-step-desktop-commands-verification.md`

## 8. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)
- [x] 8.1 Ejecutar validación visual y de DOM sobre `"main"` y `"neko-ai"` verificando checklist `[i/N]`, edición inline con `autocomplete="off"`, semáforo de severidad, vista `HaltedOnError`/Rollback, cierre `Completed` e iconos Lucide outline
- [x] 8.2 Generar reporte en `openspec/changes/ai-guided-execution/reports/2026-11-24-step-desktop-ui-verification.md`

## 9. Update Technical Documentation (MANDATORY)
- [x] 9.1 Actualizar `docs/plan/integracion-agentes-ia.md` (Hito 4 `✅ COMPLETADO`), `docs/project_scope.md`, `docs/design/DESIGN.md` y `docs/design/ui-layout-contract.md` consolidando el cierre completo de la integración de agentes IA
