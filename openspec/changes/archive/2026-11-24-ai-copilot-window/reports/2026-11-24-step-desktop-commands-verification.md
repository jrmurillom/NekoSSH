# Reporte de Verificación de Comandos Desktop (Step 7)

- **Change:** `ai-copilot-window` (Hito 3)
- **Fecha:** 2026-11-24
- **Superficie:** `desktop-commands`
- **Harness de Prueba:** Validación IPC en `app_lib` (`cargo test ai::`) y verificación de esquema/registro en `cargo check --bin app`

## 1. Comandos IPC Nuevos y Extendidos Registrados en `src/lib.rs`

Además de los comandos núcleo del arnés (`ai-harness-core` / `ai-provider-driver-agy`), se registraron en `tauri::generate_handler!`:

1. `open_or_focus_ai_copilot_window_cmd`: crea la `WebviewWindow` `"neko-ai"` apuntando a `ai-copilot.html` o la enfoca si ya existe (instancia única).
2. `set_ai_window_always_on_top_cmd`: alterna `always_on_top` en `"neko-ai"`.
3. `notify_ai_theme_changed_cmd`: emite el evento `nekossh-theme-changed` hacia `"neko-ai"`.
4. `sync_active_shells_cmd`: actualiza `ActiveShellsCatalog` desde `"main"` manteniendo el estado real `is_listening` de `PtyTapRegistry` y emite `ai-shells-updated`.
5. `get_active_shells_cmd`: entrega el `ActiveShellsSnapshotDto` inicial al montar `ai-copilot.html`.
6. `set_pty_listening_cmd`: activa/desactiva `PtyContextTap` opt-in y purga memoria al apagar.
7. `get_pty_context_snapshot_cmd`: devuelve líneas sanitizadas (`Head + Tail` + redacción `SecretRedactor`).
8. `execute_ai_readonly_step_cmd`: valida el comando con `ReadOnlyEnforcer`, arma centinela POSIX con `nonce` único en `PtyTapRegistry` e inyecta en el PTY remoto.
9. `approve_harness_plan_cmd`: valida transición HITL hacia `Executing` en `HarnessRegistry`.
10. `cancel_harness_mission_cmd`: cancela turno activo (`AiTurnCancelRegistry`), limpia centinelas pendientes en `PtyTapRegistry` y reinicia la FSM.

## 2. Capability Aislada y Ciclo de Vida Multi-Ventana

- **`app/src-tauri/capabilities/ai-copilot.json`:** validada por `cargo check --bin app` (`windows: ["neko-ai"]`, permisos restringidos a `core:default`, `clipboard-manager:allow-read-text`, `clipboard-manager:allow-write-text`).
- **Cierre en cascada (`src/lib.rs`):** en `.on_window_event`, cuando `"main"` recibe `WindowEvent::Destroyed`, se cancelan turnos activos en `AiTurnCancelRegistry` y se destruye `"neko-ai"`.
- **Pipeline de salida PTY (`src/lib.rs` y `src/external_edit.rs`):** `emit_filtered_ssh_stdout` filtra el marcador centinela antes de emitir `ssh-stdout` a `xterm.js` y emite `ai-pty-command-finished` al completarse el comando de lectura.

## 3. Evidencia de Ejecución (`cargo test ai::commands::tests` y `cargo test ai::pty_tap::tests`)

- `validate_and_prepare_readonly_step_bloquea_mutaciones_y_envuelve_lecturas`: bloquea comandos mutantes en Fase 1 y envuelve comandos de lectura con el centinela `__NEKO_END_<nonce>:$?__`.
- `active_shells_catalog_sincroniza_escucha_y_foco`: sincroniza múltiples shells preservando estado `is_listening` desde `PtyTapRegistry`.
- `approve_y_cancel_mission_actualizan_fsm`: valida transición explícita de aprobación de plan y limpieza en cancelación.
- `centinela_posix_detecta_y_oculta_marcador_incluso_fragmentado_entre_paquetes`: confirma tolerancia a fragmentación TCP y ocultamiento antes de `xterm.js`.

## 4. Conclusión
Todos los comandos IPC de Hito 3 compilan, pasan las pruebas deterministas de frontera y seguridad, y cuentan con capability aislada verificada por Tauri v2.
