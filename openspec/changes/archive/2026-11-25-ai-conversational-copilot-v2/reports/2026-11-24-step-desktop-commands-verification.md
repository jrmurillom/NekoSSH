# Reporte de Verificación de Comandos Desktop e Integración Real `agy.EXE` (Step 7)

- **Change:** `ai-conversational-copilot-v2`
- **Fecha:** 2026-11-24
- **Superficie:** `desktop-commands`
- **Harness de Prueba:** Comandos IPC en `app_lib` (`cargo test`) + ejecución multi-turno real contra `agy.EXE 1.5.16` en sandbox aislado (`smoke_real_agy_probe_models_y_stream_json`).

## 1. Nuevos Comandos IPC Registrados en `src/lib.rs`

1. `execute_ai_command_cmd`: ejecuta de forma soberana el comando autorizado por el operador (sin listas blancas bloqueantes), envolviéndolo con el centinela POSIX inmune a eco TTY (`%s_%s:%d__`) y rechazando colisiones de doble disparo.
2. `seed_pty_visible_screen_cmd`: recibe líneas visibles limpias del buffer `xterm.js` cuando el operador activa `Escucha ON` (operando solo si `listening == true`, purgado a `0 bytes` en `OFF`).
3. `clear_ai_conversation_cmd`: limpia voluntariamente la bitácora conversacional y `provider_conversation_id` de una sesión.

## 2. Evidencia Real Multi-Turno contra `agy.EXE 1.5.16`

Ejecución real de `cargo test smoke_real_agy_probe_models_y_stream_json -- --ignored --nocapture`:

- **Salud y Catálogo:** `DriverHealthStatus { available: true, version: Some("1.5.16") }`, `MODELS COUNT: 27`.
- **Turno 1 (`"puedes revisar que OS tenemos?"`):**
  - El modelo respondió conforme al esquema v2 (`phase: "diagnosis"`) proponiendo:
    `cat /etc/os-release 2>/dev/null || cat /etc/*release 2>/dev/null; uname -mrs`
  - `CommandRiskGuard::evaluate_proposal` retornó `ProposalSafetyReport { effective_risk: ReadOnly, auto_runnable_in_flexible: true, audit_warning: None }` — **cero falsos bloqueos por `2>/dev/null`**.
- **Turno 2 (con output simulado de `Debian GNU/Linux 12 (bookworm)`):**
  - El modelo continuó la misma sesión (`conversation_id: "3ccaea17-5675-4e24-8101-6262c0a69c23"`) y retornó `ReplyOnly { phase_tag: Some(DirectAnswer), message: "### Identificación de Sistema Operativo y Kernel\n\n| Parámetro | Detalle Técnico |..." }` sin burocracia ni planes innecesarios.

## 3. Conclusión
El arnés conversacional de 4 fases opera de extremo a extremo contra `agy.EXE 1.5.16` real sin errores de esquema ni bloqueos por sintaxis POSIX estándar.
