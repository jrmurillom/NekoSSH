## Context

Change 2 (`ai-core-hygiene-and-redaction`) corresponde al bloque **🟡 Media Prioridad** definido tras la auditoría de NekoAI (`feature/ai-conversational-copilot-v2`). Los 4 cambios previos (`ai-conversational-copilot-v2`, `ai-copilot-clean-hud`, `ai-risk-guard-clarity`, `ai-critical-hardening`) ya fueron archivados en `openspec/specs/`.

## Goals / Non-Goals

**Goals:**
- Eliminar el bloqueo UX cuando un proveedor retorna `ProposePlan` sin romper las pruebas existentes de `AgentHarnessFsm::advance_to_plan` / `approve_plan_and_start_execution`.
- Evitar retención en RAM de `AgentHarnessFsm` tras cerrar pestañas SSH (`close_ssh_session`).
- Hacer que el recorte `Head + Tail` de salidas capturadas por el centinela obedezca la preferencia `context_max_lines` (`50–1000`) configurada por el usuario.
- Blindar `SecretRedactor` contra fugas de credenciales en archivos `.env` / JSON / YAML y tokens cloud comunes antes de enviar contexto a proveedores externos.

**Non-Goals:**
- No agregar todavía atajos nuevos de UX como `Ctrl+Shift+E` ni botón *Insertar en terminal* (pertenecen al Change 3 `ai-copilot-power-ux`).
- No alterar el esquema SQLite existente (`app_preferences` ya almacena `ai_context_max_lines`).

## Decisions

### 1. Sincronización lock-free de `context_max_lines` en `PtyTapRegistry`
- **Decisión:** Agregar `max_capture_lines: Arc<AtomicUsize>` dentro de `PtyTapRegistry` (inicializado en `DEFAULT_CONTEXT_MAX_LINES = 200`), exponer `set_max_capture_lines(&self, max_lines: usize)` / `max_capture_lines(&self) -> usize` aplicando `clamp_context_max_lines`, e invocarlo durante `setup`, `get_ai_preferences_cmd`, `set_ai_preferences_cmd` y `reset_ai_preferences_cmd`. Dentro de `process_chunk`, pasar `max_capture_lines` a `clean_captured_command_output(raw, nonce, max_lines)`.
- **Alternativa descartada:** Consultar SQLite desde `emit_filtered_ssh_stdout` en cada paquete SSH recibido. Introduciría I/O y contención de conexión DB en el hilo de lectura del socket PTY.

### 2. Redactor por patrones sin regex pesadas y seguro frente a UTF-8
- **Decisión:** Mantener `SecretRedactor` libre de dependencias nuevas usando escaneo ASCII sobre bytes y fronteras de token válidas (`char::is_whitespace`, delimitadores JSON/YAML `,`, `}`, `]`, comillas):
  - Generalizar detección `KEY=VALUE` por sufijo de clave (`_PASSWORD=`, `_PASSWD=`, `_SECRET=`, `_TOKEN=`, `_API_KEY=`, `_ACCESS_KEY=`, `_AUTH_KEY=`, `_PRIVATE_KEY=` además de las claves exactas previas).
  - Añadir `redact_structured_kv_secrets` para claves JSON/YAML (`password`, `passwd`, `secret`, `token`, `api_key`, `access_key`, `client_secret`, `private_key`) seguidas de `:`.
  - Añadir `redact_known_token_prefixes` para prefijos estándar (`sk-`, `ghp_`, `gho_`, `github_pat_`, `glpat-`, `AKIA`) verificando fronteras de identificador y longitud mínima (`>= 16` caracteres para evitar falsos positivos en texto común).

### 3. Puente seguro para `ProposePlan` en UI + limpieza en `close_ssh_session`
- **Decisión:** Conservar intacta la FSM y sus comandos IPC heredados para retrocompatibilidad de tests, agregando un helper puro en `copilot-state.ts` (`formatPlanSummaryMarkdown(reasoning, tasks)`) que construye la tabla/lista ordenada de pasos propuestos y promueve la primera tarea pendiente a `PendingActionState` en `copilot-main.ts` cuando llega `resp.kind === "propose_plan"`. Paralelamente, añadir `harness: State<'_, ai::HarnessRegistry>` en `close_ssh_session` para invocar `harness.remove(&terminal_id)`.

## Risks / Trade-offs

- **[Falso positivo de `SecretRedactor` sobre cadenas cortas como `sk-test` o claves CSS/CLI]** → Exigir frontera alfanumérica previa (inicio de texto, espacio, comilla, `=` o `:`) y longitud mínima de token realista (`>= 16` caracteres alfanuméricos/`_`/`-` tras el prefijo, o exactamente `AKIA` + 16 alfanuméricos mayúsculos).
- **[Aprobación implícita no deseada al recibir `ProposePlan`]** → Promover solo la *propuesta visual* de la primera tarea hacia tarjeta HITL esperando clic humano explícito (nunca auto-ejecutar), manteniendo la soberanía del operador.
