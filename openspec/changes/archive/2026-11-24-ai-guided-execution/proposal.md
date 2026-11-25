## Why

Con los Hitos 1, 2 y 3 completados (`ai-harness-core`, `ai-provider-driver-agy` y `ai-copilot-window`), el operador ya cuenta con diagnóstico de solo lectura (Fase 1) y debate/aprobación del plan estructurado (Fase 2), pero al aprobar el plan la misión aún se detiene sin poder ejecutar las tareas aprobadas sobre el servidor remoto. Este cuarto y último hito del roadmap agéntico habilita la Fase 3 (`GuidedExecution`) con control humano estricto paso a paso, detección determinista de códigos de salida en el PTY real, freno de emergencia automático ante errores y contingencia quirúrgica mediante `rollback_cmd`.

## What Changes

- **Corredor `PtySentinelRunner` blindado contra eco TTY y sincronizado en Backend (`pty_tap.rs` + `lib.rs`):**
  - Evoluciona `wrap_command_with_sentinel` hacia un bloque POSIX `{ <cmd>\n} ; __neko_rc=$? ; printf '\n%s_%s:%d__\n' '__NEKO_END' '<nonce>' "$__neko_rc"`, soportando comentarios `#` y multilínea, capturando `$?` antes de `PROMPT_COMMAND` e impidiendo que el eco TTY dispare falsos negativos cuando eco y salida viajan en el mismo paquete TCP.
  - Etiqueta cada centinela activo con su propósito (`ReadOnlyRecon`, `PlanStep { step_index, command }`, `StepRollback { step_index, rollback_cmd }`) e impide colisiones si ya existe un comando en vuelo en la misma terminal.
  - Actualiza atómicamente `HarnessRegistry` desde el propio hilo lector Rust al completarse un centinela (o al caer el socket SSH a mitad de un paso en vuelo), garantizando consistencia incluso si la ventana `"neko-ai"` se cierra temporalmente durante un comando largo.
- **Ejecución Guiada Paso a Paso con Autorización Humana Universal y Edición Inline (`commands.rs` + `fsm.rs` + `copilot-main.ts`):**
  - Exige clic explícito del operador en **Autorizar y Ejecutar** para absolutamente todos los pasos `1..N` del plan en Fase 3 (incluyendo verificaciones `ReadOnly`), tanto en `Modo Estricto` como en `Modo Flexible`.
  - Permite editar inline el comando del paso actual antes de ejecutarlo, re-auditando en vivo su nivel de riesgo mediante `CommandRiskGuard` en UI y en backend.
  - Expone interrupción inmediata `interrupt_ai_pty_step_cmd` (`Ctrl+C` / `\x03`) que aborta el proceso remoto, limpia el centinela activo y transiciona deterministamente a `HaltedOnError` (`exit_code = 130`).
- **Freno de Emergencia (`HaltedOnError`), Reintento Asistido y Rollback Quirúrgico por Paso (`fsm.rs` + `orchestrator.rs` + `schema.rs`):**
  - Cuando un paso devuelve `exit_code != 0`, congela el resto del plan y expone en `"neko-ai"` la salida capturada junto a tres acciones controladas: consultar corrección a la IA (`propose_step` / replanteamiento de pasos restantes), editar manualmente y reintentar (`resume_from_error`), o ejecutar el `rollback_cmd` del paso fallido con centinela POSIX.
  - Extiende `AgentTaskProposal` con `#[serde(default)] pub is_rolled_back: bool`, permitiendo además revertir bajo confirmación humana pasos previos ya completados `[✓]` (priorizando orden inverso LIFO) y marcándolos como `[↺ Revertido]` para evitar doble ejecución accidental.
- **Cierre Determinista en `[N/N]` y Consolidación Documental SSOT:**
  - Al finalizar el último paso con `exit_code == 0`, transiciona directamente a `Completed` sin invocar llamadas adicionales al proveedor de IA, dejando opcional solicitar un resumen o verificación posterior.
  - Actualiza la documentación permanente (`docs/plan/integracion-agentes-ia.md`, `docs/project_scope.md`, `docs/design/DESIGN.md` y `docs/design/ui-layout-contract.md`) cerrando la iniciativa completa de integración de agentes IA.

## Capabilities

### New Capabilities
- `ai-guided-execution`: Ejecución guiada paso a paso de Fase 3 en PTY con centinela POSIX inmune a eco TTY, autorización humana obligatoria en todos los pasos, edición inline re-auditada, interrupción `Ctrl+C`, freno automático ante fallo, rollback quirúrgico por paso (actual y completados previos) y cierre determinista en `Completed`.

### Modified Capabilities
- `ai-harness-core`: Extiende `AgentTaskProposal` con estado `is_rolled_back`, soporte de reversión por índice, transición atómica de FSM en backend al completarse centinelas o desconectarse en vuelo y asistencia de corrección en `HaltedOnError`.
- `ai-copilot-window`: Reemplaza el placeholder de Fase 3 en `"neko-ai"` por el checklist interactivo de progreso `[i/N]`, tarjeta activa con edición inline y `Ctrl+C`, vista `HaltedOnError` con `rollback_cmd` editable y pantalla de cierre `Completed`.

## Impact

- **Backend (`app/src-tauri/src/`):** `ai/pty_tap.rs`, `ai/schema.rs`, `ai/fsm.rs`, `ai/orchestrator.rs`, `ai/commands.rs`, `lib.rs`. Sin dependencias externas nuevas en `Cargo.toml`.
- **Frontend (`app/`):** `ai-copilot.html`, `src/ai-copilot/copilot.css`, `src/ai-copilot/copilot-state.ts`, `src/ai-copilot/copilot-state.test.ts`, `src/ai-copilot/copilot-main.ts`, `src/icons.ts` (si se requiere icono adicional Lucide outline).
- **Documentación SSOT (`docs/`):** `docs/plan/integracion-agentes-ia.md`, `docs/project_scope.md`, `docs/design/DESIGN.md`, `docs/design/ui-layout-contract.md`.
