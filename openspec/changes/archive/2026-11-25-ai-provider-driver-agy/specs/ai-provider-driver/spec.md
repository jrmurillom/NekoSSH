## ADDED Requirements

### Requirement: Puerto Agnóstico de Proveedor y Descubrimiento de Modelos (AiProviderDriver)
El backend SHALL definir un puerto abstracto (`trait AiProviderDriver`) completamente desacoplado de detalles de transporte (CLI local, HTTP REST o modelos locales), exponiendo tres operaciones canónicas: diagnóstico de salud (`probe_health`), catálogo dinámico de modelos disponibles (`list_models`) y ejecución de turno estructurado (`execute_turn`) con notificación de progreso en tiempo real y cancelación cooperativa.

#### Scenario: Listado dinámico de modelos para selector rápido
- **WHEN** el cliente consulta los modelos disponibles del proveedor activo mediante `list_models`
- **THEN** el driver SHALL retornar una colección ordenada de descriptores `AiModelDescriptor { id, label }` sin exponer detalles internos de CLI o endpoints REST

#### Scenario: Intercambiabilidad de drivers sin alterar el arnés
- **WHEN** el orquestador ejecuta un turno usando cualquier implementación de `AiProviderDriver` (`AgyCliDriver` o `FakeAiDriver`) enviando `AgentTurnRequest` (con `selected_model` opcional, historial `turns`, `provider_conversation_id` y `json_schema` plano)
- **THEN** SHALL recibir fragmentos de progreso vía callback y un `AgentTurnOutcome` con `AgentResponseKind` validado sin acoplar el arnés a banderas específicas de `agy`

---

### Requirement: Adaptador Aislado AgyCliDriver (Cerebro JSON en Sandbox)
El adaptador `AgyCliDriver` SHALL encapsular en exclusiva todos los detalles del ejecutable `agy`, invocándolo únicamente como motor de razonamiento estructurado sin acceso a herramientas de edición ni archivos del equipo anfitrión.

#### Scenario: Banderas de confinamiento, esquema en archivo y prompt acotado
- **WHEN** `AgyCliDriver` ejecuta un turno contra `agy`
- **THEN** SHALL establecer `current_dir` en `{appData}/ai-sandbox/`, escribir el esquema JSON en un archivo temporal con limpieza RAII dentro del sandbox, acotar de forma segura el payload pasado a `--print` para respetar los límites de `CreateProcess` en Windows manteniendo `stdin` cerrado, e incluir `--output-format stream-json`, `--sandbox`, `--disable-slash-commands` y `--json-schema <ruta_esquema>`, omitiendo estrictamente `--dangerously-skip-permissions`

#### Scenario: Selección de modelo, reanudación de conversación y supresión de consola en Windows
- **WHEN** la solicitud especifica `selected_model` o `provider_conversation_id` en sistema operativo Windows
- **THEN** `AgyCliDriver` SHALL anexar `--model <id>` y `--conversation <id>` según corresponda y lanzar tanto `agy models` como `agy -p` con la bandera `CREATE_NO_WINDOW` (`0x08000000`)

#### Scenario: Parseo de catálogo real desde `agy models`
- **WHEN** `AgyCliDriver::list_models` procesa la salida de `agy models`
- **THEN** SHALL ignorar líneas informativas de encabezado (p. ej. `Fetching available models...`) y extraer cada par `<id>\t<label>` válido como `AiModelDescriptor`

---

### Requirement: Streaming NDJSON, Diagnóstico de Salud y Errores Accionables
`AgyCliDriver` SHALL procesar línea por línea el flujo `--output-format stream-json` emitiendo avances hacia el callback de progreso y mapeando fallas operativas a errores legibles en español latino.

#### Scenario: Emisión de progreso y extracción del resultado final estructurado
- **WHEN** `agy` emite líneas NDJSON de razonamiento seguidas del evento final sujeto a `--json-schema`
- **THEN** el driver SHALL notificar los eventos intermedios de progreso y deserializar el objeto final mediante `AgentTurnWireEnvelope` hacia `AgentResponseKind`

#### Scenario: Binario ausente o error de autenticación
- **WHEN** el ejecutable `agy` no existe en `driver_options["agy_binary_path"]` ni en `PATH`, o reporta sesión no autenticada en `stderr`
- **THEN** el driver SHALL retornar `Unavailable` / `AiDriverError::AuthenticationRequired` con instrucciones claras en español latino

---

### Requirement: Cancelación Cooperativa, Timeout y Ejecución Sin Bloqueo de Mutex
El orquestador y `AiProviderDriver` SHALL permitir abortar cualquier turno activo de inmediato y NUNCA retener el `Mutex` global de `HarnessRegistry` mientras espera la respuesta del proveedor.

#### Scenario: Consulta concurrente y cancelación inmediata de turno en vuelo
- **WHEN** un turno de IA se encuentra en ejecución y el cliente consulta el estado de la sesión o invoca `cancel_ai_turn_cmd`
- **THEN** `HarnessRegistry` SHALL responder sin bloqueo de mutex, señalizar el `TurnCancellationToken`, terminar el subproceso hijo de inmediato y descartar cualquier escritura tardía mediante control de época/generación de la sesión

#### Scenario: Limpieza global al cerrar la aplicación
- **WHEN** la aplicación recibe evento de salida (`RunEvent::ExitRequested` / `RunEvent::Exit`)
- **THEN** `AiTurnCancelRegistry::cancel_all` SHALL abortar todos los turnos activos evitando procesos huérfanos

---

### Requirement: Orquestador Integrado con Guardrails y FakeAiDriver
`HarnessOrchestrator` SHALL coordinar `PromptAssembler`, `SecretRedactor`, `AiProviderDriver`, `ReadOnlyEnforcer` y `CommandRiskGuard`, contando con `FakeAiDriver` para verificar deterministamente todos los flujos en memoria.

#### Scenario: Rechazo de mutación en Fase 1 y elevación de riesgo en propuestas del driver
- **WHEN** el driver retorna un comando mutante durante `Reconnaissance` o subestima el riesgo de un comando en cualquier fase
- **THEN** el orquestador SHALL bloquear la mutación en Fase 1 mediante `ReadOnlyEnforcer` y aplicar siempre `CommandRiskGuard::assess_and_elevate` antes de persistir o retornar el turno
