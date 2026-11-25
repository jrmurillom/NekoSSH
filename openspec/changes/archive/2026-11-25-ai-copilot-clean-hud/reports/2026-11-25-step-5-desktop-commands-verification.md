# Verificación Step 5 — Comandos Desktop y Ciclo de Cierre (`neko-ai`)

- **Fecha:** 2026-11-25
- **Cambio OpenSpec:** `ai-copilot-clean-hud`

## Comandos y estado verificado

- `ai::pty_tap::tests::disable_all_listening_apaga_escucha_y_purga_multiples_sesiones` — Verificado en `cargo test --lib`: apaga `listening = false` en múltiples sesiones activas, purga snapshots capturados a `""` e idempotencia al invocarse sin escucha encendida.
- `ai::commands::tests::ai_ui_feed_registry_guarda_acota_y_limpia_por_sesion` — Verificado en `cargo test --lib`: `AiUiFeedRegistry` almacena entradas DTO por sesión, acota al límite `MAX_FEED_ENTRIES_PER_SESSION = 150` y limpia atómicamente por sesión (`clear`).
- `ai::commands::tests::cierre_aislado_de_neko_ai_cancela_turnos_y_apaga_escucha` — Verificado en `cargo test --lib`: `handle_copilot_window_close_state` (invocado por `cleanup_on_copilot_window_closed` en `WindowEvent::Destroyed` de `"neko-ai"` en `app/src-tauri/src/lib.rs`) cancela tokens activos en `AiTurnCancelRegistry`, apaga escucha en `PtyTapRegistry` y retorna `ActiveShellsSnapshotDto` con `is_listening: false` para emitir `"ai-shells-updated"`, conservando memoria de sesión y feed en RAM.
- Registro en Tauri `generate_handler![]`: `get_ai_feed_cmd`, `append_ai_feed_cmd` y `clear_ai_conversation_cmd` compilados y enlazados en `cargo check --bin app`.
