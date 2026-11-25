**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Feature Branch Verification (MANDATORY)

- [x] 0.1 Verificar que el trabajo continúa sobre la rama activa `feature/ai-conversational-copilot-v2` sin tocar `main`

## 1. Telemetría Ambiental `OSC 7`, Conteo de Contexto e Inserción Pasiva en Backend (TDD)

- [x] 1.1 Escribir pruebas unitarias que fallen en `app/src-tauri/src/ai/pty_tap.rs` y `app/src-tauri/src/ai/commands.rs` verificando seguimiento de `current_cwd` OSC 7 incluso con `Escucha OFF` (sin retener stdout), conteo real `captured_lines`, construcción de encabezado ambiental (`profile_name` + `cwd`) y validación de `insert_ai_command_to_pty` sin `\n`/`\r`
- [x] 1.2 Implementar seguimiento de `current_cwd` (`osc7::extract_cwd_paths`) y `captured_lines` en `PtyTapRegistry` (`app/src-tauri/src/ai/pty_tap.rs`), exponerlos en `ActiveShellInfoDto` (`commands.rs`), inyectar encabezado ambiental sanitizado en `run_ai_harness_turn_cmd` y emitir `ai-shells-updated` cuando cambie `current_cwd` en `lib.rs`
- [x] 1.3 Implementar `insert_ai_command_to_pty_cmd` y el canal efímero `send_selection_to_ai_cmd` / `take_pending_ai_selection_cmd` en `app/src-tauri/src/ai/commands.rs` y registrarlos en `tauri::generate_handler![]` (`app/src-tauri/src/lib.rs`)

## 2. Lógica Pura en Copiloto: Streaming Limpio, Historial de Prompts y Etiqueta de Contexto (TDD)

- [x] 2.1 Escribir pruebas en `app/src/ai-copilot/copilot-state.test.ts` para `extractReadableStreamPreview` (ocultamiento de llaves JSON crudas, extracción parcial de `"message"`/`"reasoning"` y fallback `"Preparando respuesta técnica…"`), `formatListenChipLabel` (`Escucha ON · N líneas` / `Escucha OFF`), `stepPromptHistory` (`↑`/`↓` preservando borrador) y `formatQuotedSelectionPrompt`
- [x] 2.2 Implementar dichas funciones puras en `app/src/ai-copilot/copilot-state.ts`

## 3. Experiencia Comercial en `"main"` y `"neko-ai"`

- [x] 3.1 Conectar `Ctrl+Shift+E` en `"main"` (`app/src/main.ts`) sobre `xterm.js` conectado para capturar selección e invocar `send_selection_to_ai_cmd` aun con `Escucha OFF`, y documentar el atajo en `#shortcuts-popover` (`app/index.html`)
- [x] 3.2 Integrar en `app/src/ai-copilot/copilot-main.ts` y `copilot.css` el botón **Insertar en terminal** en la tarjeta HITL, streaming limpio en `#copilot-progress-stream`, etiqueta viva `Escucha ON · N líneas`, auto-ajuste del compositor `<textarea>` hasta `140px`, navegación `↑`/`↓` de prompts previos y recepción de selección (`ai-composer-selection` + `take_pending_ai_selection_cmd`)

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar y asegurar que todas las pruebas unitarias previas de `pty_tap.rs`, `commands.rs`, `osc7.rs`, `orchestrator.rs` y `copilot-state.test.ts` pasen sin regresiones

## 5. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 5.1 Ejecutar `cargo test --lib`, `cargo check --bin app`, `npm test` y `npm run build` desde `app/`, verificando ausencia de cambios en SQLite y generando `openspec/changes/ai-copilot-power-ux/reports/2026-11-25-step-5-unit-test-and-db-verification.md`

## 6. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Verificar mediante pruebas unitarias/IPC que `insert_ai_command_to_pty_cmd`, `send_selection_to_ai_cmd`/`take_pending_ai_selection_cmd`, `ActiveShellInfoDto` (`captured_lines`, `current_cwd`) y `run_ai_harness_turn_cmd` funcionan sin error, generando `openspec/changes/ai-copilot-power-ux/reports/2026-11-25-step-6-desktop-commands-verification.md`

## 7. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 7.1 Verificar ambos entrypoints compilados (`app/dist/index.html` y `app/dist/ai-copilot.html`) con Edge headless (`--headless --dump-dom`) validando estructura limpia y atajo documentado, generando `openspec/changes/ai-copilot-power-ux/reports/2026-11-25-step-7-desktop-ui-verification.md`

## 8. Update Technical Documentation (MANDATORY)

- [x] 8.1 Actualizar `docs/design/ui-layout-contract.md` para documentar `Ctrl+Shift+E`, **Insertar en terminal**, indicador de líneas `Escucha ON · N líneas` y telemetría ambiental no sensible