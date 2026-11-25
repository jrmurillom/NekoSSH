## MODIFIED Requirements

### Requirement: Ensamblador de Prompt en Dos Capas y Persistencia
El sistema SHALL ensamblar el contexto inicial del agente combinando una Capa 1 inmutable en código (Constitución del Arnés) con una Capa 2 personalizable almacenada en SQLite `app_preferences` mediante campos universales desacoplados del proveedor (`execution_mode`, `active_driver`, `selected_model`, `context_max_lines`, `operator_persona`, `operator_infra_rules` y `driver_options: HashMap<String, String>`).

#### Scenario: Ensamblado determinista del prompt de sistema
- **WHEN** el arnés genera las instrucciones para un turno agéntico
- **THEN** SHALL anteponer la Constitución fija (mandatos de fases, higiene de PTY, esquema JSON) e incorporar a continuación las directivas del operador (`operator_persona` y `operator_infra_rules`)

#### Scenario: Valores por defecto y restauración de preferencias
- **WHEN** la base de datos no contiene directivas configuradas o el usuario solicita restaurar valores
- **THEN** el sistema SHALL proveer directivas predeterminadas profesionales, configurar `execution_mode = Strict`, `active_driver = "agy"`, `selected_model = ""`, `context_max_lines = 200` y `driver_options` vacío en `app_preferences`

---

### Requirement: Comandos IPC Tauri para Control del Arnés
El backend SHALL exponer comandos Tauri asíncronos para inspeccionar el estado actual del arnés por terminal, aprobar el plan de Fase 2 (`approve_harness_plan_cmd`), cancelar la misión activa (`cancel_harness_mission_cmd`), evaluar riesgos, gestionar preferencias, consultar salud del driver (`probe_ai_driver_health_cmd`), listar modelos disponibles (`list_ai_models_cmd`), ejecutar turnos del orquestador (`run_ai_harness_turn_cmd`) y cancelar turnos activos (`cancel_ai_turn_cmd`).

#### Scenario: Consulta de estado y preferencias por terminal
- **WHEN** el cliente invoca los comandos IPC correspondientes proporcionando el identificador de sesión
- **THEN** el backend SHALL responder con el estado actual de la FSM para esa terminal, la versión del protocolo y las preferencias almacenadas en SQLite

#### Scenario: Ejecución de turno, listado de modelos y cancelación por IPC
- **WHEN** el cliente invoca `probe_ai_driver_health_cmd`, `list_ai_models_cmd`, `run_ai_harness_turn_cmd` o `cancel_ai_turn_cmd`
- **THEN** el backend SHALL delegar en el driver configurado emitiendo eventos `ai-turn-progress` en tiempo real y permitiendo cancelar en caliente turnos en curso

#### Scenario: Aprobación formal del plan y cancelación de misión por IPC
- **WHEN** el operador pulsa **Aprobar Plan** en Fase 2 o **Cancelar Misión** en la ventana `"neko-ai"`
- **THEN** `approve_harness_plan_cmd` SHALL validar y transicionar la FSM a `GuidedExecution`, mientras que `cancel_harness_mission_cmd` SHALL abortar cualquier turno activo y restablecer la FSM a `Idle`
