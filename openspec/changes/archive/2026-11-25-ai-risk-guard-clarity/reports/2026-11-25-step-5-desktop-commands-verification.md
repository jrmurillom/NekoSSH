# Verificación Step 5 — Auditoría Léxica e IPC (`ai-risk-guard-clarity`)

- **Fecha:** 2026-11-25
- **Cambio OpenSpec:** `ai-risk-guard-clarity`

## Evidencia verificada

- `ai::risk_guard::tests::command_lookup_redirecciones_y_docker_diagnostico_son_solo_lectura_sin_alerta`:
  - Comando compuesto reportado por el usuario (`command -v docker 2>&1; systemctl is-active docker containerd 2>&1; docker ps 2>&1; ps -ef | grep -iE 'dockerd|containerd' | grep -v grep 2>&1; true`) evaluado por `CommandRiskGuard::evaluate_proposal(RiskLevel::ReadOnly, ...)`: `effective_risk == RiskLevel::ReadOnly`, `auto_runnable_in_flexible == true`, `audit_warning == None`, y `ReadOnlyEnforcer::validate_recon_command` retorna `Ok(())`.
  - Validados igualmente `command -v nginx`, `command -V systemctl 2>/dev/null`, `docker --version`, `docker compose ps`, `docker network ls` y `systemctl --version` como `ReadOnly` sin alertas falsas.
  - Verificado que `command rm -rf /tmp/demo` sigue escalando a `RiskLevel::Destructive` y bloqueándose para auto-ejecución.
  - Verificado que `docker rm -f old_box` genera un `audit_warning` claro sin la frase interna `"Fase de Reconocimiento prohíbe"`.
- Pruebas IPC existentes (`ai::commands::tests::comandos_ipc_validacion_y_auditoria` y `ai::commands::tests::validate_and_prepare_readonly_step_bloquea_mutaciones_y_envuelve_lecturas`) continúan verdes.
