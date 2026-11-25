**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)
- [x] 0.1 Crear y posicionarse en la rama de feature `feature/ai-copilot-window` (`git checkout -b feature/ai-copilot-window`)
- [x] 0.2 Verificar que el árbol de trabajo esté posicionado en la rama creada

## 1. Colector `PtyContextTap` Opt-In y Centinela Silencioso de Lectura (TDD Backend)
- [x] 1.1 Extender `AiPreferencesDto` en `src/ai/preferences.rs` con `context_max_lines` (default `200`, acotado `50..=1000`) en SQLite `app_preferences`
- [x] 1.2 Implementar `PtyContextTap` y `PtyTapRegistry` en `src/ai/pty_tap.rs` con pruebas TDD cubriendo `0 bytes` en `OFF`, purga al apagar, limpieza CSI/OSC ANSI, resolución de `\r`, omisión de pantalla alterna (`?1049h`..`?1049l`) y recorte `Head + Tail`
- [x] 1.3 Implementar en `src/ai/pty_tap.rs` el detector y filtro de centinela POSIX (`__NEKO_END_<nonce>:<exit>__`) tolerante a fragmentación TCP de lecturas parciales y conectarlo al bucle lector de `start_ssh_session` en `src/lib.rs`

## 2. Catálogo Multi-Shell, Ciclo de Vida de Ventana `"neko-ai"` y Comandos IPC
- [x] 2.1 Implementar `ActiveShellsCatalog` y comandos de sincronización/escucha/lectura segura (`sync_active_shells_cmd`, `get_active_shells_cmd`, `set_pty_listening_cmd`, `get_pty_context_snapshot_cmd`, `execute_ai_readonly_step_cmd`, `approve_harness_plan_cmd`, `cancel_harness_mission_cmd`) en `src/ai/commands.rs`
- [x] 2.2 Implementar gestión de `WebviewWindow` `"neko-ai"` (`open_or_focus_ai_copilot_window_cmd`, `set_ai_window_always_on_top_cmd`), cierre en cascada al destruir `"main"` (`on_window_event`) y crear `app/src-tauri/capabilities/ai-copilot.json`

## 3. Configuración Multi-Entry Vite y Módulos Frontend Desacoplados (`vitest`)
- [x] 3.1 Consultar `modern-web-guidance` para los patrones HTML/CSS/accesibilidad aplicables y configurar `build.rollupOptions.input` (`index.html` + `ai-copilot.html`) en `app/vite.config.ts`
- [x] 3.2 Agregar iconos Lucide outline necesarios (`Sparkles`, `Pin`, `Shield`, `Zap`, `Ear`/`Radio`, `Lock`, `Unlock`, `RotateCcw`) en `app/src/icons.ts` manteniendo `stroke="currentColor"`
- [x] 3.3 Implementar helpers puros en `app/src/ai-copilot/copilot-state.ts` (selección/Pin de shell, tope `MAX_AUTO_RECON_STEPS`, reglas Modo Estricto vs Flexible y formateo seguro anti-XSS) con suite de pruebas en `vitest`

## 4. Interfaz Nativa `ai-copilot.html` e Integración Mínima en Ventana Principal
- [x] 4.1 Crear `app/ai-copilot.html`, estilos modulares `app/src/ai-copilot/copilot.css` (basados estrictamente en tokens de `styles.css`, sin neon glow y con `autocomplete="off"`) y controlador `app/src/ai-copilot/copilot-main.ts` (barra rápida de Shell/Pin/Escucha/Modo/Modelo, Fases 1 y 2 en vivo y panel Ajustes)
- [x] 4.2 Integrar botón disparador en `.footer-actions` (`app/index.html`), atajo `Ctrl+Shift+A` en el popover de ayuda y notificación reactiva de shells/tema desde `app/src/main.ts`

## 5. Review and Update Existing Unit Tests (MANDATORY)
- [x] 5.1 Revisar y actualizar suites existentes en `cargo test` y `npm test` (`vitest`) garantizando cero regresiones

## 6. Run Unit Tests and Verify Local DB (MANDATORY)
- [x] 6.1 Ejecutar `cargo test`, `npm test` y `npm run build` validando compilación de ambos bundles HTML y persistencia SQLite en memoria
- [x] 6.2 Generar reporte en `openspec/changes/ai-copilot-window/reports/2026-11-24-step-6-unit-test-and-db-verification.md`

## 7. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)
- [x] 7.1 Verificar comandos IPC de ventana, `PtyContextTap`, centinela silencioso `ReadOnly`, catálogo de shells, aprobación de plan y preferencias
- [x] 7.2 Generar reporte en `openspec/changes/ai-copilot-window/reports/2026-11-24-step-desktop-commands-verification.md`

## 8. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)
- [x] 8.1 Ejecutar validación visual y funcional de `"main"` y `"neko-ai"` (compilación/renderizado de vista, sincronización de los 8 temas, controles accesibles Lucide, modos `Estricto`/`Flexible` y `autocomplete="off"`)
- [x] 8.2 Generar reporte en `openspec/changes/ai-copilot-window/reports/2026-11-24-step-desktop-ui-verification.md`

## 9. Update Technical Documentation (MANDATORY)
- [x] 9.1 Actualizar `docs/plan/integracion-agentes-ia.md` (y contratos `DESIGN.md` / `ui-layout-contract.md` en lo referente al botón disparador y ventana secundaria) reflejando el cierre del Hito 3
