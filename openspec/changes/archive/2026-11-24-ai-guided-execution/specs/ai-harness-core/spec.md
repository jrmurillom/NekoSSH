## MODIFIED Requirements

### Requirement: Transiciones de Máquina de Estados del Arnés
El backend SHALL implementar una máquina de estados finita (`AgentHarnessFsm`) que restrinja estrictamente el ciclo de vida del agente en los estados `Idle`, `Reconnaissance`, `PlanAndDebate`, `GuidedExecution`, `HaltedOnError`, `PausedOnDisconnect` y `Completed`, manteniendo un historial portátil de turnos (`turns: Vec<ConversationTurn>`), un `provider_conversation_id` opcional y soporte determinista de edición de paso actual, registro de reversión por tarea (`mark_step_rolled_back`), actualización de pasos pendientes desde error y protección ante desconexión con paso en vuelo. Queda prohibido saltar a ejecución sin haber aprobado previamente un plan o permanecer en reconocimiento con comandos mutantes.

#### Scenario: Transición válida de Reconocimiento a Plan
- **WHEN** el arnés recibe la orden de iniciar diagnóstico para un objetivo y completa la recolección de lecturas autorizadas
- **THEN** la máquina de estados transiciona de `Reconnaissance` a `PlanAndDebate`, bloqueando toda ejecución en el servidor mientras se debate el plan

#### Scenario: Freno automático ante salida de error en ejecución
- **WHEN** durante el estado `GuidedExecution` un comando ejecutado devuelve un código de salida diferente de cero (`exit_code != 0`)
- **THEN** el arnés SHALL transicionar de inmediato al estado `HaltedOnError`, congelando las tareas restantes, conservando íntegro el vector `tasks` del plan aprobado y requiriendo acción explícita de corrección o reversión

#### Scenario: Recuperación tras freno por error preservando el plan o actualizando pasos pendientes
- **WHEN** el usuario reanuda desde `HaltedOnError` invocando `resume_from_error(fixed_command)` (o aplicando un sub-plan corregido para los pasos `step_index..N`)
- **THEN** el arnés SHALL conservar intactos los pasos previos completados `0..step_index`, actualizar el comando o cola restante re-auditada y restaurar `GuidedExecution` posicionado en `step_index`

#### Scenario: Marcado idempotente de tarea revertida
- **WHEN** un centinela `StepRollback` finaliza con `exit_code == 0` para `step_index`
- **THEN** `mark_step_rolled_back(step_index)` SHALL registrar `is_rolled_back = true` sobre esa tarea en el plan activo impidiendo marcarla o dispararla dos veces por error

#### Scenario: Congelamiento y reanudación ante desconexión de red (incluyendo paso en vuelo)
- **WHEN** la sesión SSH asociada se desconecta mientras el arnés se encuentra en `GuidedExecution`, `HaltedOnError` o `PlanAndDebate`
- **THEN** si existía un `PlanStep` en vuelo sin completar centinela, el arnés SHALL registrar primero `HaltedOnError` (`exit_code = -1`, desconexión durante ejecución) y acto seguido envolver el estado en `PausedOnDisconnect`, permitiendo al reconectar inspeccionar y decidir sin re-ejecutar a ciegas

#### Scenario: Rechazo de transición ilegal
- **WHEN** se intenta avanzar a `GuidedExecution` directamente desde `Idle` o `Reconnaissance` sin la previa aprobación del plan
- **THEN** el arnés SHALL rechazar la transición con un error de violación de fase y mantener el estado previo intacto

---

### Requirement: Contratos Tipados y Validación de Esquemas
Toda interacción de entrada y salida del agente SHALL modelarse con estructuras tipadas en Rust (`serde`), proveyendo además un sobre plano canónico (`AgentTurnWireEnvelope`) libre de `oneOf` en la raíz para compatibilidad universal con motores estrictos de JSON Schema y conversión validada hacia `AgentResponseKind`. Cada propuesta de tarea (`AgentTaskProposal`) SHALL incluir estado de ejecución (`is_done`) y estado de reversión (`#[serde(default)] pub is_rolled_back: bool`), y toda tarea mutante SHALL exigir justificación, nivel de riesgo, estrategia de respaldo y comando de reversión.

#### Scenario: Validación de propuesta de tarea completa y retrocompatibilidad de is_rolled_back
- **WHEN** el payload de una tarea o un `AgentTurnWireEnvelope` válido incluye `command`, `risk_level`, `why`, `backup_strategy` y `rollback_cmd` (omitiendo `is_rolled_back` en la salida del modelo)
- **THEN** el deserializador y conversor SHALL instanciar exitosamente `AgentTaskProposal` con `is_rolled_back = false` y `AgentResponseKind` válido

#### Scenario: Rechazo de tarea sin estrategia de reversión en cambios de configuración
- **WHEN** una tarea con nivel de riesgo `config_change` o superior omite el campo `rollback_cmd` o `backup_strategy` sin justificación
- **THEN** el validador de contrato SHALL rechazar la estructura y exigir la definición de contingencia

---

### Requirement: Comandos IPC Tauri para Control del Arnés
El backend SHALL exponer comandos Tauri asíncronos para inspeccionar el estado actual del arnés por terminal, aprobar el plan de Fase 2 (`approve_harness_plan_cmd`), ejecutar el paso actual de Fase 3 con edición opcional (`execute_ai_plan_step_cmd`), interrumpir comandos activos en PTY (`interrupt_ai_pty_step_cmd`), reanudar desde error (`retry_halted_step_cmd`), ejecutar rollback quirúrgico por paso (`execute_ai_rollback_step_cmd`), cancelar la misión activa (`cancel_harness_mission_cmd`), evaluar riesgos, gestionar preferencias, consultar salud del driver (`probe_ai_driver_health_cmd`), listar modelos disponibles (`list_ai_models_cmd`), ejecutar turnos del orquestador (`run_ai_harness_turn_cmd`) y cancelar turnos activos (`cancel_ai_turn_cmd`).

#### Scenario: Consulta de estado y preferencias por terminal
- **WHEN** el cliente invoca los comandos IPC correspondientes proporcionando el identificador de sesión
- **THEN** el backend SHALL responder con el estado actual de la FSM para esa terminal, la versión del protocolo y las preferencias almacenadas en SQLite

#### Scenario: Ejecución de turno, listado de modelos y cancelación por IPC
- **WHEN** el cliente invoca `probe_ai_driver_health_cmd`, `list_ai_models_cmd`, `run_ai_harness_turn_cmd` o `cancel_ai_turn_cmd`
- **THEN** el backend SHALL delegar en el driver configurado emitiendo eventos `ai-turn-progress` en tiempo real y permitiendo cancelar en caliente turnos en curso

#### Scenario: Ejecución guiada de Fase 3, interrupción Ctrl+C y rollback por IPC
- **WHEN** el operador invoca `execute_ai_plan_step_cmd`, `interrupt_ai_pty_step_cmd`, `retry_halted_step_cmd` o `execute_ai_rollback_step_cmd` desde `"neko-ai"`
- **THEN** el backend SHALL validar la fase y el índice de tarea en `HarnessRegistry`, auditar el riesgo con `CommandRiskGuard`, gestionar el centinela en `PtyTapRegistry` y emitir la fase actualizada a la interfaz
