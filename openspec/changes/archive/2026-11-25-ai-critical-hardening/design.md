## Context

Tras auditar extremo a extremo la arquitectura actual de NekoAI (`feature/ai-conversational-copilot-v2` incluyendo `ai-copilot-clean-hud` y `ai-risk-guard-clarity`), identificamos 4 brechas de alta prioridad que afectan confiabilidad en producción:
1. **Procesos huérfanos en Windows (`agy.rs`)**: `child.kill()` sobre `cmd /C agy.cmd` deja vivo al subproceso nieto (`agy.exe`/`node.exe`), reteniendo pipes abiertos y bloqueando `stdout_handle.join()`.
2. **Evasión de `CommandRiskGuard` (`risk_guard.rs`)**: wrappers como `xargs`, `timeout`, `nice`, `ionice`, `stdbuf`, `setsid` y acciones destructivas de `journalctl` (`--vacuum*`, `--rotate`) escapan como `ReadOnly` en *Modo Flexible*.
3. **Volatilidad de estado pendiente en UI (`copilot-main.ts`)**: `pendingAction`, `lastFailedContext` y `lastRollback` residen en variables globales sueltas que `refreshPhaseForTarget()` borra al cambiar de pestaña o al cerrar `"neko-ai"`.
4. **Eco TTY con ANSI y bloqueo silencioso de PTY (`pty_tap.rs`)**: `zsh-syntax-highlighting` intercala códigos CSI rompiendo `strip_echo_plumbing`, y prompts interactivos (`sudo`, `[y/N]`) o shells `fish` dejan `"Ejecutando comando en terminal…"` sin diagnóstico ni salida clara.

## Goals / Non-Goals

**Goals:**
- Garantizar terminación inmediata de todo el árbol de procesos en Windows (`taskkill /F /T /PID`) al cancelar o expirar un turno `agy`, retornando sin espera bloqueante.
- Unificar la extracción de verbo y cola de argumentos (`extract_effective_verb_and_tail`) cubriendo wrappers POSIX (`xargs`, `timeout`, `nice`, `ionice`, `stdbuf`, `setsid`) y purgas `journalctl`.
- Preservar tarjeta pendiente, contexto de error y último rollback por `terminal_id` tanto al alternar pestañas SSH como al cerrar/reabrir `"neko-ai"` en la misma sesión de NekoSSH.
- Hacer `strip_echo_plumbing` inmune a secuencias ANSI CSI/OSC en el eco TTY y detectar prompts interactivos / incompatibilidad `fish` para informar al operador con un aviso en vivo.

**Non-Goals:**
- No implementar un emulador `sudo` ni responder contraseñas desde NekoAI (las credenciales siguen ingresándose únicamente en la terminal principal por seguridad).
- No refactorizar la máquina de estados `ProposePlan` ni ampliar `SecretRedactor` todavía (corresponden al Change 2 `ai-core-hygiene-and-redaction`).

## Decisions

### 1. Terminación de árbol de procesos y retorno no bloqueante en `agy.rs`
- **Decisión:** Extraer helper `terminate_process_tree(child: &mut Child)` que en `#[cfg(windows)]` ejecute `taskkill /F /T /PID <pid>` con `CREATE_NO_WINDOW` seguido de `child.kill()` / `child.wait()`. En las ramas de cancelación y timeout de `execute_turn`, llamar `terminate_process_tree` y retornar de inmediato `Err(AiDriverError::Cancelled)` / `Err(AiDriverError::Timeout(...))` sin bloquear esperando `stdout_handle.join()` ni `stderr_handle.join()`.
- **Alternativa descartada:** Esperar `join()` con polling manual sobre hilos. Complejidad innecesaria cuando el SO cierra los handles al morir los procesos; en casos extremos donde un nieto herede un handle, retornar tras `taskkill /F /T` desbloquea la UI en `< 50ms`.

### 2. Extractor unificado `extract_effective_verb_and_tail` en `risk_guard.rs`
- **Decisión:** Sustituir el par separado `extract_effective_verb` + `args_after_verb` por una única pasada de máquina de estados sobre `&[ShellToken]` que devuelva `(verb, &tokens[tail_start..])`:
  - Salta asignaciones `KEY=VAL`, wrappers `sudo` (`-u`/`-g`), `env`, `nohup`, `time`, `exec`, `setsid`, `stdbuf` (`-i`/`-o`/`-e`), `nice` (`-n`/ajuste numérico), `ionice` (`-c`/`-n`/`-p`), `timeout` (`-s`/`-k` + primer operando posicional de duración), `xargs` (flags con argumento `-n`/`-P`/`-I`/`-d`/`-L`/`-s`/`-E`), y `command` (devolviendo `"command"` solo si lleva `-v`/`-V`).
  - Añade detección en `journalctl` para `--vacuum-time`, `--vacuum-size`, `--vacuum-files`, `--flush`, `--rotate` y `--relinquish-var` como `ConfigChange`.
- **Alternativa descartada:** Agregar verbos `xargs`/`timeout` a `is_mutating_verb`. Provocaría falsos positivos sobre usos benignos como `timeout 5 ping -c 1 host` o `cat lista | xargs echo`.

### 3. Registro volátil por sesión para estado HITL (`AiUiFeedRegistry` extendido)
- **Decisión:** Ampliar `AiUiFeedRegistry` (o añadir mapa hermano `AiUiPendingStateRegistry` por `session_id` dentro de `commands.rs`) con `pending_action: Option<AiPendingActionDto>`, `last_failed: Option<AiFailedContextDto>` y `last_rollback: Option<AiRollbackDto>`, expuestos mediante `get_ai_session_ui_state_cmd` y `save_ai_session_ui_state_cmd`, limpiados en `clear_ai_conversation_cmd` y `close_ssh_session`. En `copilot-main.ts`, `Map<string, SessionUiState>` refleja esta información por `terminal_id` en lugar de variables globales compartidas.

### 4. Higiene ANSI en `strip_echo_plumbing` y detección pasiva de espera interactiva
- **Decisión:**
  - Actualizar `strip_echo_plumbing` para omitir al compactar cualquier secuencia CSI (`\x1b[...<byte final 0x40..0x7e>`) u OSC (`\x1b]...\x07|\x1b\\`), manteniendo `byte_map` exacto sobre los índices originales de `text` (respetando fronteras UTF-8).
  - Añadir a `ActiveSentinel` un campo `waiting_hint_emitted: bool` y una función pura `detect_waiting_input_hint(clean_text: &str) -> Option<&'static str>` que detecte prompts (`[sudo] password`, `password:`, `enter passphrase`, `[y/n]`, `(y/n)`, `are you sure`) y errores de `fish` (`unsupported use of '='`, `fish: unknown command`), emitiendo `ProcessChunkResult { waiting_input_hint }` una sola vez por centinela hacia `"ai-pty-waiting-input"`. En `copilot-main.ts`, complementar con un aviso no intrusivo en la tarjeta activa si llegan prompts interactivos o pasan `> 10s` esperando respuesta.

## Risks / Trade-offs

- **[Duración con signo o bandera desconocida en `timeout`]** → Saltar opciones `--signal=...` / `--kill-after=...` / `-s <val>` / `-k <val>` y consumir un único token de duración no-flag (`isdigit` al inicio), preservando el siguiente token como el comando real.
- **[Falso positivo de `detect_waiting_input_hint` en logs pasivos que contengan la palabra `password:`]** → Activar la heurística únicamente mientras existe un `active_sentinel` en vuelo y evaluar solo las últimas líneas recientes (`tail`) del chunk visible, como aviso informativo (sin abortar el comando).
