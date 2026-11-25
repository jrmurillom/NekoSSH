## ADDED Requirements

### Requirement: Transiciones de Máquina de Estados del Arnés
El backend SHALL implementar una máquina de estados finita (`AgentHarnessFsm`) que restrinja estrictamente el ciclo de vida del agente en los estados `Idle`, `Reconnaissance`, `PlanAndDebate`, `GuidedExecution`, `HaltedOnError`, `PausedOnDisconnect` y `Completed`. Queda prohibido saltar a ejecución sin haber aprobado previamente un plan o permanecer en reconocimiento con comandos mutantes.

#### Scenario: Transición válida de Reconocimiento a Plan
- **WHEN** el arnés recibe la orden de iniciar diagnóstico para un objetivo y completa la recolección de lecturas autorizadas
- **THEN** la máquina de estados transiciona de `Reconnaissance` a `PlanAndDebate`, bloqueando toda ejecución en el servidor mientras se debate el plan

#### Scenario: Freno automático ante salida de error en ejecución
- **WHEN** durante el estado `GuidedExecution` un comando ejecutado devuelve un código de salida diferente de cero (`exit_code != 0`)
- **THEN** el arnés SHALL transicionar de inmediato al estado `HaltedOnError`, congelando las tareas restantes y requiriendo acción explícita de corrección o reversión

#### Scenario: Congelamiento y reanudación ante desconexión de red
- **WHEN** la sesión SSH asociada se desconecta mientras el arnés se encuentra en `GuidedExecution` o `PlanAndDebate`
- **THEN** el arnés SHALL transicionar a `PausedOnDisconnect` conservando el plan y las tareas completadas en memoria, permitiendo al reconectar la sesión reanudar la misión sin pérdida de avance

#### Scenario: Rechazo de transición ilegal
- **WHEN** se intenta avanzar a `GuidedExecution` directamente desde `Idle` o `Reconnaissance` sin la previa aprobación del plan
- **THEN** el arnés SHALL rechazar la transición con un error de violación de fase y mantener el estado previo intacto

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
Durante la fase de `Reconnaissance`, el backend SHALL validar mediante análisis léxico que todo comando propuesto sea estrictamente de inspección y no cause mutaciones en el sistema remoto.

#### Scenario: Aprobación de comandos puros de diagnóstico
- **WHEN** el agente propone comandos como `uname -a`, `cat /etc/os-release`, `free -h`, `ss -tulpn` o `df -h`
- **THEN** el validador SHALL clasificar el comando como permitido en fase de reconocimiento

#### Scenario: Bloqueo de redirecciones de escritura y mutaciones
- **WHEN** un comando en reconocimiento contiene operadores de redirección (`>`, `>>`, `| tee`), gestores de paquetes (`apt`, `dnf`, `yum`, `apk`, `pacman`), creación o borrado de archivos (`rm`, `touch`, `mkdir`, `mv`, `cp`, `chmod`, `chown`) o control de servicios (`systemctl`, `service`, `kill`)
- **THEN** el validador SHALL rechazar el comando marcándolo como no conforme para la fase de reconocimiento, impidiendo su presentación como lectura segura

---

### Requirement: Auditor Estático de Riesgo en Cuatro Etapas (CommandRiskGuard)
El sistema SHALL auditar y clasificar el riesgo operativo de cada comando propuesto mediante un pipeline léxico determinista de 4 etapas: Tokenizador/Segmentador por operadores, Detector de tuberías hacia intérpretes de shell, Análisis de verbos y modificadores, y Auditor de operadores de redirección. La salida SHALL categorizarse en cuatro niveles: `read_only` (verde), `config_change` (amarillo), `service_impact` (naranja) y `destructive` (rojo).

#### Scenario: Elevación forzosa ante tuberías a intérpretes de shell
- **WHEN** un comando incluye tuberías hacia intérpretes como `| bash`, `| sh` o `| python` (p. ej. `curl ... | bash`)
- **THEN** `CommandRiskGuard` SHALL clasificar el comando automáticamente como `destructive` (🔴) sin importar la etiqueta propuesta por la IA

#### Scenario: Elevación por operadores de redirección sobre comandos de lectura
- **WHEN** un comando con verbo de lectura (p. ej. `cat` o `dmesg`) incluye un operador de redirección hacia archivo (`>` o `>>`)
- **THEN** el auditor SHALL elevar inmediatamente la severidad de `read_only` a `config_change` (🟡) por implicar escritura en el sistema de archivos

#### Scenario: Identificación de impacto en servicios del sistema
- **WHEN** un comando manipula el ciclo de vida de demonios o servicios en vivo (`systemctl restart|stop|reload`, `service ... restart`) o reglas de red activa (`ufw enable|disable`, `iptables`)
- **THEN** el auditor SHALL clasificar el riesgo en `service_impact` (🟠), requiriendo validación de contingencia

---

### Requirement: Sanitizador Preventivo de Secretos (SecretRedactor)
El sistema SHALL sanitizar todo fragmento de texto o buffer capturado antes de persistirlo o enviarlo fuera del runtime de la terminal, enmascarando patrones de información sensible.

#### Scenario: Enmascaramiento de llaves privadas PEM
- **WHEN** el texto capturado contiene bloques delimitados por `-----BEGIN [A-Z ]*PRIVATE KEY-----` y `-----END [A-Z ]*PRIVATE KEY-----`
- **THEN** el sanitizador SHALL reemplazar todo el bloque por el marcador `[REDACTED_PRIVATE_KEY]`

#### Scenario: Enmascaramiento de tokens y variables de credenciales
- **WHEN** el texto contiene asignaciones del tipo `PASSWORD=...`, `TOKEN=...`, `SECRET=...` o encabezados `Authorization: Bearer ...`
- **THEN** el sanitizador SHALL sustituir los valores sensibles por `[REDACTED_SECRET]` preservando el nombre de la variable para contexto diagnóstico

---

### Requirement: Contratos Tipados y Validación de Esquemas
Toda interacción de entrada y salida del agente SHALL modelarse con estructuras tipadas en Rust con soporte para deserialización estricta (`serde`). Cada propuesta de tarea en ejecución SHALL incluir obligatoriamente justificación, nivel de riesgo, estrategia de respaldo y comando de reversión.

#### Scenario: Validación de propuesta de tarea completa
- **WHEN** el payload de una tarea incluye `command`, `risk_level`, `why`, `backup_strategy` y `rollback_cmd`
- **THEN** el deserializador SHALL instanciar exitosamente la estructura `AgentTaskProposal`

#### Scenario: Rechazo de tarea sin estrategia de reversión en cambios de configuración
- **WHEN** una tarea con nivel de riesgo `config_change` o superior omite el campo `rollback_cmd` o incluye una cadena vacía sin justificación
- **THEN** el validador de contrato SHALL rechazar la estructura y exigir la definición de contingencia

---

### Requirement: Ensamblador de Prompt en Dos Capas y Persistencia
El sistema SHALL ensamblar el contexto inicial del agente combinando una Capa 1 inmutable en código (Constitución del Arnés) con una Capa 2 personalizable (Directivas del Operador y modo de ejecución) almacenada en la tabla SQLite `app_preferences`.

#### Scenario: Ensamblado determinista del prompt de sistema
- **WHEN** el arnés genera las instrucciones para un turno agéntico
- **THEN** SHALL anteponer la Constitución fija (mandatos de fases, higiene de PTY, esquema JSON) e incorporar a continuación las directivas del operador (`ai_operator_persona_prompt` y `ai_operator_infra_rules`)

#### Scenario: Valores por defecto y restauración de preferencias
- **WHEN** la base de datos no contiene directivas configuradas o el usuario solicita restaurar valores
- **THEN** el sistema SHALL proveer directivas predeterminadas profesionales, configurar el modo `Strict` por defecto y persistirlas en `app_preferences`

---

### Requirement: Comandos IPC Tauri para Control del Arnés
El backend SHALL exponer comandos Tauri asíncronos para inspeccionar el estado actual del arnés por terminal, evaluar riesgos de comandos y consultar o actualizar las directivas y modo de ejecución del operador.

#### Scenario: Consulta de estado y preferencias por terminal
- **WHEN** el cliente invoca los comandos IPC correspondientes proporcionando el identificador de sesión
- **THEN** el backend SHALL responder con el estado actual de la FSM para esa terminal, la versión del protocolo y las preferencias almacenadas en SQLite
