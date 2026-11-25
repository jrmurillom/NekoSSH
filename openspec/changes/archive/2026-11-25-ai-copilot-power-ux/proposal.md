## Why

Con la arquitectura, seguridad de comandos e higiene del arnés ya consolidadas en producción, **NekoAI** necesita cerrar la brecha entre un copiloto seguro y una herramienta diaria de productividad extrema para SysAdmins Senior. Hoy el operador aún debe activar escucha global o copiar/pegar manualmente fragmentos de terminal, no puede pre-cargar un comando en su prompt sin ejecutarlo de inmediato, ve fragmentos JSON crudos durante el streaming de `agy` y desperdicia el primer turno respondiendo en qué servidor o directorio está parado.

## What Changes

- **"Preguntar a NekoAI sobre selección" (`Ctrl+Shift+E`):** Atajo desde cualquier `ShellPane` conectado de `"main"` que toma el texto seleccionado en `xterm.js`, abre o enfoca `"neko-ai"` y precarga el fragmento como bloque citado en `#copilot-prompt-input` incluso manteniendo `Escucha OFF`.
- **Acción `Insertar en terminal` sin `Enter` en la tarjeta HITL:** Nuevo botón en la tarjeta de autorización (junto a *Autorizar y Ejecutar* y *Copiar*) y comando IPC `insert_ai_command_to_pty_cmd` que inyecta el comando editado/limpio en el PTY activo sin `\n`/`\r`, permitiendo completar rutas con `Tab` o revisar flags en la consola antes de ejecutar.
- **Streaming limpio en `#copilot-progress-stream`:** Parser progresivo tolerante a fragmentos JSON (`extractReadableStreamPreview`) que extrae en tiempo real texto legible de diagnóstico/razonamiento y oculta llaves o campos técnicos crudos del esquema.
- **Telemetría ambiental gratuita (`profile_name` + `cwd` OSC 7) y conteo vivo de contexto:** Seguimiento no intrusivo de `current_cwd` vía `osc7::extract_cwd_paths` y conteo de `captured_lines` en `PtyTapRegistry` / `ActiveShellInfoDto`, inyectados como metadato contextual limpio en `run_ai_harness_turn_cmd` y mostrados en el chip `Escucha ON · N líneas`.
- **Compositor de alta velocidad:** Auto-ajuste vertical de `<textarea>` hasta `140px` al pegar trazas extensas e historial con tecla `↑`/`↓` por sesión en `"neko-ai"`.

## Capabilities

### New Capabilities
- *(Ninguna capability completamente nueva; todas extienden las capacidades existentes del copiloto y el arnés.)*

### Modified Capabilities
- `ai-copilot-window`: Agrega envío de selección desde `"main"` (`Ctrl+Shift+E`) con entrega sin condición de carrera al abrir `"neko-ai"`, botón *Insertar en terminal* sin `Enter`, limpieza progresiva de streaming JSON, micro-etiqueta `Escucha ON · N líneas`, auto-ajuste del compositor hasta `140px` y recuperación de historial con `↑`/`↓`.
- `ai-harness-core`: Incorpora el seguimiento liviano de `current_cwd` mediante `osc7` y conteo de líneas en `PtyTapRegistry`/`ActiveShellInfoDto`, exposición de `insert_ai_command_to_pty_cmd` y envío automático del encabezado ambiental (`profile_name` + `cwd`) en `run_ai_harness_turn_cmd`.

## Impact

- **Backend Rust (`app/src-tauri/src/`):**
  - `ai/pty_tap.rs`, `osc7.rs`, `lib.rs` (`emit_filtered_ssh_stdout`, registro de handlers en `tauri::generate_handler![]`).
  - `ai/commands.rs` (`ActiveShellInfoDto`, `insert_ai_command_to_pty_cmd`, cola efímera `PendingSelectionPrompt` + comando/evento, encabezado ambiental en `run_ai_harness_turn_cmd`).
- **Frontend (`app/`):**
  - `app/index.html` (entrada `Ctrl+Shift+E` en `#shortcuts-popover`).
  - `app/src/main.ts` (`Ctrl+Shift+E` sobre `xterm.js` conectado).
  - `app/src/ai-copilot/copilot-state.ts`, `copilot-main.ts`, `copilot.css`, `copilot-state.test.ts`.
- **Documentación SSOT:** `docs/design/ui-layout-contract.md`.
