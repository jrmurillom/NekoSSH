## Context

Este es el **Cambio 3 (`ai-copilot-power-ux` — 🚀 Usabilidad Comercial)** derivado de [auditoria_integral_nekoai.md](file:///C:/Users/Roberto/.gemini/antigravity/brain/6a8d4ebc-d291-4895-b3fa-b5330492bd74/auditoria_integral_nekoai.md) sobre la rama `feature/ai-conversational-copilot-v2`. Los cambios críticos de proceso/auditoría léxica y la higiene de memoria/redacción ya fueron implementados y archivados.

## Goals / Non-Goals

**Goals:**
- Permitir consultas instantáneas sobre trazas concretas seleccionadas en `xterm.js` (`Ctrl+Shift+E`) sin comprometer el modo privacidad (`Escucha OFF`).
- Brindar inyección no ejecutiva en consola (`Insertar en terminal` sin salto de línea) junto a los controles existentes de la tarjeta HITL.
- Eliminar fragmentos JSON crudos en el panel de streaming mientras `agy` responde.
- Aprovechar el perfil conectado y la ruta OSC 7 actual (`osc7::extract_cwd_paths`) como telemetría gratuita en cada turno, y mostrar cuántas líneas reales alimentan `Escucha ON`.
- Hacer cómodo el compositor multilínea (`<= 140px`, navegación `↑`/`↓` de prompts recientes).

**Non-Goals:**
- Cambiar políticas de auto-aprobación (`Strict`/`Flexible`) ni relajar `CommandRiskGuard`.
- Persistir historial de prompts o `cwd` en SQLite (todo el estado rápido de sesión vive en memoria RAM de proceso).
- Sincronizar el árbol SFTP visual con `osc7` (el explorador SFTP conserva su navegación explícita actual conforme a `ui-layout-contract.md`).

## Decisions

### 1. Entrega sin condición de carrera para `Ctrl+Shift+E` (`PendingSelection`)
- **Decisión:** Si el operador presiona `Ctrl+Shift+E` con texto seleccionado mientras `"neko-ai"` está cerrada, el frontend de `"main"` invoca `send_selection_to_ai_cmd(terminalId, selection)` que almacena el borrador sanitizado/acotado en memoria del backend (`AiSelectionQueue`), abre/enfoca `"neko-ai"` y emite el evento `ai-composer-selection`. Al inicializar `"neko-ai"`, además de suscribirse al evento, llama a `take_pending_ai_selection_cmd` para recoger la selección si el evento ocurrió antes de montar el listener.
- **Alternativa descartada:** Solo emitir evento DOM desde `"main"` → fallaría la primera vez que `"neko-ai"` tarda ~200 ms en cargar su WebView.

### 2. Extracción liviana de `cwd` OSC 7 y conteo de líneas en `PtyTapRegistry`
- **Decisión:** `PtySessionTap` mantiene un campo `current_cwd: Option<String>` y un `osc7_leftover: String` mínimo (`<= 512` bytes). Durante `process_chunk`, `crate::osc7::extract_cwd_paths` actualiza `current_cwd` incluso cuando `listening == false`, manteniendo estricto `0 bytes` retenidos de salida stdout cuando `Escucha OFF`. Cuando `current_cwd` cambia o cambia la escucha, se puede reflejar en `ActiveShellInfoDto` (`captured_lines` y `current_cwd` con `#[serde(default)]`).
- **Rationale:** El usuario obtiene conciencia inmediata de directorio (`/var/www`, `/etc/nginx`) en `run_ai_harness_turn_cmd` sin gastar tokens ni revelar contenido de consola en modo privacidad. Se remueve el `#![allow(dead_code)]` parcial en `osc7.rs` al darle uso productivo real.

### 3. Botón `Insertar en terminal` sin envolver centinela
- **Decisión:** `insert_ai_command_to_pty_cmd(terminalId, command)` limpia saltos finales (`trim_end_matches(&['\r', '\n'])`) rechazando payloads vacíos y llama a `crate::write_ssh_raw(&ssh_state, &terminal_id, clean)` sin `\n` ni wrapper `__NEKO_END`. No cambia la fase de la FSM ni arma centinela; la tarjeta HITL permanece disponible por si el usuario decide ejecutar o descartar después.

### 4. Streaming progresivo legible en TypeScript (`extractReadableStreamPreview`)
- **Decisión:** Mantener una función pura en `copilot-state.ts` que recibe el buffer acumulado de chunks `ai-turn-progress`. Si detecta que el contenido inicia con `{` (streaming `--json-schema` de `agy`), extrae progresivamente los valores string de las claves `"message"` o `"reasoning"` (decodificando `\n`, `\t`, `\"`, `\\` de manera tolerante a strings aún abiertos sin comilla final); si todavía no llega ninguna clave textual legible, devuelve `"Preparando respuesta técnica…"`. Si el texto no inicia con `{`, lo muestra tal cual.
- **Alternativa descartada:** Parsear JSON completo en Rust por cada token → innecesariamente complejo cuando los tokens parciales no forman JSON válido hasta el final.

### 5. Auto-resize (`max 140px`), historial `↑`/`↓` e indicador `Escucha ON · N líneas`
- **Decisión:** Se agregan funciones puras en `copilot-state.ts` (`formatListenChipLabel`, `pushPromptHistory`, `stepPromptHistory`, `computeComposerHeightPx`) cubiertas con Vitest. El conteo `captured_lines` se refresca cuando cambian snapshots de shells (`syncActiveShellsToAi` / respuesta al activar escucha / término de comando), mostrando `Escucha ON · 84 líneas` en `#listen-label` y regresando a `Escucha OFF` al apagar.

## Risks / Trade-offs

- **[Riesgo] `Ctrl+Shift+E` entra en conflicto con `xterm.js` o atajos del navegador** → Interceptar explícitamente dentro de `term.attachCustomKeyEventHandler` (`ev.ctrlKey && ev.shiftKey && !ev.altKey && !ev.metaKey && ev.key.toLowerCase() === "e"`) devolviendo `false` y en listener global solo cuando haya foco en consola y texto seleccionado.
- **[Riesgo] Selección gigantesca al pulsar `Ctrl+Shift+E`** → Acotar la selección capturada a las últimas 250 líneas / 12 KiB en el frontend/backend y pasarla por `SecretRedactor` si atraviesa el backend, previniendo congelar el `<textarea>`.
- **[Riesgo] `insert_ai_command_to_pty_cmd` inserta saltos de línea internos en un comando multilínea** → Reemplazar `\r\n` o `\n` finales para nunca auto-enviar `Enter`, y documentar que comandos multilínea conservan su estructura pegada (o enviar advertencia si contienen newlines sin bloque).
