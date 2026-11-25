**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Feature Branch Verification (MANDATORY)

- [x] 0.1 Verificar que el trabajo corre sobre la rama activa `feature/ai-conversational-copilot-v2` sin tocar `main`

## 1. Backend: Auditor Léxico sin Falsos Positivos y Alertas Claras (TDD)

- [x] 1.1 Escribir pruebas unitarias que fallen en `app/src-tauri/src/ai/risk_guard.rs` cubriendo `command -v docker 2>&1; systemctl is-active docker containerd 2>&1; docker ps 2>&1; ps -ef | grep -iE 'dockerd|containerd' | grep -v grep 2>&1; true`, `command -V nginx`, `docker --version`, `docker compose ps`, `command rm -rf /tmp/x` (sigue bloqueado) y verificación de mensajes claros en `audit_warning` sin `"Fase de Reconocimiento prohíbe"`
- [x] 1.2 Implementar en `app/src-tauri/src/ai/risk_guard.rs` el manejo correcto de `command -v`/`-V`, filtrado de redirecciones inocuas entre tokens, flags informativos `--version` / subcomandos compuestos seguros de lectura y analizador compartido `inspect_readonly_safety` con mensajes claros para `CommandRiskGuard::evaluate_proposal` (manteniendo prefijo compatible en `ReadOnlyEnforcer::validate_recon_command`)

## 2. Frontend: Etiquetas de Seguridad Autoexplicativas (TDD)

- [x] 2.1 Escribir pruebas que fallen en `app/src/ai-copilot/copilot-state.test.ts` validando las nuevas etiquetas claras de `formatRiskLabel` (`"Solo lectura · No modifica"`, `"Modifica archivos / config"`, `"Impacto en servicio"`, `"Crítico · Destructivo"`)
- [x] 2.2 Actualizar `formatRiskLabel` en `app/src/ai-copilot/copilot-state.ts` con las etiquetas claras sin ambigüedad

## 3. Review and Update Existing Unit Tests (MANDATORY)

- [x] 3.1 Revisar y asegurar que todas las pruebas unitarias previas de `risk_guard.rs`, `commands.rs`, `orchestrator.rs` y `copilot-state.test.ts` pasen sin relajarse

## 4. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 4.1 Ejecutar `cargo test --lib`, `cargo check --bin app`, `npm test` y `npm run build` desde `app/`, verificando que no haya mutación de esquema SQLite y generando `openspec/changes/ai-risk-guard-clarity/reports/2026-11-25-step-4-unit-test-and-db-verification.md`

## 5. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 5.1 Verificar vía pruebas IPC (`audit_command_risk_cmd`, `validate_recon_command_cmd` y `HarnessOrchestrator::execute_turn`) que el diagnóstico Docker retorna `read_only` / `auto_runnable_in_flexible == true` / `audit_warning == None` y que comandos mutantes devuelven alertas claras, registrando evidencia en `openspec/changes/ai-risk-guard-clarity/reports/2026-11-25-step-5-desktop-commands-verification.md`

## 6. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Verificar la compilación final de `dist/ai-copilot.html` con Edge headless (`--dump-dom`) y las aserciones de renderizado de `formatRiskLabel`, generando `openspec/changes/ai-risk-guard-clarity/reports/2026-11-25-step-6-desktop-ui-verification.md`

## 7. Update Technical Documentation (MANDATORY)

- [x] 7.1 Revisar `docs/design/DESIGN.md` y confirmar integridad de tokens/documentación (sin cambios de tokens requeridos; los estilos `.copilot-risk-badge` existentes se conservan intactos)
