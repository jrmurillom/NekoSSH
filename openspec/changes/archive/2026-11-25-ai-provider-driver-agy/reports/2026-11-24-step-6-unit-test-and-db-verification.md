# Reporte de Verificación de Pruebas Unitarias y Base de Datos (Step 6)

- **Change:** `ai-provider-driver-agy`
- **Fecha:** 2026-11-24
- **Comando ejecutado:** `cd app\src-tauri; cargo test`

## 1. Resumen de Ejecución

- **Suite unitaria estándar:** `128 passed; 0 failed; 1 ignored` (la prueba marcada `#[ignore]` corresponde al smoke test de integración en vivo contra el binario real `agy.EXE`, ejecutada por separado en el Step 7).
- **Cobertura destacada de Hito 2 + Remediación de Auditoría:**
  - `ai::fsm::tests::freno_automatico_preserva_plan_y_aplica_comando_corregido` — Verifica que `HaltedOnError` conserva el vector `tasks` íntegro y aplica `fixed_command` al reanudar.
  - `ai::fsm::tests::cambio_de_driver_invalida_conversation_id_pero_conserva_turns_portatiles` — Verifica portabilidad de historial al alternar entre drivers con estado y *stateless*.
  - `ai::risk_guard::tests::*` (5 pruebas) — Verifica tokenización consciente de comillas, pipes sin espacio (`|bash`, `| sudo sh`), subcomandos exactos sin falsos positivos (`systemctl restart status-api`) y verbos mutantes adicionales (`tee`, `crontab`, `find -delete`, `docker`).
  - `ai::redactor::tests::enmascara_asignaciones_sensibles_y_unicode_multibyte_sin_panic` — Verifica búsqueda ASCII *case-insensitive* por bytes sin panics UTF-8 frente a `ß`, `İ`, `ñ`.
  - `ai::schema::tests::sobre_plano_convierte_a_agent_response_kind_y_rechaza_invalidos` — Verifica `AgentTurnWireEnvelope` plano sin `oneOf` en raíz y su JSON Schema Draft-07.
  - `ai::drivers::fake::tests::*` y `ai::drivers::agy::tests::*` — Verifican catálogo `agy models`, parser `stream-json` real (`step_update.text_delta`, `result.structured_output`), construcción de flags aislados, errores de autenticación/binario y cancelación.
  - `ai::orchestrator::tests::*` (4 pruebas) — Verifican sanitización previa al driver, bloqueo en Fase 1, elevación forzosa de riesgo, auto-aprobación solo en modo `Flexible` para lectura pura y concurrencia sin bloqueo de `Mutex` con cancelación en vuelo.

## 2. Verificación de Persistencia SQLite (`app_preferences`)

- **Pruebas en memoria (`Connection::open_in_memory()`):**
  - `ai::preferences::tests::get_retorna_defaults_en_bd_vacia` (`active_driver = "agy"`, `selected_model = ""`, `driver_options = {}`, `execution_mode = Strict`).
  - `ai::preferences::tests::set_y_get_roundtrip_incluyendo_driver_y_modelo`.
  - `ai::preferences::tests::reset_restaura_defaults`.
- **Estado de datos locales:** Cero mutaciones sobre bases de datos de usuario; pruebas ejecutadas íntegramente en memoria.
