## Why

Con el núcleo determinista del arnés creado en el Hito 1, el sistema requiere conectarse a su primer motor de razonamiento productivo a través de un puerto agnóstico multi-LLM (`trait AiProviderDriver`) que permita descubrir modelos disponibles dinámicamente, transmitir progreso en tiempo real (`stream-json`/SSE) y aislar la implementación inicial sobre **`agy` (`v1.5.16`)** como un "Cerebro JSON" sin acceso al sistema local. Asimismo, la auditoría profunda del Hito 1 identificó 5 ajustes críticos de integridad, seguridad léxica, manejo UTF-8, portabilidad de historial y concurrencia que deben quedar saneados antes de construir la interfaz gráfica del Hito 3.

## What Changes

- **Puerto Agnóstico Multi-Proveedor (`src/ai/driver.rs`)**:
  - Definición de `trait AiProviderDriver` con 3 operaciones universales: `probe_health`, `list_models` (alimentando el selector rápido de modelos en UI) y `execute_turn` con callback de progreso en vivo y `TurnCancellationToken`.
  - Definición de un sobre de transporte plano (`AgentTurnWireEnvelope`) para `--json-schema` / *Structured Outputs* 100% compatible con motores estrictos (`agy`, OpenAI, Gemini API, Claude, Ollama) sin depender de `oneOf` polimórficos en la raíz.
- **Adaptador Aislado `AgyCliDriver` (`src/ai/drivers/agy.rs`)**:
  - Descubrimiento de modelos reales vía `agy models` (`<id>\t<label>`) y diagnóstico `probe_health` en español latino.
  - Ejecución *headless* con `--output-format stream-json`, `--mode plan`, `--sandbox`, `--disable-slash-commands`, `--json-schema <archivo_en_sandbox.json>`, `--model <id>` opcional, `--conversation <id>` opcional y envío del prompt ensamblado por **`stdin`** para evitar límites de longitud de línea de comandos en Windows.
  - Confinamiento de `current_dir` en `{appData}/ai-sandbox/`, prohibición total de `--dangerously-skip-permissions` y `CREATE_NO_WINDOW` (`0x08000000`) en Windows.
- **`FakeAiDriver` Determinista (`src/ai/drivers/fake.rs`)**: Simulador en memoria de modelos, eventos de progreso, turnos estructurados, errores y cancelación para TDD sin red ni credenciales.
- **Remediación de los 5 Hallazgos de Auditoría sobre `ai-harness-core`**:
  1. **`fsm.rs` (`HaltedOnError` & Historial Portátil)**: Preservar el vector completo `tasks` en `HaltedOnError`, aplicar `fixed_command` en `resume_from_error`, y mantener tanto `turns: Vec<ConversationTurn>` (para futuros drivers HTTP *stateless*) como `provider_conversation_id: Option<String>` (invalidado al cambiar de proveedor).
  2. **`risk_guard.rs` & `redactor.rs`**: Detección de tuberías a intérpretes por tokens reales sin importar espaciado (`|bash`, `| sudo sh`), evaluación de subcomandos por token exacto (corrigiendo falsos positivos como `systemctl restart status-api`), ampliación de verbos mutantes (`tee`, `crontab`, `find -delete`/`-exec`, contenedores) y búsqueda ASCII *case-insensitive* por bytes en `SecretRedactor` libre de `panic!` con caracteres UTF-8 multibyte.
  3. **`preferences.rs` Desacoplada**: Extender `AiPreferencesDto` únicamente con campos universales (`active_driver`, `selected_model` y `driver_options: HashMap<String, String>`, donde reside opcionalmente `agy_binary_path` sin acoplar el DTO común a un CLI concreto).
  4. **Orquestador sin Bloqueo de Mutex (`orchestrator.rs` & `commands.rs`)**: Patrón *Snapshot → Unlock → Inferencia cancelable en background → Relock & Commit con verificación de época*, evitando congelar consultas IPC durante turnos de 10–25 s, emitiendo eventos de progreso Tauri (`ai-turn-progress`) y exponiendo `probe_ai_driver_health_cmd`, `list_ai_models_cmd`, `run_ai_harness_turn_cmd` y `cancel_ai_turn_cmd`.

## Capabilities

### New Capabilities
- `ai-provider-driver`: Puerto agnóstico `AiProviderDriver` (`probe_health`, `list_models`, `execute_turn` con streaming y cancelación), adaptador blindado `AgyCliDriver`, `FakeAiDriver` y orquestación concurrente sin bloqueo.

### Modified Capabilities
- `ai-harness-core`: Preservación de `tasks` al recuperar de `HaltedOnError`, historial dual portátil (`turns` + `provider_conversation_id`), endurecimiento por tokens en `ReadOnlyEnforcer`/`CommandRiskGuard`, seguridad UTF-8 en `SecretRedactor`, sobre plano `AgentTurnWireEnvelope`, preferencias universales (`active_driver`, `selected_model`, `driver_options`) y nuevos comandos IPC.

## Impact

- **Backend Rust (`app/src-tauri/src/ai/`)**: Nuevos módulos `driver.rs`, `drivers/mod.rs`, `drivers/agy.rs`, `drivers/fake.rs`, `orchestrator.rs`; actualización enfocada en `fsm.rs`, `schema.rs`, `risk_guard.rs`, `redactor.rs`, `preferences.rs`, `commands.rs` y `mod.rs`; registro mínimo en `src/lib.rs` (incluyendo `AiTurnCancelRegistry::cancel_all()` al salir).
- **Persistencia SQLite (`app_preferences`)**: Claves `ai_active_driver`, `ai_selected_model` y `ai_driver_options_json` con valores predeterminados retrocompatibles.
- **Dependencias**: Cero crates externos nuevos.
