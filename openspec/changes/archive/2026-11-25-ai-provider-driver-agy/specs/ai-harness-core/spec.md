## MODIFIED Requirements

### Requirement: Transiciones de Máquina de Estados del Arnés
El backend SHALL implementar una máquina de estados finita (`AgentHarnessFsm`) que restrinja estrictamente el ciclo de vida del agente en los estados `Idle`, `Reconnaissance`, `PlanAndDebate`, `GuidedExecution`, `HaltedOnError`, `PausedOnDisconnect` y `Completed`, manteniendo además un historial portátil de turnos (`turns: Vec<ConversationTurn>`) y un `provider_conversation_id` opcional que se invalida al cambiar de proveedor o cancelar la misión. Queda prohibido saltar a ejecución sin haber aprobado previamente un plan o permanecer en reconocimiento con comandos mutantes.

#### Scenario: Transición válida de Reconocimiento a Plan
- **WHEN** el arnés recibe la orden de iniciar diagnóstico para un objetivo y completa la recolección de lecturas autorizadas
- **THEN** la máquina de estados transiciona de `Reconnaissance` a `PlanAndDebate`, bloqueando toda ejecución en el servidor mientras se debate el plan

#### Scenario: Freno automático ante salida de error en ejecución
- **WHEN** durante el estado `GuidedExecution` un comando ejecutado devuelve un código de salida diferente de cero (`exit_code != 0`)
- **THEN** el arnés SHALL transicionar de inmediato al estado `HaltedOnError`, congelando las tareas restantes, conservando íntegro el vector `tasks` del plan aprobado y requiriendo acción explícita de corrección o reversión

#### Scenario: Recuperación tras freno por error preservando el plan
- **WHEN** el usuario reanuda desde `HaltedOnError` invocando `resume_from_error(fixed_command)`
- **THEN** el arnés SHALL restaurar todas las tareas del plan aprobado en `GuidedExecution` aplicando `fixed_command` sobre `tasks[step_index]`

#### Scenario: Congelamiento y reanudación ante desconexión de red
- **WHEN** la sesión SSH asociada se desconecta mientras el arnés se encuentra en `GuidedExecution` o `PlanAndDebate`
- **THEN** el arnés SHALL transicionar a `PausedOnDisconnect` conservando el plan y las tareas completadas en memoria, permitiendo al reconectar la sesión reanudar la misión sin pérdida de avance

#### Scenario: Rechazo de transición ilegal
- **WHEN** se intenta avanzar a `GuidedExecution` directamente desde `Idle` o `Reconnaissance` sin la previa aprobación del plan
- **THEN** el arnés SHALL rechazar la transición con un error de violación de fase y mantener el estado previo intacto

---

### Requirement: Validador Estricto de Solo Lectura (ReadOnlyEnforcer)
Durante la fase de `Reconnaissance`, el backend SHALL validar mediante análisis léxico basado en tokens conscientes de comillas que todo comando propuesto sea estrictamente de inspección y no cause mutaciones en el sistema remoto.

#### Scenario: Aprobación de comandos puros de diagnóstico
- **WHEN** el agente propone comandos como `uname -a`, `cat /etc/os-release`, `free -h`, `ss -tulpn`, `df -h` o búsquedas como `grep -i "error" /var/log/syslog`
- **THEN** el validador SHALL clasificar el comando como permitido en fase de reconocimiento

#### Scenario: Bloqueo de redirecciones de escritura y mutaciones
- **WHEN** un comando en reconocimiento contiene redirecciones fuera de comillas (`>`, `>>`), `tee` directo o en tubería, tuberías hacia intérpretes con o sin espacios/prefijos (`|bash`, `|  sh`, `| sudo bash`), `find` con `-delete`/`-exec`, `crontab`, gestores de paquetes/contenedores, creación o borrado de archivos (`rm`, `touch`, `mkdir`, `mv`, `cp`, `chmod`, `chown`) o control de servicios (`systemctl`, `service`, `kill`)
- **THEN** el validador SHALL rechazar el comando marcándolo como no conforme para la fase de reconocimiento, impidiendo su presentación como lectura segura

---

### Requirement: Auditor Estático de Riesgo en Cuatro Etapas (CommandRiskGuard)
El sistema SHALL auditar y clasificar el riesgo operativo de cada comando propuesto mediante un pipeline léxico determinista de 4 etapas evaluando tokens reales por segmento: Tokenizador/Segmentador por operadores consciente de comillas, Detector de tuberías hacia intérpretes de shell, Análisis de verbos y subcomandos/flags por token exacto, y Auditor de operadores de redirección. La salida SHALL categorizarse en cuatro niveles: `read_only` (verde), `config_change` (amarillo), `service_impact` (naranja) y `destructive` (rojo).

#### Scenario: Elevación forzosa ante tuberías a intérpretes de shell
- **WHEN** un comando incluye tuberías hacia intérpretes con cualquier espaciado o prefijo como `| bash`, `|bash`, `| sudo sh` o `|/usr/bin/python3`
- **THEN** `CommandRiskGuard` SHALL clasificar el comando automáticamente como `destructive` (🔴) sin importar la etiqueta propuesta por la IA

#### Scenario: Elevación por operadores de redirección sobre comandos de lectura
- **WHEN** un comando con verbo de lectura (p. ej. `cat` o `dmesg`) incluye un operador de redirección hacia archivo fuera de comillas (`>`, `>>` o `tee`)
- **THEN** el auditor SHALL elevar inmediatamente la severidad de `read_only` a `config_change` (🟡) por implicar escritura en el sistema de archivos

#### Scenario: Identificación de impacto en servicios del sistema
- **WHEN** un comando manipula el ciclo de vida de demonios o servicios en vivo (incluyendo nombres que contengan la subcadena `status`, p. ej. `systemctl restart status-api`) o reglas de red activa (`ufw enable|disable`, `iptables`)
- **THEN** el auditor SHALL evaluar el subcomando real (`restart`) y clasificar el riesgo en `service_impact` (🟠), reservando `read_only` únicamente cuando el subcomando sea `status`, `is-active` o `is-enabled`

---

### Requirement: Sanitizador Preventivo de Secretos (SecretRedactor)
El sistema SHALL sanitizar todo fragmento de texto o buffer capturado antes de persistirlo o enviarlo fuera del runtime de la terminal, enmascarando patrones de información sensible mediante búsqueda ASCII *case-insensitive* por bytes segura frente a caracteres UTF-8 multibyte.

#### Scenario: Enmascaramiento de llaves privadas PEM
- **WHEN** el texto capturado contiene bloques delimitados por `-----BEGIN [A-Z ]*PRIVATE KEY-----` y `-----END [A-Z ]*PRIVATE KEY-----`
- **THEN** el sanitizador SHALL reemplazar todo el bloque por el marcador `[REDACTED_PRIVATE_KEY]`

#### Scenario: Enmascaramiento de tokens y variables de credenciales
- **WHEN** el texto contiene asignaciones del tipo `PASSWORD=...`, `TOKEN=...`, `SECRET=...`, encabezados `Authorization: Bearer ...` o caracteres Unicode multibyte cuyo tamaño en bytes varía al cambiar de caja (`ß`, `İ`)
- **THEN** el sanitizador SHALL sustituir los valores sensibles por `[REDACTED_SECRET]` preservando el nombre de la variable sin sufrir `panic!` UTF-8

---

### Requirement: Contratos Tipados y Validación de Esquemas
Toda interacción de entrada y salida del agente SHALL modelarse con estructuras tipadas en Rust (`serde`), proveyendo además un sobre plano canónico (`AgentTurnWireEnvelope`) libre de `oneOf` en la raíz para compatibilidad universal con motores estrictos de JSON Schema y conversión validada hacia `AgentResponseKind`. Cada propuesta de tarea mutante SHALL incluir obligatoriamente justificación, nivel de riesgo, estrategia de respaldo y comando de reversión.

#### Scenario: Validación de propuesta de tarea completa
- **WHEN** el payload de una tarea o un `AgentTurnWireEnvelope` válido incluye `command`, `risk_level`, `why`, `backup_strategy` y `rollback_cmd`
- **THEN** el deserializador y conversor SHALL instanciar exitosamente `AgentTaskProposal` y `AgentResponseKind`

#### Scenario: Rechazo de tarea sin estrategia de reversión en cambios de configuración
- **WHEN** una tarea con nivel de riesgo `config_change` o superior omite el campo `rollback_cmd` o `backup_strategy` sin justificación
- **THEN** el validador de contrato SHALL rechazar la estructura y exigir la definición de contingencia

---

### Requirement: Ensamblador de Prompt en Dos Capas y Persistencia
El sistema SHALL ensamblar el contexto inicial del agente combinando una Capa 1 inmutable en código (Constitución del Arnés) con una Capa 2 personalizable almacenada en SQLite `app_preferences` mediante campos universales desacoplados del proveedor (`execution_mode`, `active_driver`, `selected_model`, `operator_persona`, `operator_infra_rules` y `driver_options: HashMap<String, String>`).

#### Scenario: Ensamblado determinista del prompt de sistema
- **WHEN** el arnés genera las instrucciones para un turno agéntico
- **THEN** SHALL anteponer la Constitución fija (mandatos de fases, higiene de PTY, esquema JSON) e incorporar a continuación las directivas del operador (`operator_persona` y `operator_infra_rules`)

#### Scenario: Valores por defecto y restauración de preferencias
- **WHEN** la base de datos no contiene directivas configuradas o el usuario solicita restaurar valores
- **THEN** el sistema SHALL proveer directivas predeterminadas profesionales, configurar `execution_mode = Strict`, `active_driver = "agy"`, `selected_model = ""` y `driver_options` vacío en `app_preferences`

---

### Requirement: Comandos IPC Tauri para Control del Arnés
El backend SHALL exponer comandos Tauri asíncronos para inspeccionar el estado actual del arnés por terminal, evaluar riesgos, gestionar preferencias, consultar salud del driver (`probe_ai_driver_health_cmd`), listar modelos disponibles (`list_ai_models_cmd`), ejecutar turnos del orquestador (`run_ai_harness_turn_cmd`) y cancelar turnos activos (`cancel_ai_turn_cmd`).

#### Scenario: Consulta de estado y preferencias por terminal
- **WHEN** el cliente invoca los comandos IPC correspondientes proporcionando el identificador de sesión
- **THEN** el backend SHALL responder con el estado actual de la FSM para esa terminal, la versión del protocolo y las preferencias almacenadas en SQLite

#### Scenario: Ejecución de turno, listado de modelos y cancelación por IPC
- **WHEN** el cliente invoca `probe_ai_driver_health_cmd`, `list_ai_models_cmd`, `run_ai_harness_turn_cmd` o `cancel_ai_turn_cmd`
- **THEN** el backend SHALL delegar en el driver configurado emitiendo eventos `ai-turn-progress` en tiempo real y permitiendo cancelar en caliente turnos en curso
