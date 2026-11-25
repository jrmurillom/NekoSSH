## Why

Tras resolver los huecos críticos en `ai-critical-hardening`, quedan 4 deudas técnicas de **prioridad media** identificadas en la auditoría de NekoAI que impactan higiene de memoria, consistencia de flujo y defensa contra fugas de secretos:

1. **Split-Brain en `ProposePlan`**: Si un proveedor devuelve un `plan` heredado (`AgentResponseKind::ProposePlan`), el orquestador transiciona la FSM a `PlanAndDebate`, pero `copilot-main.ts` solo renderiza `resp.reasoning` sin exponer las tareas ni transición ejecutable, dejando la sesión atascada sin botón para avanzar.
2. **Retención en `HarnessRegistry` al cerrar pestañas**: `close_ssh_session` limpia `ai_cancels`, `pty_taps` y `ai_feeds`, pero omite liberar `harness.remove(&terminal_id)`.
3. **Presupuesto fijo de captura tras centinela**: `clean_captured_command_output` en `pty_tap.rs` trunca siempre a `200` líneas aunque el operador configure `context_max_lines` entre `50` y `1000` en Ajustes.
4. **Cobertura limitada de `SecretRedactor`**: No enmascara variantes compuestas con `_KEY=` (ej. `AWS_SECRET_ACCESS_KEY=...`), credenciales en JSON/YAML (`"password": "..."`, `token: ...`) ni prefijos estándar (`sk-...`, `ghp_...`, `gho_...`, `glpat-...`, `AKIA...`).

## What Changes

- **Unificación progresiva de respuestas `ProposePlan` (`app/src/ai-copilot/copilot-main.ts`, `app/src-tauri/src/ai/orchestrator.rs`)**:
  - Cuando un turno retorne `ProposePlan` en la vista conversacional v2, renderizar el resumen estructurado de pasos en Markdown y promover el primer paso accionable a tarjeta de autorización (`sess.pendingAction`) para evitar estados bloqueados en `PlanAndDebate`, manteniendo intacta la compatibilidad con los contratos y pruebas unitarias existentes de la FSM.
- **Limpieza completa de `HarnessRegistry` en cierre de sesión SSH (`app/src-tauri/src/lib.rs`)**:
  - Invocar `harness.remove(&terminal_id)` dentro de `close_ssh_session` para evitar acumulación de arneses huérfanos en sesiones largas.
- **Límite dinámico `context_max_lines` en salida capturada (`app/src-tauri/src/ai/pty_tap.rs`, `app/src-tauri/src/ai/commands.rs`, `app/src-tauri/src/lib.rs`)**:
  - Incorporar un parámetro atómico `max_capture_lines` en `PtyTapRegistry` sincronizado con `ai_preferences.context_max_lines` al iniciar la app y al guardar o restaurar ajustes, sustituyendo el hardcode `200` en `clean_captured_command_output`.
- **Ampliación de `SecretRedactor` (`app/src-tauri/src/ai/redactor.rs`)**:
  - Cubrir claves `*_KEY=`, `*_SECRET=`, `*_TOKEN=`, `*_PASSWORD=`, pares JSON/YAML (`"password": "..."`, `"api_key": "..."`, `token: ...`) y tokens/llaves con prefijos reconocidos (`sk-`, `ghp_`, `gho_`, `github_pat_`, `glpat-`, `AKIA`).

## Capabilities

### New Capabilities
- *(Ninguna)*

### Modified Capabilities
- `ai-harness-core`: Ampliación de cobertura de `SecretRedactor` y manejo coherente de propuestas multipaso en el orquestador/cliente.
- `ai-copilot-window`: Respeto dinámico de `context_max_lines` en salidas de centinela, liberación integral al cerrar pestañas SSH y renderizado accionable de respuestas `ProposePlan`.

## Impact

- **Backend (`app/src-tauri/src/`)**: `ai/redactor.rs`, `ai/pty_tap.rs`, `ai/commands.rs`, `ai/orchestrator.rs`, `lib.rs`.
- **Frontend (`app/src/`)**: `ai-copilot/copilot-main.ts`, `ai-copilot/copilot-state.ts`, `ai-copilot/copilot-state.test.ts`.
