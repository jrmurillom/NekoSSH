## Context

Tras completar el Hito 1 e inspeccionar tanto el binario real `agy.EXE v1.5.16` en Windows como el código existente en `app/src-tauri/src/ai/`, el Hito 2 cumple un doble propósito:
1. Conectar el núcleo determinista con un puerto hexagonal multi-LLM (`trait AiProviderDriver`) y su primer adaptador aislado (`AgyCliDriver`) con descubrimiento dinámico de modelos y progreso en tiempo real (`stream-json`).
2. Remediar los 5 hallazgos detectados en la auditoría arquitectónica antes de construir la ventana secundaria `neko-ai` del Hito 3.

## Goals / Non-Goals

**Goals:**
- Definir `trait AiProviderDriver` 100% agnóstico de proveedor con `probe_health`, `list_models` y `execute_turn` (streaming de progreso + `TurnCancellationToken`).
- Implementar `AgyCliDriver` confinado dentro de `src/ai/drivers/agy.rs` usando las capacidades verificadas de `agy 1.5.16`: `agy models`, `--print`, `--output-format stream-json`, `--mode plan`, `--sandbox`, `--disable-slash-commands`, `--json-schema <archivo_sandbox>`, `--model`, `--conversation`, prompt vía `stdin` y `CREATE_NO_WINDOW` en Windows.
- Implementar `FakeAiDriver` para cobertura TDD determinista sin red ni credenciales.
- Corregir los 5 hallazgos de auditoría:
  1. Preservar `tasks` y aplicar `fixed_command` en `HaltedOnError` → `resume_from_error`.
  2. Endurecer `risk_guard.rs` por evaluación de tokens exactos y eliminar el riesgo de `panic!` UTF-8 en `redactor.rs`.
  3. Mantener historial portátil `turns: Vec<ConversationTurn>` junto a `provider_conversation_id` en `AgentHarnessFsm`.
  4. Definir `AgentTurnWireEnvelope` plano (sin `oneOf` raíz) para máxima compatibilidad de JSON Schema entre `agy`, OpenAI, Gemini API, Claude y Ollama.
  5. Ejecutar turnos en `HarnessOrchestrator` sin retener el `Mutex` de `HarnessRegistry` (patrón *Snapshot → Unlock → Inferencia → Relock con verificación de época*).
- Mantener `AiPreferencesDto` libre de campos acoplados a `agy` usando `active_driver`, `selected_model` y `driver_options: HashMap<String, String>`.

**Non-Goals:**
- Drivers HTTP directos (OpenAI, Anthropic, Gemini API, Ollama); quedan listos para enchufarse en futuros changes sin tocar la FSM ni el DTO de preferencias.
- Ventana nativa `neko-ai` y UI de selección (Hito 3).
- Inyección automática en PTY SSH y centinela POSIX (Hito 4).

## Decisions

### Decisión 1: Frontera Hexagonal Estricta (`AiPreferencesDto` + `trait AiProviderDriver`)
- **Razón**: Evitar que conceptos exclusivos de `agy` (`--mode plan`, `--disable-slash-commands` o `agy_binary_path`) contaminen el dominio común. `AiPreferencesDto` expone `active_driver`, `selected_model` (universal para el `<select>` de modelos) y un mapa `driver_options: HashMap<String, String>` serializado en JSON dentro de `app_preferences`. Solo `AgyCliDriver` lee `driver_options.get("agy_binary_path")`.
- **Alternativa descartada**: Agregar columnas o campos por cada futuro proveedor en `AiPreferencesDto`.

### Decisión 2: `--output-format stream-json` + `--json-schema <archivo>` + `--print` acotado en `AgyCliDriver`
- **Razón**: Verificado en vivo contra `agy.EXE 1.5.16`:
  - `stream-json` emite eventos `step_update.text_delta` en tiempo real y entrega `result.structured_output` válido contra `--json-schema`.
  - En `agy 1.5.16`, `--print` toma su argumento de línea de comandos como prompt (ignorando `stdin` plano) y `--mode plan` no tiene efecto cuando `--disable-slash-commands` está activo; por ello `AgyCliDriver` escribe el esquema en `{appData}/ai-sandbox/turn-schema-<uuid>.json` (con borrado RAII), aplica `--sandbox` + `--disable-slash-commands`, cierra `stdin` (`Stdio::null()`) y acota el payload de `--print` a `MAX_CLI_PROMPT_CHARS = 20_000` (bajo el límite de 32,767 chars de `CreateProcessW` en Windows).

### Decisión 3: Sobre JSON Plano (`AgentTurnWireEnvelope`) para Compatibilidad Multi-LLM
- **Razón**: Los validadores estrictos de *Structured Outputs* (OpenAI `strict: true`, Gemini `responseSchema` y `--json-schema`) rechazan frecuentemente uniones polimórficas `oneOf` en la raíz. Un objeto plano con `kind: "propose_step" | "propose_plan" | "reply_only" | "goal_reached"`, `message: String`, `step: Option<WireStep>` y `plan: Option<Vec<AgentTaskProposal>>` es aceptado universalmente y se transforma en Rust mediante `TryFrom<AgentTurnWireEnvelope> for AgentResponseKind`.

### Decisión 4: Historial Dual Portátil (`turns` + `provider_conversation_id`)
- **Razón**: Permite aprovechar `--conversation <id>` cuando el driver es `agy`, pero mantiene una bitácora acotada `Vec<ConversationTurn>` en `AgentHarnessFsm` para que cualquier driver HTTP *stateless* futuro (o un cambio de proveedor/modelo en caliente) disponga del contexto completo de la misión. Si `active_driver` cambia, `provider_conversation_id` se limpia automáticamente.

### Decisión 5: Concurrencia por Época (`generation: u64`) sin Retener `Mutex`
- **Razón**: `AgentHarnessFsm` incrementa un contador `generation: u64` al iniciar, cancelar o resetear misión. Antes de invocar al driver, el orquestador toma snapshot bajo lock (<1 ms) y libera el `Mutex`. Durante el turno registra un `TurnCancellationToken` en `AiTurnCancelRegistry`. Al terminar el driver, re-adquiere el lock y solo aplica la transición si `fsm.generation == snapshot_generation`, descartando resultados obsoletos si el usuario canceló a mitad de vuelo.

## Risks / Trade-offs

- **[Riesgo] Variaciones de formato en líneas intermedias de `stream-json` entre versiones de `agy`** → *Mitigación*: Parser línea a línea defensivo que extrae fragmentos de progreso conocidos e identifica de forma determinista el objeto final válido contra `AgentTurnWireEnvelope` (con fallback a extracción del último objeto JSON completo en `stdout`).
- **[Riesgo] Latencia al listar modelos (`agy models` tarda ~1–2 s consultando cuenta)** → *Mitigación*: Comando asíncrono no bloqueante `list_ai_models_cmd` ejecutable bajo demanda al abrir Ajustes o cambiar proveedor, manteniendo siempre disponible la opción `"Predeterminado del proveedor"` (`""`).

## Migration Plan

Retrocompatible sobre SQLite `app_preferences`: `ai_active_driver` (`"agy"`), `ai_selected_model` (`""`) y `ai_driver_options_json` (`"{}"`) usan defaults limpios cuando no existen filas previas.
