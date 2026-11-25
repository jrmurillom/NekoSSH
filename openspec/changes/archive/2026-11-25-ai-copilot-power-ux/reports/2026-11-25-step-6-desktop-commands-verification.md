# Step 6 Verification Report — Desktop Commands & Telemetry

- **Change:** `ai-copilot-power-ux`
- **Date:** 2026-11-25
- **Surface type:** `desktop-commands`

## Verificaciones Ejecutadas

1. **Seguimiento no intrusivo de `OSC 7` y conteo de líneas (`pty_tap.rs` / `osc7.rs`):**
   - `ai::pty_tap::tests::osc7_actualiza_cwd_incluso_con_escucha_off_y_reporta_telemetria` (`ok`): confirma parsing fragmentado de `OSC 7` aun con `Escucha OFF` reteniendo `0 bytes` de stdout, reporte exacto de `captured_line_count`, y preservación de `current_cwd` al purgar líneas al apagar escucha.
2. **Comandos IPC y telemetría ambiental (`commands.rs` / `lib.rs`):**
   - `ai::commands::tests::telemetria_ambiental_y_canales_rapidos_de_insercion_y_seleccion` (`ok`): verifica `ActiveShellInfoDto` (`captured_lines`, `current_cwd`), `build_effective_turn_context` (encabezado `[Sesión SSH: perfil="..." | cwd="..."]` sanitizado con `SecretRedactor`), `normalize_command_for_passive_insert` (recorte de `\r`/`\n` finales y rechazo vacío) y consumo único en `AiSelectionQueue`.
   - `cargo check --manifest-path app\src-tauri\Cargo.toml --bin app` (`ok`): verifica registro limpio de `insert_ai_command_to_pty_cmd`, `send_selection_to_ai_cmd` y `take_pending_ai_selection_cmd` en `tauri::generate_handler![]`.
