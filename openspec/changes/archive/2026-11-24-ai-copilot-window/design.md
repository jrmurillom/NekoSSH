## Context

Los Hitos 1 y 2 dejaron construidos y probados en Rust el arnés determinista (`AgentHarnessFsm`, `ReadOnlyEnforcer`, `CommandRiskGuard`, `SecretRedactor`, `PromptAssembler`) y el puerto `AiProviderDriver` con `AgyCliDriver` validado en vivo contra `agy.EXE v1.5.16`.
El Hito 3 construye la **Ventana Nativa Independiente (`"neko-ai"`)** en Tauri v2 + Vite multi-página y el colector opt-in `PtyContextTap` (con centinela silencioso para lecturas de Fase 1), habilitando de extremo a extremo el flujo de **Reconocimiento (Fase 1)** y **Debate/Aprobación del Plan (Fase 2)** sobre servidores SSH reales sin invadir la terminal principal.

## Goals / Non-Goals

**Goals:**
- Configurar Vite multi-entry (`index.html` + `ai-copilot.html`), capability dedicada `capabilities/ai-copilot.json` y gestión de ciclo de vida multi-ventana en Rust (instancia única, `Always on Top` y cierre en cascada al cerrar `"main"`).
- Sincronizar entre `"main"` y `"neko-ai"` tanto los 8 temas conceptuales (`nekossh-theme`) como el catálogo en vivo de shells (`ActiveShellsCatalog` para padre e hijos de cuadrícula) con soporte de **Pin Shell**.
- Implementar `PtyContextTap` (`src/ai/pty_tap.rs`) estrictamente opt-in (`0 bytes` en `OFF`), con limpieza CSI/OSC ANSI, manejo de `\r`, omisión de pantalla alterna (`?1049h`/`?1049l`), recorte `Head + Tail` (`context_max_lines`) y corredor de lecturas Fase 1 con centinela silencioso POSIX (`__NEKO_END_<nonce>:<exit>__`).
- Construir la UI de `neko-ai` (`app/ai-copilot.html` + `app/src/ai-copilot/`) cumpliendo 100% `DESIGN.md`, `ui-icons` (Lucide outline `currentColor`, cero emojis en chrome) y `base-standards.md` (`autocomplete="off"`, español latino, anti-XSS).
- Habilitar la barra rápida superior (`Shell + Pin`, `Escucha`, `Modo Estricto/Flexible`, `<select>` rápido de `Modelo`), el loop automático de lecturas en modo `Flexible` (y autorizado con retorno automático en `Estricto`), el debate y aprobación del plan en Fase 2 y la vista interna de Ajustes.

**Non-Goals:**
- Ejecución paso a paso de tareas mutantes de **Fase 3**, edición inline de comandos mutantes, envío de `Ctrl+C` de tarea en curso, freno interactivo `HaltedOnError` y disparo de `rollback_cmd` (reservados íntegramente para el **Hito 4**; al aprobar el plan en Hito 3 la FSM queda posicionada en `GuidedExecution` mostrando la vista lista de plan aprobado).
- Ventana acoplada dentro del DOM de `"main"` (descartada por diseño en favor de `WebviewWindow` independiente).

## Decisions

### Decisión 1: Interceptor y Filtro de Centinela en el Hilo Lector de `start_ssh_session` (`pty_tap.rs`)
- **Razón**: El hilo lector de PTY en `src/lib.rs` ya recibe cada fragmento de salida antes de emitir `"ssh-stdout"`. Pasar el fragmento por `PtyTapRegistry::process_chunk(terminal_id, raw_chunk)`:
  1. Si `listening == false` y no hay centinela activo: es un `no-op` de $O(1)$ que devuelve el chunk intacto.
  2. Si hay un comando de lectura corriendo con centinela `__NEKO_END_<nonce>:<code >__`: retiene posibles cortes de línea parciales del marcador, extrae el `exit_code` exacto, oculta el marcador del texto reenviado a `xterm.js` y emite el evento Tauri `"ai-pty-command-finished"` hacia `"neko-ai"` con la salida limpia de ese comando.
  3. Si `listening == true`: alimenta el *ring buffer* limpio (sin ANSI, resolviendo `\r` y omitiendo alternate screen `?1049h`..`?1049l`).
- **Alternativa descartada**: Leer el DOM de `xterm.js` desde `"main"` por mensajes cruzados entre ventanas. Descartado por latencia IPC doble y imposibilidad de ocultar limpiamente el centinela antes de pintar en `xterm.js`.

### Decisión 2: Guardia Inviolable de Fase 1 en `execute_ai_readonly_step_cmd`
- **Razón**: Aunque `neko-ai` invoque la ejecución de una lectura (automática en `Flexible` o autorizada por clic en `Estricto`), el comando IPC en Rust vuelve a validar `ReadOnlyEnforcer::validate_recon_command` + `!CommandRiskGuard::audit_risk(&cmd).is_mutating()` antes de escribir en `SshConnections`. Así es físicamente imposible que este comando ejecute una mutación.

### Decisión 3: Sincronización de Shells y Temas vía Estado Rust + `localStorage`/`emit`
- **Razón**: `"main"` registra altas, bajas, cambios de estado y cambios de foco de `ShellPane` mediante un comando ligero `sync_active_shells_cmd` que actualiza `ActiveShellsCatalog` en Rust y emite `"ai-shells-updated"`. Cuando `"neko-ai"` abre tarde (cuando ya había 3 pestañas abiertas), consulta el snapshot inicial de `ActiveShellsCatalog` sin depender de que `"main"` re-envíe eventos. Para temas, ambas ventanas leen `localStorage.getItem("nekossh-theme")` y reaccionan al evento `storage` + evento Tauri `"nekossh-theme-changed"`.

### Decisión 4: Cierre en Cascada con `on_window_event` sobre `"main"`
- **Razón**: En Tauri v2, `RunEvent::Exit` no se dispara al cerrar `"main"` si `"neko-ai"` sigue abierta. Interceptar `WindowEvent::CloseRequested` / `WindowEvent::Destroyed` de `"main"` para invocar `ai_cancels.cancel_all()` y `neko_ai_win.close()` garantiza apagado atómico de toda la aplicación.

### Decisión 5: Módulos Frontend Puros + `vitest` (`app/src/ai-copilot/`)
- **Razón**: Separar la lógica de presentación/estado (`state-helper.ts`, `sanitize-html.ts` / render DOM seguro con `textContent`) del controlador de eventos Tauri (`copilot-main.ts`) permite probar con `vitest` las reglas de Pin Shell, renderizado de fases, selector de modelos y restricciones de Modo Estricto/Flexible sin depender del runtime nativo.

## Risks / Trade-offs

- **[Riesgo] Fragmentación TCP de la marca `__NEKO_END_<nonce>:0__` en dos lecturas consecutivas del PTY (`buf[..n]`)** → *Mitigación*: `PtyContextTap` mantiene una cola corta de línea pendiente (*tail buffer*) cuando detecta un prefijo parcial de `__NEKO_END_` sin salto de línea final, ensamblándolo en el siguiente `read` sin perder bytes ni filtrarlos a `xterm.js`.
- **[Riesgo] Bucle infinito de lecturas en Modo Flexible si el modelo pide lecturas sin fin** → *Mitigación*: Tope duro de seguridad (`MAX_AUTO_RECON_STEPS = 5`) por turno de objetivo; al alcanzarlo, `neko-ai` pausa la auto-ejecución y pide confirmación humana o transición a plan.

## Migration Plan

- `ai_context_max_lines` en `app_preferences` toma valor por defecto `200` (acotado en rango `50..=1000`).
- `vite.config.ts` añade `rollupOptions.input` manteniendo intacto el bundle principal `index.html`.
