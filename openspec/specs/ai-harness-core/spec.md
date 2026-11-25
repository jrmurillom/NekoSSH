# ai-harness-core Specification

## Purpose
TBD - created by archiving change ai-harness-core. Update Purpose after archive.
## Requirements
### Requirement: Transiciones de Máquina de Estados del Arnés
El backend SHALL gestionar por cada sesión SSH un contexto conversacional continuo (`AgentHarnessFsm`) con soporte para el ciclo operativo adaptativo de 4 fases y estados formales (`Idle`, `Reconnaissance`, `PlanAndDebate`, `GuidedExecution`, `HaltedOnError`, `PausedOnDisconnect`, `Completed`), conservando la memoria portátil de turnos (`turns: Vec<ConversationTurn>`), el `provider_conversation_id` y el último `rollback_cmd` disponible incluso después de responder una consulta o concluir una intervención, salvo cuando el operador invoque explícitamente **Limpiar conversación**.

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

#### Scenario: Consulta informativa directa sin burocracia y conservación de memoria
- **WHEN** el operador formula una pregunta diagnóstica (p. ej. *"¿qué OS tenemos?"*) y el agente recolecta la lectura y entrega la respuesta técnica
- **THEN** la sesión SHALL presentar la respuesta enriquecida y quedar lista para la siguiente instrucción **conservando intacta** la memoria de lo descubierto en esa sesión SSH

#### Scenario: Limpieza explícita de conversación a petición del operador
- **WHEN** el operador pulsa explícitamente **Limpiar conversación** (`clear_ai_conversation_cmd`)
- **THEN** el backend SHALL purgar `turns`, invalidar `provider_conversation_id`, incrementar `generation` y restablecer la fase a `Idle`

---

### Requirement: Modos de Ejecución (Estricto vs Flexible)
El sistema SHALL soportar dos modos de ejecución configurables por el usuario persistidos en SQLite (`ai_execution_mode`): `Strict` (predeterminado) y `Flexible`. Queda prohibido cualquier modo "turbo" o ejecución desatendida sin supervisión.

#### Scenario: Modo Estricto solicita confirmación para toda lectura
- **WHEN** el arnés opera en modo `Strict` y se propone un comando clasificado como `read_only` (🟢)
- **THEN** el sistema SHALL pausar y requerir autorización humana explícita antes de inyectar el comando

#### Scenario: Modo Flexible auto-autoriza únicamente lecturas puras verificadas
- **WHEN** el arnés opera en modo `Flexible` y se propone un comando verificado como `read_only` puro sin redirecciones ni advertencias
- **THEN** el sistema SHALL permitir la ejecución automática del comando de lectura para acelerar el diagnóstico

#### Scenario: Modo Flexible frena obligatoriamente ante cambios o riesgo
- **WHEN** en modo `Flexible` se propone un comando clasificado como `config_change` (🟡), `service_impact` (🟠) o `destructive` (🔴)
- **THEN** el sistema SHALL detenerse y exigir autorización humana obligatoria con tarjeta HITL

---

### Requirement: Validador Estricto de Solo Lectura (ReadOnlyEnforcer)
Durante la fase de `Reconnaissance`, el backend SHALL validar mediante análisis léxico basado en tokens conscientes de comillas que todo comando propuesto sea estrictamente de inspección y no cause mutaciones en el sistema remoto, reconociendo búsquedas POSIX inocuas (`command -v`, `command -V`, `which`, `type`), redirecciones estándar inocuas (`2>&1`, `2>/dev/null`), flags `--version` y subcomandos compuestos de solo lectura en herramientas de contenedores y servicios.

#### Scenario: Aprobación de comandos puros de diagnóstico
- **WHEN** el agente propone comandos como `uname -a`, `cat /etc/os-release`, `free -h`, `ss -tulpn`, `df -h`, `grep -i "error" /var/log/syslog` o diagnósticos compuestos como `command -v docker 2>&1; systemctl is-active docker containerd 2>&1; docker ps 2>&1; ps -ef | grep -iE 'dockerd|containerd' | grep -v grep 2>&1; true`
- **THEN** el validador SHALL clasificar el comando como permitido (`read_only`) sin falsos positivos ni advertencias espurias de modificación

#### Scenario: Bloqueo de redirecciones de escritura y mutaciones
- **WHEN** un comando en reconocimiento contiene redirecciones fuera de comillas hacia archivos reales (`>`, `>>`), `tee` directo o en tubería, tuberías hacia intérpretes con o sin espacios/prefijos (`|bash`, `|  sh`, `| sudo bash`), `find` con `-delete`/`-exec`, `crontab`, gestores de paquetes/contenedores con acciones mutantes (incluyendo `command rm` o `command -p systemctl restart`), creación o borrado de archivos (`rm`, `touch`, `mkdir`, `mv`, `cp`, `chmod`, `chown`) o control de servicios (`systemctl`, `service`, `kill`)
- **THEN** el validador SHALL rechazar el comando marcándolo como no conforme para la fase de reconocimiento, impidiendo su presentación como lectura segura

#### Scenario: Reconocimiento de supresión estándar POSIX 2>/dev/null y 2>&1 como lectura pura
- **WHEN** el agente propone comandos diagnósticos como `cat /etc/os-release 2>/dev/null || uname -a` o `nginx -t 2>&1`
- **THEN** `ReadOnlyEnforcer` y `CommandRiskGuard` SHALL clasificarlos como `read_only` (`🟢`) sin emitir falsos positivos de redirección de escritura

#### Scenario: Degradación segura a autorización humana sin bloquear el turno
- **WHEN** el agente propone un comando que utiliza un binario propio no catalogado, sustitución `$()`, redirección real a archivo (`> /tmp/out`) o modificación de sistema
- **THEN** el backend **NO** abortará el turno con error; desactivará `auto_approved_read_only`, asignará el nivel de riesgo auditado junto con `audit_warning` explicativo en español claro y presentará la tarjeta al operador para que decida si autoriza, edita u omite

---

### Requirement: Auditor Estático de Riesgo en Cuatro Etapas (CommandRiskGuard)
El sistema SHALL auditar y clasificar el riesgo operativo de cada comando propuesto mediante un pipeline léxico determinista de 4 etapas evaluando tokens reales por segmento: Tokenizador/Segmentador por operadores consciente de comillas, Detector de tuberías hacia intérpretes de shell, Análisis de verbos y subcomandos/flags por token exacto (distinguiendo `command -v`/`-V` de ejecuciones `command <bin>`), y Auditor de operadores de redirección. La salida SHALL categorizarse en cuatro niveles (`read_only`, `config_change`, `service_impact`, `destructive`) y proveer en `evaluate_proposal` advertencias explicativas en español claro sin jerga interna de fases.

#### Scenario: Elevación forzosa ante tuberías a intérpretes de shell
- **WHEN** un comando incluye tuberías hacia intérpretes con cualquier espaciado o prefijo como `| bash`, `|bash`, `| sudo sh` o `|/usr/bin/python3`
- **THEN** `CommandRiskGuard` SHALL clasificar el comando automáticamente como `destructive` (🔴) sin importar la etiqueta propuesta por la IA

#### Scenario: Elevación por operadores de redirección sobre comandos de lectura
- **WHEN** un comando con verbo de lectura (p. ej. `cat` o `dmesg`) incluye un operador de redirección hacia archivo fuera de comillas (`>`, `>>` o `tee`)
- **THEN** el auditor SHALL elevar inmediatamente la severidad de `read_only` a `config_change` (🟡) por implicar escritura en el sistema de archivos, mientras mantiene en `read_only` las redirecciones inocuas (`2>&1`, `2>/dev/null`)

#### Scenario: Identificación de impacto en servicios del sistema
- **WHEN** un comando manipula el ciclo de vida de demonios o servicios en vivo (incluyendo nombres que contengan la subcadena `status`, p. ej. `systemctl restart status-api`) o reglas de red activa (`ufw enable|disable`, `iptables`)
- **THEN** el auditor SHALL evaluar el subcomando real (`restart`), clasificar el riesgo en `service_impact` (🟠), emitir en `audit_warning` una explicación clara para el operador (sin usar frases internas como `"Fase de Reconocimiento prohíbe..."`), y reservar `read_only` cuando el subcomando sea `status`, `is-active`, `is-enabled` o `--version`

### Requirement: Sanitizador Preventivo de Secretos (SecretRedactor)
El sistema SHALL sanitizar todo fragmento de texto o buffer capturado antes de persistirlo o enviarlo fuera del runtime de la terminal, enmascarando llaves privadas PEM, asignaciones `KEY=VALUE` simples o compuestas (`*_PASSWORD=`, `*_SECRET=`, `*_TOKEN=`, `*_KEY=`), pares JSON/YAML sensibles (`"password": "..."`, `token: ...`), encabezados `Bearer`, credenciales en URLs y prefijos reconocidos de tokens cloud (`sk-`, `ghp_`, `gho_`, `github_pat_`, `glpat-`, `AKIA...`) mediante búsqueda ASCII *case-insensitive* por bytes segura frente a caracteres UTF-8 multibyte.

#### Scenario: Enmascaramiento de llaves privadas PEM
- **WHEN** el texto capturado contiene bloques delimitados por `-----BEGIN [A-Z ]*PRIVATE KEY-----` y `-----END [A-Z ]*PRIVATE KEY-----`
- **THEN** el sanitizador SHALL reemplazar todo el bloque por el marcador `[REDACTED_PRIVATE_KEY]`

#### Scenario: Enmascaramiento de tokens y variables de credenciales
- **WHEN** el texto contiene asignaciones del tipo `PASSWORD=...`, `TOKEN=...`, `SECRET=...`, `AWS_SECRET_ACCESS_KEY=...`, claves JSON/YAML (`"api_key": "..."`, `token: ...`), prefijos (`sk-...`, `ghp_...`, `glpat-...`, `AKIA...`), encabezados `Authorization: Bearer ...` o caracteres Unicode multibyte cuyo tamaño en bytes varía al cambiar de caja (`ß`, `İ`)
- **THEN** el sanitizador SHALL sustituir los valores sensibles por `[REDACTED_SECRET]` o `[REDACTED_TOKEN]` preservando el nombre de la variable/clave sin sufrir `panic!` UTF-8

---

### Requirement: Contratos Tipados y Validación de Esquemas
Toda salida del proveedor SHALL parsearse mediante un sobre flexible y resiliente (`AgentTurnWireEnvelope`) libre de `oneOf` en la raíz que prioriza una explicación técnica rica en Markdown (`message`) acompañada opcionalmente de **una única propuesta de acción ejecutable por turno** (`step` con `phase`, `purpose`, `command`, `risk_level`, `backup_strategy`, `rollback_cmd` y `audit_warning` calculado en Rust). Si el modelo omite campos secundarios o devuelve texto libre, el parser SHALL degradar con elegancia a respuesta conversacional sin arrojar errores fatales de esquema.

#### Scenario: Validación de propuesta de tarea completa y retrocompatibilidad de is_rolled_back
- **WHEN** el payload de una tarea o un `AgentTurnWireEnvelope` válido incluye `command`, `risk_level`, `why`, `backup_strategy` y `rollback_cmd` (omitiendo `is_rolled_back` en la salida del modelo)
- **THEN** el deserializador y conversor SHALL instanciar exitosamente `AgentTaskProposal` con `is_rolled_back = false` y `AgentResponseKind` válido

#### Scenario: Rechazo de tarea sin estrategia de reversión en cambios de configuración
- **WHEN** una tarea con nivel de riesgo `config_change` o superior omite el campo `rollback_cmd` o `backup_strategy` sin justificación
- **THEN** el validador de contrato SHALL rechazar la estructura y exigir la definición de contingencia

#### Scenario: Propuesta progresiva con explicación rica y alerta de auditoría sin rechazo
- **WHEN** el proveedor devuelve una explicación técnica en `message` junto con un objeto `step` (incluso si `CommandRiskGuard` eleva su `risk_level` y `rollback_cmd` viene vacío)
- **THEN** el backend SHALL instanciar válidamente el turno conservando `message`, ajustando `risk_level` al valor auditado y poblando `audit_warning` para informar al operador en la tarjeta

#### Scenario: Degradación elegante ante respuesta en texto plano
- **WHEN** un proveedor retorna texto técnico útil fuera de estructura o sin bloque `step`
- **THEN** el sistema SHALL entregarlo como mensaje conversacional válido al operador en lugar de abortar con pantalla de error

---

### Requirement: Ensamblador de Prompt en Dos Capas y Persistencia
El sistema SHALL ensamblar el prompt combinando la **Capa 1 inmutable (Constitución Operativa de 4 Fases de SysAdmin Senior)** con la **Capa 2 personalizable** en SQLite `app_preferences` (`execution_mode`, `active_driver`, `selected_model`, `context_max_lines`, `operator_persona`, `operator_infra_rules`, `driver_options`). La Capa 1 SHALL instruir: (1) Diagnóstico sin suposiciones de stack, (2) Diagnóstico de causa raíz + respaldo fechado (`cp archivo archivo.bak.$(date +%F_%T)`) + rollback explícito + alerta de riesgos de red/SSH, (3) Ejecución progresiva de un comando por turno con validación sintáctica *dry-run* pre-reinicio (`nginx -t`, `sshd -t`, `apache2ctl configtest`, `visudo -c`), y (4) Certificación Health Check post-cambio (`systemctl is-active`, `ss -tulpn`, `journalctl -n 20 --no-pager`), respondiendo de forma directa y concisa cuando el usuario solo realice consultas informativas.

#### Scenario: Ensamblado determinista del prompt de sistema
- **WHEN** el arnés genera las instrucciones para un turno agéntico
- **THEN** SHALL anteponer la Constitución fija (mandatos de fases, higiene de PTY, esquema JSON) e incorporar a continuación las directivas del operador (`operator_persona` y `operator_infra_rules`)

#### Scenario: Valores por defecto y restauración de preferencias
- **WHEN** la base de datos no contiene directivas configuradas o el usuario solicita restaurar valores
- **THEN** el sistema SHALL proveer directivas predeterminadas profesionales, configurar `execution_mode = Strict`, `active_driver = "agy"`, `selected_model = ""`, `context_max_lines = 200` y `driver_options` vacío en `app_preferences`

#### Scenario: Ensamblado determinista de la Constitución de 4 Fases y Directivas del Operador
- **WHEN** el orquestador construye el prompt de sistema para un turno
- **THEN** SHALL incluir las reglas de higiene PTY, las 4 fases adaptativas con requisito de *dry-run* y *health check*, el formato rico Markdown y las directivas vigentes de Capa 2

### Requirement: Comandos IPC Tauri para Control del Arnés
El backend SHALL exponer comandos Tauri asíncronos para inspeccionar el estado actual del arnés por terminal, aprobar el plan de Fase 2 (`approve_harness_plan_cmd`), ejecutar el paso actual de Fase 3 con edición opcional (`execute_ai_plan_step_cmd`), interrumpir comandos activos en PTY (`interrupt_ai_pty_step_cmd`), reanudar desde error (`retry_halted_step_cmd`), ejecutar rollback quirúrgico por paso (`execute_ai_rollback_step_cmd`), cancelar la misión activa (`cancel_harness_mission_cmd`), evaluar riesgos, gestionar preferencias, consultar salud del driver (`probe_ai_driver_health_cmd`), listar modelos disponibles (`list_ai_models_cmd`), ejecutar turnos del orquestador (`run_ai_harness_turn_cmd`) y cancelar turnos activos (`cancel_ai_turn_cmd`).

#### Scenario: Consulta de estado y preferencias por terminal
- **WHEN** el cliente invoca los comandos IPC correspondientes proporcionando el identificador de sesión
- **THEN** el backend SHALL responder con el estado actual de la FSM para esa terminal, la versión del protocolo y las preferencias almacenadas en SQLite

#### Scenario: Ejecución de turno, listado de modelos y cancelación por IPC
- **WHEN** el cliente invoca `probe_ai_driver_health_cmd`, `list_ai_models_cmd`, `run_ai_harness_turn_cmd` o `cancel_ai_turn_cmd`
- **THEN** el backend SHALL delegar en el driver configurado emitiendo eventos `ai-turn-progress` en tiempo real y permitiendo cancelar en caliente turnos en curso

#### Scenario: Aprobación formal del plan y cancelación de misión por IPC
- **WHEN** el operador pulsa **Aprobar Plan** en Fase 2 o **Cancelar Misión** en la ventana `"neko-ai"`
- **THEN** `approve_harness_plan_cmd` SHALL validar y transicionar la FSM a `GuidedExecution`, mientras que `cancel_harness_mission_cmd` SHALL abortar cualquier turno activo y restablecer la FSM a `Idle`

#### Scenario: Ejecución guiada de Fase 3, interrupción Ctrl+C y rollback por IPC
- **WHEN** el operador invoca `execute_ai_plan_step_cmd`, `interrupt_ai_pty_step_cmd`, `retry_halted_step_cmd` o `execute_ai_rollback_step_cmd` desde `"neko-ai"`
- **THEN** el backend SHALL validar la fase y el índice de tarea en `HarnessRegistry`, auditar el riesgo con `CommandRiskGuard`, gestionar el centinela en `PtyTapRegistry` y emitir la fase actualizada a la interfaz

### Requirement: Blindaje de Wrappers de Ejecución y Purgas de Bitácora en Auditoría Léxica
`CommandRiskGuard` y `ReadOnlyEnforcer` SHALL desenvolver de forma determinista wrappers encadenados de ejecución (`xargs`, `timeout`, `nice`, `ionice`, `stdbuf`, `setsid`, además de `sudo`, `env`, `nohup`, `time`, `exec` y `command`) para auditar siempre el binario y subcomandos efectivos, y clasificar operaciones destructivas sobre bitácoras (`journalctl --vacuum*`, `--flush`, `--rotate`, `--relinquish-var`) como `config_change`.

#### Scenario: Bloqueo de evasión mediante xargs, timeout, nice, ionice, stdbuf o setsid
- **WHEN** el modelo o el operador propone comandos envueltos como `ps aux | xargs kill -9`, `find /tmp -type f | xargs rm -f`, `timeout 10 systemctl restart nginx`, `nice -n 5 rm -rf /tmp/x`, `ionice -c3 cp a b` o `stdbuf -oL apt-get upgrade -y`
- **THEN** el extractor léxico SHALL resolver el binario subyacente (`kill`, `rm`, `systemctl`, `cp`, `apt-get`), clasificar el nivel real (`destructive`, `service_impact` o `config_change`), adjuntar `audit_warning` explicativo e impedir su auto-ejecución en *Modo Flexible*

#### Scenario: Uso benigno de wrappers y consulta normal de journalctl
- **WHEN** se propone `timeout 5s ping -c 1 8.8.8.8`, `echo a | xargs` o `journalctl -u nginx -n 30 --no-pager`
- **THEN** `CommandRiskGuard` SHALL mantener el riesgo en `read_only` (`Solo lectura · No modifica`), mientras que `journalctl --vacuum-time=1d` o `journalctl --rotate` SHALL elevarse a `config_change`

---

### Requirement: Telemetría Ambiental OSC 7, Conteo de Contexto e Inyección Pasiva en PTY
El backend del arnés SHALL rastrear metadatos ambientales no sensibles por sesión (`profile_name`, `current_cwd` extraído de secuencias `OSC 7` y `captured_lines` actuales), exponiéndolos hacia `"neko-ai"` y adjuntándolos automáticamente al contexto de turno de `run_ai_harness_turn_cmd`, además de ofrecer un canal de inserción pasiva en PTY sin ejecución automática.

#### Scenario: Seguimiento no invasivo de cwd OSC 7 y conteo de líneas en PtyTapRegistry
- **WHEN** la salida PTY incluye una secuencia `OSC 7` válida (p. ej. `\x1b]7;file://host/var/www/app\x07`, incluso fragmentada entre chunks) o líneas acumuladas bajo `Escucha ON`
- **THEN** `PtyTapRegistry` SHALL actualizar `current_cwd` por `terminal_id` sin retener bytes del flujo `ssh-stdout` cuando `Escucha` esté en `OFF`, y `ActiveShellInfoDto` SHALL exponer `current_cwd` y `captured_lines` reales hacia la ventana `"neko-ai"`

#### Scenario: Inyección automática de encabezado ambiental no sensible en turno agéntico
- **WHEN** se invoca `run_ai_harness_turn_cmd` para una terminal activa que tiene perfil conectado y opcionalmente un `current_cwd` detectado
- **THEN** el backend SHALL anteponer al contexto del turno (sanitizado por `SecretRedactor`) la referencia compacta del perfil y directorio actual evitando que el modelo deba consultar `pwd` inicialmente

#### Scenario: Inserción pasiva de comando sin salto de línea ni centinela
- **WHEN** se invoca `insert_ai_command_to_pty_cmd` con un comando válido
- **THEN** el backend SHALL eliminar retornos de carro o saltos de línea finales, rechazar entradas vacías y escribir el texto exacto sobre `write_ssh_raw` sin armar centinela POSIX ni alterar el estado de la FSM

