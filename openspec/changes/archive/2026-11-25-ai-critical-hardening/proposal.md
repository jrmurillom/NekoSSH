## Why

La auditoría integral sobre NekoAI identificó **4 huecos lógicos críticos / de alta prioridad** que comprometen la confiabilidad operativa y la seguridad determinista en escenarios reales de producción:

1. En Windows, cancelar o agotar timeout cuando `agy` se resuelve como `.cmd`/`.bat` (`cmd /C`) mata únicamente al padre `cmd.exe` y bloquea el hilo en `stdout_handle.join()` hasta que el árbol hijo (`agy.exe`/`node.exe`) concluye.
2. Wrappers de ejecución (`xargs`, `timeout`, `nice`, `ionice`, `stdbuf`, `setsid`) y acciones destructivas de logs (`journalctl --vacuum*`, `--rotate`) evaden `CommandRiskGuard`, pudiendo clasificarse erróneamente como `Solo lectura` y auto-ejecutarse en *Modo Flexible*.
3. El estado operativo de tarjeta pendiente (`pendingAction`), contexto de último error (`lastFailedContext`) y último rollback (`lastRollback`) vive en variables globales únicas del WebView; cambiar de pestaña SSH en modo *Seguir foco* o cerrar/reabrir `"neko-ai"` destruye la propuesta pendiente.
4. En shells remotos con `zsh` coloreado (secuencias ANSI intercaladas en el eco del comando), `strip_echo_plumbing` falla y expone la tubería `;__neko_rc=$?...` en `xterm.js`; además, si un comando autorizado se queda esperando entrada interactiva (`[sudo] password`, `[y/N]`) o corre bajo un shell no-POSIX (`fish`), la UI permanece bloqueada sin orientación clara.

## What Changes

- **Terminación real de árbol de procesos y retorno inmediato al cancelar `agy` (`app/src-tauri/src/ai/drivers/agy.rs`)**:
  - En Windows, al cancelar (`cancel_token`) o vencer `timeout`, ejecutar terminación recursiva del árbol de procesos (`taskkill /F /T /PID <pid>` con `CREATE_NO_WINDOW`) además de `child.kill()`, y retornar `Err(AiDriverError::Cancelled)` / `Err(AiDriverError::Timeout)` sin quedar colgado en `join()` sobre pipes retenidos por procesos huérfanos.
- **Blindaje de wrappers y rotación de logs en auditor léxico (`app/src-tauri/src/ai/risk_guard.rs`)**:
  - Desenvolver `timeout`, `nice`, `ionice`, `stdbuf`, `setsid` y segmentos precedidos por `xargs` hasta el binario real ejecutado (saltando argumentos propios como duración `10s` o `-n 10`), impidiendo que comandos como `| xargs kill -9`, `| xargs rm -f` o `timeout 5 systemctl restart nginx` pasen como `Solo lectura`.
  - Clasificar `journalctl --vacuum-time`, `--vacuum-size`, `--vacuum-files` y `--rotate` como `ConfigChange` y bloquearlos en lectura pura.
- **Estado operativo pendiente indexado por `terminal_id` y persistido en RAM de sesión (`app/src-tauri/src/ai/commands.rs`, `app/src/ai-copilot/copilot-state.ts`, `app/src/ai-copilot/copilot-main.ts`)**:
  - Guardar y restaurar el estado operativo de cada sesión (`pending_action`, `last_failed_context`, `last_rollback`, `rollback_draft`) por `terminal_id` tanto en memoria de JS (`Map`) como en un registro volátil RAM en backend (`AiUiSessionStateRegistry` integrado o paralelo a `AiUiFeedRegistry`), evitando perder tarjetas pendientes o rollback al cambiar de pestaña SSH o reabrir `"neko-ai"`.
- **Higiene de eco ANSI en `zsh` y detección de prompts interactivos / shells incompatibles (`app/src-tauri/src/ai/pty_tap.rs`, `app/src/ai-copilot/copilot-main.ts`)**:
  - Ignorar secuencias ANSI CSI dentro de `strip_echo_plumbing` para desnudar limpiamente el eco del wrapper incluso bajo `zsh-syntax-highlighting`.
  - Detectar patrones de espera interactiva (`[sudo] password`, `password:`, `passphrase`, `[y/N]`, `[Y/n]`, `Are you sure`) y errores tempranos de sintaxis POSIX (`fish` `$status`) en chunks activos con centinela armado, emitiendo un evento `ai-pty-waiting-input` para advertir claramente al operador en la tarjeta de ejecución que puede escribir en la terminal o pulsar `Ctrl+C`.

## Capabilities

### New Capabilities
- *(Ninguna)*

### Modified Capabilities
- `ai-provider-driver`: Terminación completa del árbol de procesos y desbloqueo inmediato ante cancelación/timeout en Windows (`AgyCliDriver`).
- `ai-harness-core`: Auditoría de wrappers (`xargs`, `timeout`, `nice`, `ionice`, `stdbuf`, `setsid`) y flags destructivos de `journalctl` en `CommandRiskGuard` / `ReadOnlyEnforcer`.
- `ai-copilot-window`: Supresión de eco POSIX resiliente a resaltado ANSI (`zsh`), aviso en vivo ante prompts interactivos bloqueantes (`sudo` / `[y/N]` / shell `fish`), y preservación por `terminal_id` de tarjeta pendiente/rollback entre cambios de pestaña y reaperturas.

## Impact

- **Backend (`app/src-tauri/src/`)**: `ai/drivers/agy.rs`, `ai/risk_guard.rs`, `ai/pty_tap.rs`, `ai/commands.rs`, `ai/mod.rs`, `lib.rs`.
- **Frontend (`app/src/`)**: `ai-copilot/copilot-state.ts`, `ai-copilot/copilot-state.test.ts`, `ai-copilot/copilot-main.ts`, `ai-copilot/copilot.css`.
