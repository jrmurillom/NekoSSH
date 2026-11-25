## MODIFIED Requirements

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

### Requirement: Validador Estricto de Solo Lectura (ReadOnlyEnforcer)
El backend SHALL utilizar `ReadOnlyEnforcer` exclusivamente como **clasificador no bloqueante de elegibilidad para auto-ejecución** en `Modo Flexible` (nunca para abortar turnos con error), reconociendo como lectura inocua la sintaxis estándar POSIX de descarte y mezcla de descriptores (`2>/dev/null`, `>/dev/null`, `1>/dev/null`, `&>/dev/null`, `2>&1`, `1>&2`) y devolviendo una nota explicativa (`audit_warning`) cuando un comando contenga redirecciones a archivo, subshells, binarios desconocidos o verbos mutantes.

#### Scenario: Aprobación de comandos puros de diagnóstico
- **WHEN** el agente propone comandos como `uname -a`, `cat /etc/os-release`, `free -h`, `ss -tulpn`, `df -h` o búsquedas como `grep -i "error" /var/log/syslog`
- **THEN** el validador SHALL clasificar el comando como permitido en fase de reconocimiento

#### Scenario: Bloqueo de redirecciones de escritura y mutaciones
- **WHEN** un comando en reconocimiento contiene redirecciones fuera de comillas (`>`, `>>`), `tee` directo o en tubería, tuberías hacia intérpretes con o sin espacios/prefijos (`|bash`, `|  sh`, `| sudo bash`), `find` con `-delete`/`-exec`, `crontab`, gestores de paquetes/contenedores, creación o borrado de archivos (`rm`, `touch`, `mkdir`, `mv`, `cp`, `chmod`, `chown`) o control de servicios (`systemctl`, `service`, `kill`)
- **THEN** el validador SHALL rechazar el comando marcándolo como no conforme para la fase de reconocimiento, impidiendo su presentación como lectura segura

#### Scenario: Reconocimiento de supresión estándar POSIX 2>/dev/null y 2>&1 como lectura pura
- **WHEN** el agente propone comandos diagnósticos como `cat /etc/os-release 2>/dev/null || uname -a` o `nginx -t 2>&1`
- **THEN** `ReadOnlyEnforcer` y `CommandRiskGuard` SHALL clasificarlos como `read_only` (`🟢`) sin emitir falsos positivos de redirección de escritura

#### Scenario: Degradación segura a autorización humana sin bloquear el turno
- **WHEN** el agente propone un comando que utiliza un binario propio no catalogado, sustitución `$()`, redirección real a archivo (`> /tmp/out`) o modificación de sistema
- **THEN** el backend **NO** abortará el turno con error; desactivará `auto_approved_read_only`, asignará el nivel de riesgo auditado junto con `audit_warning` explicativo y presentará la tarjeta al operador para que decida si autoriza, edita u omite

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
