**Surface types:** desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)
- [x] 0.1 Crear y posicionarse en la rama de feature `feature/ai-harness-core` (`git checkout -b feature/ai-harness-core`)
- [x] 0.2 Verificar que el árbol de trabajo esté posicionado en la rama creada

## 1. Modelos de Dominio y Máquina de Estados (FSM)
- [x] 1.1 Crear módulo `src/ai/mod.rs` y definir estados `HarnessPhase`, modos `ExecutionMode` y eventos de transición
- [x] 1.2 Implementar `AgentHarnessFsm` en `src/ai/fsm.rs` con pruebas TDD para transiciones de fase, freno en error y estado `PausedOnDisconnect` con reanudación

## 2. Contratos y Validación de Esquemas Tipados
- [x] 2.1 Definir estructuras `serde` en `src/ai/schema.rs` para solicitudes, respuestas agénticas y propuestas de tareas
- [x] 2.2 Implementar validación de contratos que exijan `why`, `risk_level`, `backup_strategy` y `rollback_cmd`

## 3. Guardrails de Seguridad (Auditor de Riesgo en 4 Etapas, Solo Lectura y Redactor)
- [x] 3.1 Implementar `ReadOnlyEnforcer` en `src/ai/risk_guard.rs` bloqueando mutaciones y redirecciones en fase de reconocimiento
- [x] 3.2 Implementar el pipeline de 4 etapas de `CommandRiskGuard` (tokenizador, detección de pipes a shell, verbos/flags y redirecciones) clasificando en 4 niveles de severidad
- [x] 3.3 Implementar `SecretRedactor` en `src/ai/redactor.rs` para enmascarar bloques PEM, credenciales y tokens

## 4. Ensamblador de Prompt en Dos Capas y Persistencia en SQLite
- [x] 4.1 Implementar Constitución fija del arnés y ensamblador `PromptAssembler` en `src/ai/prompt.rs`
- [x] 4.2 Agregar helpers en `src/ai/preferences.rs` para persistir directivas del operador y modo de ejecución (`Strict` vs `Flexible`) en `app_preferences`

## 5. Exposición de Comandos IPC Tauri y Registry Multi-Sesión
- [x] 5.1 Implementar `HarnessRegistry` indexado por terminal y declarar comandos Tauri en `src/ai/commands.rs`
- [x] 5.2 Registrar comandos IPC en `src/lib.rs` manteniendo acoplamiento mínimo

## 6. Review and Update Existing Unit Tests (MANDATORY)
- [x] 6.1 Revisar suite existente en `cargo test` asegurando compatibilidad con el nuevo módulo `ai`

## 7. Run Unit Tests and Verify Local DB (MANDATORY)
- [x] 7.1 Ejecutar suite completa `cargo test` validando FSM, modos estricto/flexible, guardrails, redactor y persistencia SQLite en memoria
- [x] 7.2 Generar reporte de verificación en `openspec/changes/ai-harness-core/reports/2026-11-24-step-7-unit-test-and-db-verification.md`

## 8. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)
- [x] 8.1 Invocar y verificar los comandos IPC de prueba del arnés, auditor de riesgo y preferencias
- [x] 8.2 Generar reporte de comandos en `openspec/changes/ai-harness-core/reports/2026-11-24-step-desktop-commands-verification.md`

## 9. Update Technical Documentation (MANDATORY)
- [x] 9.1 Actualizar `docs/plan/integracion-agentes-ia.md` reflejando el progreso del Hito 1
