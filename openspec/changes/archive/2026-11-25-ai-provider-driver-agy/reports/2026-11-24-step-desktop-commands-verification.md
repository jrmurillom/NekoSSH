# Reporte de Verificación de Comandos Desktop y Driver Real `agy.EXE` (Step 7)

- **Change:** `ai-provider-driver-agy`
- **Fecha:** 2026-11-24
- **Superficie:** `desktop-commands`

## 1. Verificación en Vivo contra `agy.EXE v1.5.16`

Se ejecutó el smoke test real de integración:
```powershell
cd app\src-tauri; cargo test smoke_real_agy_probe_models_y_stream_json -- --ignored --nocapture
```

**Resultados observados:**
- **`probe_health()`:** `DriverHealthStatus { driver_id: "agy", available: true, version: Some("1.5.16"), message: "agy CLI disponible (1.5.16)" }`.
- **`list_models()`:** `MODELS COUNT: 27` (catálogo real extraído desde `agy models`).
- **`execute_turn()` con `--output-format stream-json`, `--sandbox`, `--disable-slash-commands`, `--json-schema` (archivo temporal RAII) y modelo `gemini-3.8-flash-low`:**
  - Progreso en vivo recibido: `{"kind":"reply_only","message":"NekoSSH ok"}`
  - Resultado estructurado: `AgentTurnOutcome { response: ReplyOnly { message: "NekoSSH ok" }, provider_conversation_id: Some("17678e80-3f8c-4a93-ae40-594dc3772f45"), raw_model_used: Some("gemini-3.8-flash-low") }`
  - `test ai::drivers::agy::tests::smoke_real_agy_probe_models_y_stream_json ... ok` (en 6.14 s).

## 2. Verificación de Comandos IPC, Orquestador y Cancelación

- `ai::commands::tests::resolve_active_driver_resuelve_agy_y_rechaza_desconocido` → `ok`
- `ai::commands::tests::comandos_ipc_validacion_y_auditoria` → `ok`
- `ai::orchestrator::tests::orquestador_no_retiene_mutex_durante_inferencia_y_descarta_si_hubo_cancelacion` → `ok`
- Comandos registrados en `src/lib.rs`: `probe_ai_driver_health_cmd`, `list_ai_models_cmd`, `run_ai_harness_turn_cmd`, `cancel_ai_turn_cmd` y limpieza global `AiTurnCancelRegistry::cancel_all()` en `RunEvent::ExitRequested | RunEvent::Exit`.
