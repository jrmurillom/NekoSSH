# Reporte de Pruebas Unitarias y Base de Datos (Step 6)

- **Change:** `ai-guided-execution` (Hito 4)
- **Fecha:** 2026-11-24
- **Superficies:** `desktop-ui`, `desktop-commands`

## 1. Suites Ejecutadas

### 1.1 Frontend (`npm test` — Vitest v3.2.7)
- **Resultado:** `17 passed (17)` archivos de prueba, `127 passed (127)` tests exitosos, `0` fallidos (`Duration: 885ms`).
- **Ampliación en `src/ai-copilot/copilot-state.test.ts` (10 tests):**
  - Bloqueo total de auto-ejecución en Fase 3 (`canAutoRunPhase3Step` retorna `false` en `strict` y `flexible`, y `shouldAutoRunReadOnlyStep` rechaza cualquier fase distinta de `reconnaissance`).
  - Priorización determinista de rollback en orden inverso LIFO (`findRecommendedLifoRollbackIndex`) y estados visuales del checklist (`resolveTaskVisualState`: `pending`, `active`, `done`, `rolled_back`, `halted`).
  - Sanitización y validación de comandos editados inline (`sanitizeEditedCommand`).

### 1.2 Backend Rust (`cargo test` — `app_lib`)
- **Resultado global:** `141 passed; 0 failed; 1 ignored` (`finished in 0.08s`).
  - Único test marcado `ignored`: `ai::drivers::agy::tests::smoke_real_agy_probe_models_y_stream_json` (requiere CLI `agy` instalado y autenticado en entorno local).
- **Subconjunto AI (`cargo test ai::` — `51 passed; 0 failed; 1 ignored`):**
  - `ai::schema::tests::deserializa_is_rolled_back_por_defecto_en_falso`: verifica compatibilidad de `is_rolled_back = false` cuando el modelo omite el campo.
  - `ai::fsm::tests::edicion_inline_rollback_idempotente_y_reemplazo_de_cola_pendiente`: prueba elevación de riesgo al editar paso activo, bloqueo contra doble rollback (`is_rolled_back`) y reemplazo de pasos pendientes conservando `0..step_index`.
  - `ai::pty_tap::tests::wrapper_inmune_a_eco_tty_y_comentarios_en_mismo_paquete_tcp`: valida bloque POSIX con comentario `#`, separación `%s_%s:%d__` inmune a eco TTY, rechazo de colisión `try_arm_sentinel` y `apply_sentinel_completion_to_fsm` hacia `Completed`.
  - `ai::orchestrator::tests::orquestador_inyecta_error_en_halted_y_reemplaza_cola_pendiente_sin_auto_aprobar`: valida inyección de `stderr` y `exit_code` en `HaltedOnError` y reemplazo de cola restante sin auto-aprobar.
  - `ai::commands::tests::fase_3_dispatch_ctrl_c_reintento_y_rollback_quirurgico`: valida disparo con edición inline re-auditada, interrupción `Ctrl+C` (`exit_code = 130`), rollback quirúrgico y reanudación.
  - `ai::preferences::tests::*` (4 tests): persistencia SQLite en memoria sin regresiones.

### 1.3 Compilación de Bundles Multi-Entry y Binario Desktop
- `npm run build` (`tsc && vite build`): `dist/index.html` (`29.18 kB`), `dist/ai-copilot.html` (`8.54 kB`), `dist/assets/aiCopilot-C8spK8sw.css` (`13.33 kB`) y `dist/assets/aiCopilot-JuhW07YL.js` (`32.04 kB`) en `1.51s` con `0` errores TypeScript.
- `cargo check --bin app`: compilación limpia en `6.26s`.

## 2. Conclusión
Todas las suites unitarias de frontend, backend y persistencia SQLite pasan en verde sin regresiones.
