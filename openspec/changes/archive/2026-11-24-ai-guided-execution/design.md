## Context

Con Hitos 1, 2 y 3 archivados, NekoSSH dispone de:
- Máquina de estados determinista `AgentHarnessFsm` (`src/ai/fsm.rs`), validadores `ReadOnlyEnforcer` y `CommandRiskGuard` (`src/ai/risk_guard.rs`), sanitizador `SecretRedactor` (`src/ai/redactor.rs`) y preferencias en SQLite (`src/ai/preferences.rs`).
- Puerto multi-proveedor `AiProviderDriver` con adaptador blindado `AgyCliDriver` (`src/ai/drivers/agy.rs`) y `HarnessOrchestrator` sin retención de `Mutex` durante inferencia (`src/ai/orchestrator.rs`).
- Ventana nativa independiente `"neko-ai"` (`app/ai-copilot.html`), colector opt-in `PtyContextTap` (`src/ai/pty_tap.rs`) y flujo operativo de Fases 1 y 2.

Para cerrar el **Hito 4** de `docs/plan/integracion-agentes-ia.md`, falta conectar la Fase 3 (`GuidedExecution`, `HaltedOnError`, `Completed`) sobre el PTY real con garantías estrictas de nivel comercial frente a condiciones reales de red y terminal Linux (`echo` TTY en un mismo paquete TCP, comentarios `#`, `Ctrl+C`, cierre temporal de la ventana secundaria y caída abrupta de sesión SSH).

## Goals / Non-Goals

**Goals:**
- Ejecutar los pasos `1..N` del plan aprobado exigiendo **siempre** clic humano explícito en **Autorizar y Ejecutar** (incluso para pasos `ReadOnly` y tanto en `Modo Estricto` como en `Modo Flexible`).
- Blindar el centinela POSIX contra eco TTY en el mismo chunk TCP, comentarios de final de línea (`#`), comandos multilínea y sobrescritura de `$?` por `PROMPT_COMMAND`.
- Sincronizar atómicamente `HarnessRegistry` desde el hilo lector de PTY en Rust al completarse cualquier centinela o al desconectarse el socket SSH con un paso en vuelo, eliminando dependencia de que la ventana `"neko-ai"` permanezca abierta.
- Proveer edición inline re-auditada por `CommandRiskGuard`, interrupción inmediata `Ctrl+C` (`\x03`) con desarme determinista del centinela (`exit_code = 130`), asistencia de IA en `HaltedOnError` y rollback quirúrgico tanto del paso fallido como de pasos previos completados (`is_rolled_back`, sugiriendo orden inverso LIFO).
- Transicionar directamente a `Completed` al concluir el paso `[N/N]` con `exit_code == 0` sin llamadas automáticas extra al proveedor de IA, y actualizar la documentación SSOT del proyecto.

**Non-Goals:**
- Auto-ejecución desatendida de planes o pasos en Fase 3 (*auto-run* / *YOLO mode*): estrictamente prohibido en todos los modos.
- Rollback automático sin confirmación explícita del operador.
- Nuevos drivers HTTP externos (OpenAI, Claude, Gemini API, Ollama), diferidos fuera del MVP.

## Decisions

### 1. Wrapper POSIX Inmune a Eco TTY, Comentarios y `PROMPT_COMMAND`
- **Decisión:** Reemplazar la envoltura lineal por bloque agrupado en el shell actual con formato desacoplado en `printf`:
  ```sh
  { <clean_cmd>
  } ; __neko_rc=$? ; printf '\n%s_%s:%d__\n' '__NEKO_END' '<nonce>' "$__neko_rc"
  ```
- **Por qué:**
  1. El salto de línea antes de `}` evita que un comentario `#` al final de `<clean_cmd>` comente el centinela, mientras que ejecutar dentro de `{ ... }` preserva `cd` o variables de entorno en el mismo shell interactivo.
  2. `__neko_rc=$?` captura el código de retorno en la misma sentencia compuesta antes de que `PROMPT_COMMAND` o hooks de `PS1` se ejecuten.
  3. Separar `'__NEKO_END'` y `'<nonce>'` con `%s_%s` garantiza que en el **eco TTY** nunca exista la subcadena contigua `__NEKO_END_<nonce>:`, eliminando el bug donde eco y salida viajan en el mismo paquete TCP. Adicionalmente, el buscador iterará todas las apariciones de `marker_head` en `combined` hasta encontrar un `i32` válido.
- **Alternativa descartada:** Subshell `( <cmd> )` — descartado porque perdería cambios de directorio o variables exportadas deliberadamente entre comandos si el usuario los encadena o edita.

### 2. Centinelas Tipados (`SentinelPurpose`) y Transición Atómica en Backend
- **Decisión:** `PtyTapRegistry::try_arm_sentinel` recibe un `SentinelPurpose`:
  ```rust
  pub enum SentinelPurpose {
      ReadOnlyRecon { command: String },
      PlanStep { step_index: usize, command: String },
      StepRollback { step_index: usize, rollback_cmd: String },
  }
  ```
  Rechaza armar si ya existe `active_sentinel.is_some()` en ese `terminal_id`. Cuando `emit_filtered_ssh_stdout` en `src/lib.rs` recibe `Some(completion)`:
  - Redacta la salida con `SecretRedactor::redact`.
  - Si `purpose` es `PlanStep`: con `exit_code == 0` invoca `fsm.complete_current_step()`; con `exit_code != 0` invoca `fsm.halt_on_error(command, exit_code, redacted_output)`.
  - Si `purpose` es `StepRollback`: con `exit_code == 0` invoca `fsm.mark_step_rolled_back(step_index)`.
  - Emite `ai-pty-command-finished` incluyendo `purpose`, `exit_code`, `captured_output` redactado y `updated_phase: Option<HarnessPhase>`.
- **Por qué:** Evita condiciones de carrera de doble clic y garantiza que la FSM nunca pierda el resultado de un comando largo aunque el operador cierre o recargue `"neko-ai"` mientras el comando corre en `"main"`.

### 3. Interrupción Remota (`Ctrl+C`) sin Bloqueo de Centinela
- **Decisión:** El comando IPC `interrupt_ai_pty_step_cmd(terminal_id)`:
  1. Envía `"\x03"` mediante `write_ssh_raw`.
  2. Extrae y limpia inmediatamente el centinela activo con `pty_taps.take_sentinel(&terminal_id)`.
  3. Si el centinela extraído era un `PlanStep`, transiciona la FSM a `HaltedOnError` con `exit_code = 130` y mensaje `"Ejecución interrumpida manualmente por el operador (Ctrl+C)"`.
- **Por qué:** En shells interactivos (`bash`/`zsh`), `SIGINT` suele abortar la lista compuesta antes de ejecutar el `printf` posterior; desarmar y transicionar en backend garantiza respuesta instantánea en UI.

### 4. Desconexión SSH con Paso en Vuelo
- **Decisión:** En el cierre/EOF del bucle lector SSH (`src/lib.rs`), antes de llamar a `fsm.pause_on_disconnect()`, se consulta `pty_taps.take_sentinel(&term_id)`. Si había un `PlanStep { command, .. }` pendiente de terminar, se aplica primero `fsm.halt_on_error(command, -1, "Conexión SSH interrumpida mientras el paso estaba en ejecución. Verifica el servidor antes de reintentar o revertir.")` y luego `fsm.pause_on_disconnect()`.
- **Por qué:** Al reconectar (`resume_after_reconnect`), la sesión aterriza en `HaltedOnError` en lugar de aparentar que el paso nunca se ejecutó, previniendo re-ejecuciones a ciegas de comandos parcialmente aplicados.

### 5. Rollback Quirúrgico (Actual y LIFO de Pasos Completados) + Asistencia en `HaltedOnError`
- **Decisión:**
  - `AgentTaskProposal` incorpora `#[serde(default)] pub is_rolled_back: bool` (sin añadirlo a `required` del JSON Schema del proveedor para mantener 100% compatibilidad con LLMs).
  - `execute_ai_rollback_step_cmd(terminal_id, step_index, custom_rollback_cmd)` permite ejecutar con centinela `StepRollback` tanto el `rollback_cmd` del paso `step_index` fallido en `HaltedOnError` como el de cualquier paso previo `j` con `is_done && !is_rolled_back`. En la UI se resalta como recomendado el último paso completado no revertido (orden LIFO) y se pide confirmación explícita mostrando el comando exacto.
  - En `HarnessOrchestrator::execute_turn`, cuando `snapshot.phase` es `HaltedOnError`, `build_phase_directive` inyecta en el prompt de sistema el índice fallido, comando, `exit_code` y salida de error sanitizada. Si el modelo devuelve `ProposeStep`, la UI ofrece **Aplicar corrección al paso** (`retry_halted_step_cmd`); si devuelve `ProposePlan`, `fsm.replace_remaining_plan_from_error(tasks)` actualiza los pasos desde `step_index` preservando intactos los pasos `0..step_index` ya completados.

## Risks / Trade-offs

- **[Riesgo] Comandos que piden contraseña interactiva (`sudo` sin `NOPASSWD`)** → **Mitigación:** El comando corre en el PTY real visible en `"main"`, por lo que el operador puede escribir la contraseña directamente en `xterm.js` (`termios` oculta el eco y `SecretRedactor` protege cualquier captura), o pulsar `Ctrl+C` desde cualquiera de las dos ventanas y editar inline el paso.
- **[Riesgo] Rollback cuyo comando a su vez falla (`exit_code != 0`)** → **Mitigación:** Como el rollback también viaja envuelto con centinela `StepRollback`, si devuelve `exit_code != 0` **no** se marca `is_rolled_back = true`, se conserva el estado de alerta y se expone el error exacto del rollback en el feed para intervención asistida o manual.
- **[Riesgo] Edición inline que introduce comando vacío o riesgo superior sin respaldo** → **Mitigación:** `execute_ai_plan_step_cmd` rechaza comandos vacíos y recalcula siempre `CommandRiskGuard::assess_and_elevate` actualizando la tarea en la FSM antes de escribir al PTY.
