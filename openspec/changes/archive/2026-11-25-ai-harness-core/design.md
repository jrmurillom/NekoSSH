## Context

La integración de capacidades agénticas dentro de NekoSSH requiere una base arquitectónica sólida en el backend Rust para garantizar que ningún comando se ejecute sin supervisión humana y que el agente no pueda romper el sistema por alucinación. El Hito 1 establece el núcleo del arnés agéntico (`AgentHarness`), la máquina de estados de tres fases con resiliencia ante desconexiones, los modos de ejecución (Estricto vs Flexible), el auditor léxico de riesgo en 4 etapas, el sanitizador de secretos y la persistencia de configuración del operador en SQLite (`app_preferences`).

## Goals / Non-Goals

**Goals:**
- Implementar una máquina de estados finita (`AgentHarnessFsm`) testeable en memoria bajo TDD que gobierne las fases y soporte suspensión por desconexión de red (`PausedOnDisconnect`).
- Diseñar un auditor de riesgo determinista (`CommandRiskGuard`) en un pipeline de 4 etapas (Lexer, Detector de pipes a shell, Verbos/Flags y Redirecciones) que eleve forzosamente comandos riesgosos.
- Incorporar modos operativos en SQLite (`ai_execution_mode`): `Strict` (autorización para todo) y `Flexible` (auto-autorización de lecturas puras sin mutación, con freno forzoso en cambios).
- Crear el filtro léxico estricto `ReadOnlyEnforcer` para la fase de reconocimiento.
- Diseñar contratos JSON estrictos (`serde`) con validación obligatoria de justificación, respaldo y reversión.
- Implementar el sanitizador `SecretRedactor` para evitar fugas de claves PEM, credenciales o tokens.
- Implementar el ensamblador `PromptAssembler` en dos capas (Constitución inmutable en código + Directivas del operador en SQLite).
- Estructurar el estado por sesión mediante `HarnessRegistry(Arc<Mutex<HashMap<String, AgentHarnessFsm>>>)`.
- Exponer comandos IPC Tauri seguros para control del arnés y gestión de directivas.

**Non-Goals:**
- No incluye en este hito la invocación del subproceso CLI `agy` ni drivers externos (asignados al Hito 2).
- No incluye la creación de la ventana nativa secundaria `neko-ai` en el frontend (asignada al Hito 3).
- No incluye la inyección real en el PTY ni el corredor de centinelas (asignados al Hito 4).
- No incluye modo "turbo" o ejecución desatendida sin confirmación, descartado por diseño.

## Decisions

### Decisión 1: Máquina de estados gobernada en Rust con resiliencia de sesión
- **Razón**: Si la sesión SSH sufre un microcorte o el usuario reconecta con `Ctrl+R`, el arnés pasa a `PausedOnDisconnect` manteniendo el plan activo en memoria. Al reconectar, el usuario puede continuar desde la tarea en curso en lugar de reiniciar todo el flujo.
- **Alternativa descartada**: Resetear el estado a `Idle` en cada evento de desconexión. Descartado porque degrada gravemente la experiencia en servidores con redes inestables.

### Decisión 2: Pipeline de análisis de riesgo léxico en 4 etapas
- **Razón**: En lugar de confiar en una lista plana de comandos o en expresiones regulares monolíticas frágiles, `CommandRiskGuard` opera en 4 pasos secuenciales:
  1. *Tokenizador & Segmentador:* Divide la línea por operadores de control (`;`, `&&`, `||`, tuberías `|`).
  2. *Detector de Inyección a Shell:* Detecta patrones como `| bash`, `| sh`, `| python` y clasifica inmediatamente como `destructive` (🔴).
  3. *Evaluador de Verbo & Flags:* Mapea el binario base y flags críticos (p. ej. `sed -i` vs `sed`, `rm -rf` vs `ls`).
  4. *Auditor de Redirección:* Cualquier uso de `>`, `>>` o `| tee` invalida automáticamente la etiqueta `read_only` y la eleva a `config_change` (🟡) o superior.
- **Alternativa descartada**: Parser completo de AST de bash. Descartado por sobrecarga de dependencias y mantenimiento; el pipeline de 4 etapas cubre el 100% de los riesgos requeridos con cero dependencias externas.

### Decisión 3: Modos de ejecución configurables (Estricto vs Flexible)
- **Razón**: Permite al usuario elegir entre supervisión total paranoica (pide autorización para cada `cat` o `free -h`) o agilidad semi-supervisada (el agente ejecuta lecturas puras de diagnóstico automáticamente y frena en seco ante cualquier cambio o servicio).
- **Alternativa descartada**: Modo "turbo" totalmente autónomo. Descartado por directiva de seguridad del producto.

### Decisión 4: Ensamblador de prompt en dos capas separadas
- **Razón**: Si el usuario edita o borra por error el prompt en la configuración, la Capa 1 fija en Rust garantiza que las reglas estructurales (fases, contratos JSON, prohibición de paginadores interactivos) sigan intactas. La Capa 2 permite personalizar el rol y reglas de infraestructura con soporte de restauración de fábrica.
- **Alternativa descartada**: Un solo archivo de texto editable que contenga todo el prompt. Descartado por fragilidad operativa.

### Decisión 5: Reutilización de la tabla `app_preferences` y Registry multi-sesión
- **Razón**: `app_preferences` ya está probada en SQLite (`preferences.rs`) y permite persistir `ai_execution_mode`, `ai_operator_persona_prompt` y directivas sin alterar el esquema relacional. Además, indexar la FSM por `terminal_id` permite que múltiples pestañas mantengan misiones independientes sin colisión.

## Risks / Trade-offs

- **[Riesgo] Evasión de comandos de solo lectura mediante ofuscación o subshells complejas** → *Mitigación*: `ReadOnlyEnforcer` bloquea de forma conservadora cualquier comando con expansiones `$()`, `` ` ``, comillas anidadas no parseables o tuberías a intérpretes. Ante la duda léxica, se rechaza el comando en Fase 1 y se exige autorización manual.
- **[Riesgo] Falsos positivos en `SecretRedactor`** → *Mitigación*: Expresiones regulares delimitadas por fronteras de palabra (`\b`) y formatos estructurados (como bloques PEM y headers `Bearer`) para no censurar hashes o identificadores inofensivos.
- **[Riesgo] Desalineación entre modo de ejecución y estado de interfaz** → *Mitigación*: Los comandos IPC exponen atómicamente el modo activo junto al estado de la FSM en cada consulta.

## Migration Plan

No requiere migraciones de base de datos relacional. Las nuevas claves de preferencias se inicializan con valores por defecto cuando no existen.

## Open Questions

- Los valores de fábrica del modo de ejecución serán `Strict` por defecto para priorizar seguridad en instalaciones limpias.
