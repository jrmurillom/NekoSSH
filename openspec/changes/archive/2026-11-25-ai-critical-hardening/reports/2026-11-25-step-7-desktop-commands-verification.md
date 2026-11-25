# Verificación Step 7 — Blindaje Backend y Comandos IPC (`ai-critical-hardening`)

- **Fecha:** 2026-11-25
- **Cambio OpenSpec:** `ai-critical-hardening`

## Evidencia verificada

1. **Kill-tree Windows no bloqueante (`agy.rs`)**
   - Prueba: `ai::drivers::agy::tests::cancelacion_mata_arbol_de_procesos_y_retorna_rapido_sin_colgar_pipes`
   - Resultado: Shim `.cmd` lanzando subproceso hijo `ping` por `20s` fue terminado recursivamente mediante `taskkill /F /T /PID` retornando `Err(AiDriverError::Cancelled)` en menos de `1s` (suite completa `0.90s`).

2. **Blindaje de wrappers y `journalctl` (`risk_guard.rs`)**
   - Prueba: `ai::risk_guard::tests::wrappers_encadenados_y_journalctl_se_auditan_sin_bypass`
   - Resultado: `xargs kill -9`, `xargs rm -f`, `timeout 10 systemctl restart nginx`, `nice -n 5 rm -rf /tmp/x`, `ionice -c3 cp ...`, `stdbuf -oL apt-get upgrade -y`, `setsid systemctl stop apache2`, `journalctl --vacuum-time=1d` y `journalctl --rotate` quedan bloqueados en auto-ejecución con su nivel real (`Destructive`, `ServiceImpact` o `ConfigChange`); mientras `timeout 5s ping -c 1 8.8.8.8`, `echo hola | xargs` y `journalctl -u nginx -n 20 --no-pager` pasan como `ReadOnly`.

3. **Higiene ANSI en `zsh` y detección de espera interactiva (`pty_tap.rs` + `lib.rs`)**
   - Prueba: `ai::pty_tap::tests::strip_echo_zsh_con_colores_ansi_y_detecta_espera_interactiva`
   - Resultado: Eco `;__neko_rc=$?...` intercalado con secuencias `\x1b[32m` / `\x1b[0m` se suprime antes de `xterm.js`; prompts `[sudo] password for deploy: ` y sintaxis `fish` generan `waiting_input_hint` una única vez por centinela para emitir `"ai-pty-waiting-input"`.

4. **Estado HITL por sesión (`commands.rs`)**
   - Prueba: `ai::commands::tests::ai_ui_feed_registry_preserva_estado_hitl_por_sesion`
   - Resultado: `get_ai_session_ui_state_cmd`, `save_ai_session_ui_state_cmd` y `clear_ai_conversation_cmd` compilados, registrados en `lib.rs` (`cargo check --bin app` ok) y validados en memoria por `session_id`.
