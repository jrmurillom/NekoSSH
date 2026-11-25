**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Feature Branch Verification (MANDATORY)

- [x] 0.1 Verificar que el trabajo corre sobre la rama activa `feature/ai-conversational-copilot-v2` sin tocar `main`

## 1. `SecretRedactor`: Cobertura Ampliada sin Falsos Positivos (TDD)

- [x] 1.1 Escribir pruebas unitarias que fallen en `app/src-tauri/src/ai/redactor.rs` cubriendo `AWS_SECRET_ACCESS_KEY=...`, claves JSON/YAML (`"password": "..."`, `"api_key": "..."`, `token: ...`), prefijos cloud (`sk-...`, `ghp_...`, `gho_...`, `github_pat_...`, `glpat-...`, `AKIA...`) y casos negativos cortos (ej. `sk-test` no truncado)
- [x] 1.2 Implementar patrones compuestos `*_KEY`/`*_SECRET`/`*_TOKEN`/`*_PASSWORD`, `redact_structured_kv_secrets` (JSON/YAML) y `redact_known_token_prefixes` en `app/src-tauri/src/ai/redactor.rs` preservando seguridad UTF-8

## 2. `PtyTapRegistry` & Limpieza de Sesión: `context_max_lines` Dinámico y `HarnessRegistry::remove` (TDD)

- [x] 2.1 Escribir pruebas unitarias que fallen en `app/src-tauri/src/ai/pty_tap.rs` verificando que `clean_captured_command_output` y `PtyTapRegistry` respeten `set_max_capture_lines` dinámico (`50..1000`, ej. `400` líneas sin recortar a `200`, o `50` recortando con `Head + Tail`)
- [x] 2.2 Añadir `max_capture_lines: Arc<AtomicUsize>` en `PtyTapRegistry` (`app/src-tauri/src/ai/pty_tap.rs`), sincronizarlo al arrancar y al guardar/restaurar `AiPreferencesDto` (`commands.rs`, `lib.rs`), y liberar `harness.remove(&terminal_id)` dentro de `close_ssh_session` (`app/src-tauri/src/lib.rs`)

## 3. Puente Accionable `ProposePlan` en UI (TDD)

- [x] 3.1 Escribir pruebas en `app/src/ai-copilot/copilot-state.test.ts` para `formatPlanSummaryMarkdown` (renderizado Markdown de lista/tabla de pasos) y conversión del primer paso pendiente a `PendingActionState`
- [x] 3.2 Implementar `formatPlanSummaryMarkdown` en `app/src/ai-copilot/copilot-state.ts` y actualizar el manejo de `resp.kind === "propose_plan"` en `app/src/ai-copilot/copilot-main.ts` para mostrar los pasos y habilitar la tarjeta pendiente de autorización humana

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar y asegurar que todas las pruebas unitarias previas de `redactor.rs`, `pty_tap.rs`, `commands.rs`, `orchestrator.rs` y `copilot-state.test.ts` pasen sin regresiones

## 5. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 5.1 Ejecutar `cargo test --lib`, `cargo check --bin app`, `npm test` y `npm run build` desde `app/`, verificando ausencia de cambios en SQLite y generando `openspec/changes/ai-core-hygiene-and-redaction/reports/2026-11-25-step-5-unit-test-and-db-verification.md`

## 6. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Verificar vía pruebas IPC/unitarias que `set_ai_preferences_cmd`/`reset_ai_preferences_cmd` actualizan `max_capture_lines` y `SecretRedactor` redacta salidas combinadas, generando `openspec/changes/ai-core-hygiene-and-redaction/reports/2026-11-25-step-6-desktop-commands-verification.md`

## 7. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 7.1 Verificar el build final en `app/dist/ai-copilot.html` con Edge headless (`--headless --dump-dom`), generando `openspec/changes/ai-core-hygiene-and-redaction/reports/2026-11-25-step-7-desktop-ui-verification.md`

## 8. Update Technical Documentation (MANDATORY)

- [x] 8.1 Revisar `docs/design/ui-layout-contract.md` para confirmar consistencia con la limpieza de `HarnessRegistry` y soporte de `ProposePlan`