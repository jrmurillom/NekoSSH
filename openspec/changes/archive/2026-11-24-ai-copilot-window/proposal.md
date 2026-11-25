## Why

Con el arnés determinista (`ai-harness-core`) y el adaptador aislado sobre `agy` (`ai-provider-driver`) ya operativos en el backend Rust, NekoSSH necesita materializar la ventana nativa independiente multi-monitor (`neko-ai`) y el colector opt-in de terminal (`PtyContextTap`) para que el operador pueda diagnosticar servidores en tiempo real (Fase 1) y debatir/aprobar planes estructurados (Fase 2) sin copiar y pegar manualmente ni obstruir el área de terminales `xterm.js`.

## What Changes

- **Infraestructura Multi-Ventana Tauri v2 + Vite**:
  - Empaquetado multi-página en `app/vite.config.ts` (`index.html` y `ai-copilot.html`) para garantizar resolución sin errores 404 en `tauri build`.
  - Capability dedicada de mínimo privilegio `app/src-tauri/capabilities/ai-copilot.json` para la etiqueta `"neko-ai"`.
  - Gestión de ventana en Rust (`open_or_focus_ai_copilot_window_cmd`, `set_ai_window_always_on_top_cmd`) evitando ventanas duplicadas (`unminimize` + `set_focus`) y cerrando `"neko-ai"` en cascada cuando la ventana `"main"` se destruye (`WindowEvent::Destroyed`).
- **Punto de Entrada y Sincronización Multi-Shell en `"main"`**:
  - Botón disparador en `.footer-actions` con icono Lucide outline (`Sparkles` con `currentColor`, sin emojis) + atajo de teclado `Ctrl+Shift+A` documentado en el popover de ayuda (`?`).
  - Registro reactivo de shells activos (`ActiveShellsCatalog`: `terminal_id`, etiqueta legible de pestaña/celda y estado de conexión) con soporte de **Fijar Shell (*Pin*)** y sincronización instantánea de los 8 temas conceptuales (`nekossh-theme`).
- **Interceptor Opt-In `PtyContextTap` y Centinela Silencioso de Lectura (`src/ai/pty_tap.rs`)**:
  - Privacidad estricta: `0 bytes` retenidos mientras `Escucha = OFF` y purga inmediata al desactivar la escucha o cerrar la sesión SSH.
  - Con `Escucha = ON`: limpieza determinista de secuencias CSI/OSC ANSI, resolución de retornos de carro `\r`, descarte de pantallas alternas (`\x1b[?1049h`...`\x1b[?1049l`) y presupuesto `Head + Tail` configurable (`context_max_lines`, default `200`).
  - **Centinela POSIX silencioso (`__NEKO_END_<nonce>:<exit_code>__`)** para lecturas autorizadas de Fase 1: inyecta el comando verificado por `ReadOnlyEnforcer`, oculta el marcador interno antes de emitir `ssh-stdout` hacia `xterm.js` y entrega automáticamente la salida capturada al concluir, permitiendo diagnóstico automático en modo `Flexible` y retorno automático tras clic en **Autorizar** en modo `Estricto`.
- **Interfaz Visual de `neko-ai` (`app/ai-copilot.html` + `app/src/ai-copilot/`)**:
  - Cumplimiento estricto de `DESIGN.md`, `ui-icons` y `base-standards.md` (iconos Lucide outline `currentColor`, acentos planos sin *neon glow*, scrollbars Cyber-Sakura, `autocomplete="off"` en todos los campos y renderizado seguro anti-XSS sin librerías externas).
  - Barra superior rápida con selector de Shell + Pin, `Always on Top`, toggle `Escucha ON/OFF`, selector rápido de Modo (`Estricto` / `Flexible`) y `<select>` dinámico de Modelo (`list_ai_models_cmd`).
  - Vistas completas para Privacidad (`Escucha OFF`), **Fase 1 (Reconocimiento con streaming en vivo)**, **Fase 2 (Debate de ruta y botón Aprobar Plan)** y panel **Ajustes del Agente** (salud de `agy`, ruta opcional de binario, `context_max_lines` y directivas de Capa 2 con restauración).

## Capabilities

### New Capabilities
- `ai-copilot-window`: Ventana nativa independiente `"neko-ai"`, sincronización multi-shell con Pin y temas, colector `PtyContextTap` opt-in con centinela silencioso de lectura, barra rápida de modo/modelo, ajustes del operador y experiencia completa de Fases 1 y 2.

### Modified Capabilities
- `ai-harness-core`: Inclusión de `context_max_lines` (default `200`) en `AiPreferencesDto` y exposición del comando IPC `approve_harness_plan_cmd` (y `cancel_harness_mission_cmd`) para gobernar la transición formal desde la interfaz gráfica.

## Impact

- **Backend Rust (`app/src-tauri/`)**: Nuevo módulo `src/ai/pty_tap.rs` y catálogo de shells/ventana en `src/ai/window.rs` (o `commands.rs`); intercepción ligera y sin bloqueo en el hilo lector de `start_ssh_session` (`src/lib.rs`) para alimentar `PtyContextTap` y filtrar centinelas activos; cierre coordinado de `"neko-ai"` al cerrar `"main"` y nueva capability `capabilities/ai-copilot.json`.
- **Frontend (`app/`)**: `vite.config.ts` multi-entry, `ai-copilot.html`, módulos TypeScript desacoplados bajo `src/ai-copilot/` con pruebas unitarias en `vitest`, adición de iconos Lucide outline necesarios en `src/icons.ts` y diff mínimo en `index.html` / `src/main.ts`.
- **Dependencias**: Cero paquetes npm ni crates Rust adicionales.
