# Step 6 Verification Report — Desktop Commands & Core Hygiene

- **Change:** `ai-core-hygiene-and-redaction`
- **Date:** 2026-11-25
- **Surface type:** `desktop-commands`

## Verificaciones Ejecutadas

1. **Redacción ampliada de secretos (`SecretRedactor`):**
   - `ai::redactor::tests::enmascara_claves_compuestas_json_yaml_y_prefijos_cloud_sin_falsos_positivos` (`ok`): confirma enmascaramiento de `AWS_SECRET_ACCESS_KEY=...`, claves estructuradas JSON/YAML (`"password": "..."`, `"api_key": "..."`, `token: ...`), prefijos cloud (`sk-...`, `ghp_...`, `gho_...`, `github_pat_...`, `glpat-...`, `AKIA...`) y preservación sin falsos positivos de cadenas cortas (`sk-test`).
   - `ai::redactor::tests::enmascara_asignaciones_sensibles_y_unicode_multibyte_sin_panic`, `enmascara_bearer_token`, `enmascara_bloque_pem`, `enmascara_credenciales_en_url` (`ok`).
2. **Sincronización dinámica de `context_max_lines` y limpieza de sesión:**
   - `ai::pty_tap::tests::salida_capturada_respeta_limite_dinamico_max_capture_lines` (`ok`): valida que `PtyTapRegistry::set_max_capture_lines` actualiza el `AtomicUsize` acotado a `50..1000` y que `clean_captured_command_output` respeta `400` líneas sin truncar prematuramente y `50` con recorte `Head + Tail`.
   - `ai::commands::tests::comandos_ipc_validacion_y_auditoria` y `ai::preferences::tests::*` (`ok`): verifican la validación IPC y la sincronización de preferencias con `set_ai_preferences_cmd` / `reset_ai_preferences_cmd`.
   - `cargo check --manifest-path app\src-tauri\Cargo.toml --bin app` (`ok`): valida que la firma actualizada de `close_ssh_session` (con `harness: tauri::State<'_, ai::HarnessRegistry>` -> `harness.remove(&terminal_id)`) compila e integra en `tauri::generate_handler![]`.
