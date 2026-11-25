**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Feature Branch Verification (MANDATORY)

- [x] 0.1 Verificar que el trabajo se ejecuta sobre la rama de feature activa (`feature/ai-conversational-copilot-v2`) sin tocar `main`

## 1. Backend: Cierre Aislado Seguro de `neko-ai` y Memoria Volátil de Feed (TDD)

- [x] 1.1 Escribir pruebas unitarias que fallen para `PtyTapRegistry::disable_all_listening` (apaga escucha en múltiples sesiones y purga buffers) y `AiUiFeedRegistry` (append acotado a `MAX_FEED_ENTRIES_PER_SESSION`, get y clear por sesión)
- [x] 1.2 Implementar `PtyTapRegistry::disable_all_listening` en `app/src-tauri/src/ai/pty_tap.rs` y crear `AiUiFeedRegistry` (`get_ai_feed_cmd`, `append_ai_feed_cmd`, limpieza en `clear_ai_conversation_cmd`) registrado en `app/src-tauri/src/lib.rs`
- [x] 1.3 Manejar `WindowEvent::Destroyed` para `AI_COPILOT_WINDOW_LABEL` en `app/src-tauri/src/lib.rs` invocando `ai_cancels.cancel_all()`, `pty_taps.disable_all_listening()` y emitiendo `"ai-shells-updated"` conservando `HarnessRegistry` y `AiUiFeedRegistry`

## 2. Frontend: Lógica Pura y Estructura HTML/CSS Limpia Estilo Copilot Pro (TDD)

- [x] 2.1 Escribir pruebas en `app/src/ai-copilot/copilot-state.test.ts` para `formatStageBadgeText` y estado de renderizado limpio del estado vacío
- [x] 2.2 Simplificar `app/ai-copilot.html` a una única barra superior (`44px` con marca `NekoAI`, selector de shell + Pin, `Limpiar`, `Siempre visible`, `Ajustes`), lienzo de feed limpio y dock inferior unificado con `<textarea>` (`autocomplete="off"`), `Escucha ON/OFF`, `Modo`, selector/refresco de modelo y `Enviar`
- [x] 2.3 Refactorizar `app/src/ai-copilot/copilot.css` para quitar las franjas redundantes, dejar respuestas `.copilot-msg-card.is-assistant` sin caja pesada (`transparent`, sin borde), mantener bordes en burbuja usuario / tarjetas de autorización / rescate / consola, consolidar `.copilot-empty-hero`, respetar temas `DESIGN.md` sin *neon glow* ni scroll nativo, y asegurar respuesta responsive a `500px–520px` y pantalla completa
- [x] 2.4 Actualizar `app/src/ai-copilot/copilot-main.ts` para renderizar `is-assistant` limpio con badge de etapa, hero unificado en estado vacío, rehidratar/sincronizar entradas con `get_ai_feed_cmd` y `append_ai_feed_cmd` al abrir o cambiar shell, y mantener todos los flujos de autorización, edición inline, `Ctrl+C` y rollback

## 3. Review and Update Existing Unit Tests (MANDATORY)

- [x] 3.1 Revisar y actualizar las pruebas unitarias afectadas en Rust (`pty_tap.rs`, `orchestrator.rs`) y TypeScript (`copilot-state.test.ts`, `markdown-lite.test.ts`)

## 4. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 4.1 Ejecutar `cargo test`, `cargo check --bin app`, `npm test` y `npm run build` desde `app/`, verificando que no haya mutaciones de esquema SQLite y registrar reporte en `openspec/changes/ai-copilot-clean-hud/reports/2026-11-25-step-4-unit-test-and-db-verification.md`

## 5. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 5.1 Verificar los nuevos comandos Tauri (`get_ai_feed_cmd`, `append_ai_feed_cmd`, `clear_ai_conversation_cmd`) y el ciclo de limpieza al cerrar `"neko-ai"` mediante pruebas en `app/src-tauri` y registrar evidencia en `openspec/changes/ai-copilot-clean-hud/reports/2026-11-25-step-5-desktop-commands-verification.md`

## 6. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Servir la build compilada y verificar con navegador headless (Edge `--headless --dump-dom`) la renderización real de `ai-copilot.html` (cabecera única, ausencia de `.copilot-toolbar`/`.copilot-stepper`, presencia del dock unificado y estado vacío único), generando `openspec/changes/ai-copilot-clean-hud/reports/2026-11-25-step-6-desktop-ui-verification.md`

## 7. Update Technical Documentation (MANDATORY)

- [x] 7.1 Actualizar `docs/design/DESIGN.md` y `docs/design/ui-layout-contract.md` para reflejar el diseño limpio estilo Copilot Pro y el cierre aislado seguro (Opción A) de `"neko-ai"`
