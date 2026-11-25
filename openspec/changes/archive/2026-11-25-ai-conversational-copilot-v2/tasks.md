**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)
- [x] 0.1 Crear y posicionarse en la rama de feature `feature/ai-conversational-copilot-v2` (`git checkout -b feature/ai-conversational-copilot-v2`)
- [x] 0.2 Verificar que el árbol de trabajo esté posicionado en la rama creada

## 1. Constitución de 4 Fases, Clasificador No Bloqueante y Contrato Resiliente (TDD Rust)
- [x] 1.1 Actualizar `src/ai/risk_guard.rs` para reconocer redirecciones inocuas estándar POSIX (`2>/dev/null`, `>/dev/null`, `1>/dev/null`, `&>/dev/null`, `2>&1`, `1>&2`) como `ReadOnly` y exponer `evaluate_proposal` no bloqueante que devuelve `ProposalSafetyReport { effective_risk, auto_runnable_in_flexible, audit_warning }` con pruebas TDD
- [x] 1.2 Actualizar `src/ai/prompt.rs` con la Constitución Operativa de 4 Fases de SysAdmin Senior (1. Diagnóstico sin suposición de stack + respuesta directa a consultas, 2. Causa raíz + backup fechado + rollback + alerta de riesgos, 3. Un comando por turno + *dry-run* pre-reinicio `nginx -t`/`sshd -t`, 4. Certificación Health Check `is-active`/`ss -tulpn`/`journalctl`)
- [x] 1.3 Evolucionar `src/ai/schema.rs`, `src/ai/fsm.rs`, `src/ai/drivers/agy.rs` y `src/ai/orchestrator.rs` al contrato conversacional progresivo v2 (`phase`, `message` Markdown rico y `step` opcional de 1 comando con `purpose`, `audit_warning`, `rollback_cmd`), degradación elegante ante texto plano o campos omitidos y conservación continua de memoria por sesión con pruebas TDD

## 2. Sembrado Instantáneo de Pantalla Visible (`Escucha ON`) y Comandos IPC Unificados
- [x] 2.1 Extender `src/ai/pty_tap.rs` con `seed_visible_screen` (solo cuando `listening == true`, purgado a `0 bytes` al apagar) y actualizar `src/ai/commands.rs` + `src/lib.rs` con comando unificado de ejecución autorizada (`execute_ai_command_cmd`), limpieza voluntaria de chat (`clear_ai_conversation_cmd`) y sembrado de pantalla (`seed_pty_visible_screen_cmd`) con pruebas unitarias

## 3. Renderizador Markdown Técnico Seguro (`markdown-lite.ts`) y Helpers Frontend (`vitest`)
- [x] 3.1 Crear `app/src/ai-copilot/markdown-lite.ts` y `app/src/ai-copilot/markdown-lite.test.ts` implementando escapado estricto anti-XSS previo y renderizado de encabezados, listas, citas/alertas, negritas, código inline, tablas GFM y bloques de código con botón **Copiar**
- [x] 3.2 Actualizar `app/src/ai-copilot/copilot-state.ts` y `app/src/ai-copilot/copilot-state.test.ts` para las 4 fases adaptativas, regla de auto-ejecución acotada (`MAX_AUTO_CHAIN_STEPS = 6` exclusivamente para `🟢` verificados en `Modo Flexible`) y manejo de `lastRollback` de rescate

## 4. Experiencia Comercial en `"neko-ai"` y Conexión de Buffer Visible en `"main"`
- [x] 4.1 Conectar en `app/src/main.ts` la extracción de líneas visibles activas de `xterm.js` hacia `seed_pty_visible_screen_cmd` cuando el usuario enciende `Escucha ON` sobre un shell conectado
- [x] 4.2 Actualizar `app/ai-copilot.html`, `app/src/ai-copilot/copilot.css` y `app/src/ai-copilot/copilot-main.ts` con contenedor centrado (`max-width: 880px`), indicador de 4 fases + botón **Limpiar conversación**, chips rápidos de inicio, salidas de consola colapsables (`exit=0` / `exit!=0`), tarjetas explicativas (*"¿Autorizas ejecutar este comando para…?"* + `audit_warning` + **Autorizar** / **Editar** / **Copiar** / **Omitir** / **Ctrl+C** / **Rollback** a 1 clic) y compositor `<textarea>` multilínea (`Enter` / `Shift+Enter`, `autocomplete="off"`)

## 5. Review and Update Existing Unit Tests (MANDATORY)
- [x] 5.1 Revisar y actualizar las suites existentes en `cargo test` y `npm test` (`vitest`) asegurando cero regresiones

## 6. Run Unit Tests and Verify Local DB (MANDATORY)
- [x] 6.1 Ejecutar `cargo test`, `npm test` y `npm run build` verificando compilación limpia de ambos bundles HTML y pruebas SQLite en memoria
- [x] 6.2 Generar reporte en `openspec/changes/ai-conversational-copilot-v2/reports/2026-11-24-step-6-unit-test-and-db-verification.md`

## 7. Desktop Commands & Real `agy.EXE` Verification (MANDATORY - AGENT MUST EXECUTE)
- [x] 7.1 Ejecutar validación de comandos IPC y **prueba de integración real multi-turno contra `agy.EXE`** verificando consulta directa de OS (incluyendo `2>/dev/null`), flujo progresivo con *dry-run*/rollback y ausencia total de rechazos de esquema
- [x] 7.2 Generar reporte en `openspec/changes/ai-conversational-copilot-v2/reports/2026-11-24-step-desktop-commands-verification.md`

## 8. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)
- [x] 8.1 Ejecutar validación de renderizado DOM y estilos sobre `"main"` y `"neko-ai"` comprobando Markdown con tablas/copiar código, acordeones de consola, compositor multilínea (`autocomplete="off"`), ancho centrado y ausencia de *neon glow*
- [x] 8.2 Generar reporte en `openspec/changes/ai-conversational-copilot-v2/reports/2026-11-24-step-desktop-ui-verification.md`

## 9. Update Technical Documentation (MANDATORY)
- [x] 9.1 Actualizar `docs/plan/integracion-agentes-ia.md`, `docs/project_scope.md`, `docs/design/DESIGN.md` y `docs/design/ui-layout-contract.md` reflejando la arquitectura conversacional progresiva de 4 fases
