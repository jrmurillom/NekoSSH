**Surface types:** desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)
- [x] 0.1 Confirmar estado limpio de Git y crear/posicionarse en la rama `feature/ai-provider-driver-agy` (`git checkout -b feature/ai-provider-driver-agy`)
- [x] 0.2 Verificar que el árbol de trabajo esté posicionado en la rama creada

## 1. Remediación de Auditoría sobre Núcleo Hito 1 (TDD)
- [x] 1.1 Corregir `HaltedOnError` y `resume_from_error` en `src/ai/fsm.rs` para preservar `tasks`, aplicar `fixed_command`, agregar `turns: Vec<ConversationTurn>`, `provider_conversation_id` y contador `generation`
- [x] 1.2 Endurecer `ReadOnlyEnforcer` y `CommandRiskGuard` en `src/ai/risk_guard.rs` evaluando tokens reales por segmento (pipes sin espacio `|bash`, `| sudo sh`, subcomandos exactos sin falso positivo `status`, `tee`, `crontab`, `find -delete`/`-exec`, contenedores)
- [x] 1.3 Refactorizar `SecretRedactor` en `src/ai/redactor.rs` con búsqueda ASCII *case-insensitive* sobre bytes libre de `panic!` ante caracteres UTF-8 multibyte (`ß`, `İ`)
- [x] 1.4 Implementar `AgentTurnWireEnvelope` plano y su esquema JSON canónico + conversión validada hacia `AgentResponseKind` en `src/ai/schema.rs`

## 2. Preferencias Universales Desacopladas y Puerto `AiProviderDriver`
- [x] 2.1 Extender `AiPreferencesDto` en `src/ai/preferences.rs` con `active_driver`, `selected_model` y `driver_options: HashMap<String, String>` en SQLite `app_preferences`
- [x] 2.2 Definir en `src/ai/driver.rs` el `trait AiProviderDriver` (`probe_health`, `list_models`, `execute_turn` con callback de progreso y `TurnCancellationToken`), `AiModelDescriptor`, `AgentTurnRequest`, `AgentTurnOutcome` y `AiDriverError`
- [x] 2.3 Implementar `FakeAiDriver` en `src/ai/drivers/fake.rs` y `src/ai/drivers/mod.rs` soportando catálogo simulado de modelos, eventos de progreso, turnos programados y cancelación

## 3. Adaptador Aislado `AgyCliDriver` (`agy.EXE` Headless + Stream JSON)
- [x] 3.1 Implementar resolución de binario (`driver_options["agy_binary_path"]` / `PATH`), `probe_health` y parser de `agy models` (`<id>\t<label>`) con `CREATE_NO_WINDOW` en `src/ai/drivers/agy.rs`
- [x] 3.2 Implementar ejecución aislada en `{appData}/ai-sandbox/` con archivo de esquema RAII, payload acotado en `--print` (`MAX_CLI_PROMPT_CHARS`), flags `--output-format stream-json`, `--sandbox`, `--disable-slash-commands`, `--model` y `--conversation` (sin `--dangerously-skip-permissions`)
- [x] 3.3 Implementar lector línea a línea de `stream-json`, notificación de progreso, mapeo de errores en español latino y terminación cooperativa del subproceso hijo con pruebas unitarias de contrato

## 4. Orquestador Concurrente sin Bloqueo y Comandos IPC Tauri
- [x] 4.1 Implementar `HarnessOrchestrator` y `AiTurnCancelRegistry` en `src/ai/orchestrator.rs` aplicando *Snapshot → Unlock → Inferencia → Relock con verificación de `generation`*, `SecretRedactor`, `ReadOnlyEnforcer` y `CommandRiskGuard`
- [x] 4.2 Exponer comandos `probe_ai_driver_health_cmd`, `list_ai_models_cmd`, `run_ai_harness_turn_cmd` (emitiendo `ai-turn-progress`) y `cancel_ai_turn_cmd` en `src/ai/commands.rs`, registrándolos junto a `cancel_all()` en `src/lib.rs`

## 5. Review and Update Existing Unit Tests (MANDATORY)
- [x] 5.1 Revisar y ampliar las pruebas existentes de `ai::fsm`, `ai::risk_guard`, `ai::redactor`, `ai::preferences` y `ai::commands` cubriendo los 5 hallazgos de auditoría y los nuevos contratos

## 6. Run Unit Tests and Verify Local DB (MANDATORY)
- [x] 6.1 Ejecutar suite completa `cargo test` verificando cero regresiones, seguridad UTF-8, preservación de plan en error, `FakeAiDriver`, contrato `AgyCliDriver`, concurrencia del orquestador y SQLite en memoria
- [x] 6.2 Generar reporte de verificación en `openspec/changes/ai-provider-driver-agy/reports/2026-11-24-step-6-unit-test-and-db-verification.md`

## 7. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)
- [x] 7.1 Ejecutar verificación real de comandos (`probe_ai_driver_health_cmd`, `list_ai_models_cmd` contra `agy` disponible, flujo completo de orquestador y cancelación)
- [x] 7.2 Generar reporte de comandos en `openspec/changes/ai-provider-driver-agy/reports/2026-11-24-step-desktop-commands-verification.md`

## 8. Update Technical Documentation (MANDATORY)
- [x] 8.1 Actualizar `docs/plan/integracion-agentes-ia.md` reflejando el alcance enriquecido (catálogo de modelos, `stream-json` y blindajes de auditoría) y estatus del Hito 2
