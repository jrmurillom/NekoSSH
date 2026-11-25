# Reporte de Pruebas Unitarias y Base de Datos (Step 6)

- **Change:** `ai-copilot-window` (Hito 3)
- **Fecha:** 2026-11-24
- **Superficies:** `desktop-ui`, `desktop-commands`

## 1. Suites Ejecutadas

### 1.1 Frontend (`npm test` — Vitest v3.2.7)
- **Resultado:** `17 passed (17)` archivos de prueba, `125 passed (125)` tests exitosos, `0` fallidos (`Duration: 815ms`).
- **Nueva suite agregada (`src/ai-copilot/copilot-state.test.ts` — 8 tests):**
  - Resolución automática de shell objetivo cuando sigue el foco vs preservación cuando **Pin Shell** está activo (`resolveTargetShell`).
  - Límite de pasos automáticos de solo lectura en `Modo Flexible` (`MAX_AUTO_RECON_STEPS = 5`) y bloqueo estricto en `Modo Estricto` o fuera de fase `Reconnaissance` (`shouldAutoRunReadOnlyStep`).
  - Etiquetado de riesgo en 4 niveles (`formatRiskLabel`) y sanitización de texto plano contra XSS (`escapeHtmlText`).
  - Progresión visual del stepper de 3 fases (`activeStepperStage`).

### 1.2 Backend Rust (`cargo test` — `app_lib`)
- **Resultado global:** `136 passed; 0 failed; 1 ignored` (`finished in 0.07s`).
  - El único test marcado `ignored` es `ai::drivers::agy::tests::smoke_real_agy_probe_models_y_stream_json` (prueba de integración manual contra CLI `agy` real).
- **Subconjunto AI (`cargo test ai::` — `46 passed; 0 failed; 1 ignored`):**
  - `ai::preferences::tests::*` (4 tests):
    - `get_retorna_defaults_en_bd_vacia`: valida valores iniciales en SQLite en memoria, incluyendo `context_max_lines = 200`.
    - `set_y_get_roundtrip_incluyendo_driver_modelo_y_context_lines`: valida persistencia ida y vuelta en SQLite `app_preferences` (`ai_context_max_lines`).
    - `context_max_lines_se_acota_al_rango_seguro`: valida clamping determinista dentro de `50..=1000`.
    - `reset_restaura_defaults`: verifica restauración limpia de preferencias.
  - `ai::pty_tap::tests::*` (4 tests):
    - `modo_privacidad_retiene_cero_bytes_y_purga_al_apagar`: garantiza `0 bytes` en `OFF` y purga inmediata al desactivar escucha.
    - `limpia_ansi_osc_retornos_cr_y_descarta_pantalla_alterna`: limpia secuencias CSI/OSC, resuelve `\r` y descarta bloques de pantalla alterna (`?1049h`..`?1049l`).
    - `presupuesto_head_tail_recorta_salidas_extensas`: verifica recorte *Head + Tail* respetando `context_max_lines`.
    - `centinela_posix_detecta_y_oculta_marcador_incluso_fragmentado_entre_paquetes`: comprueba stripping limpio del marcador `__NEKO_END_<nonce>:<exit>__` partido entre múltiples paquetes TCP y extracción exacta de `exit_code`.
  - `ai::commands::tests::*` (6 tests):
    - Cobertura de `ActiveShellsCatalog`, `validate_and_prepare_readonly_command`, `approve_y_cancel_mission_actualizan_fsm`, `resolve_active_driver`, `HarnessRegistry` y comandos de auditoría.

### 1.3 Compilación de Bundles Multi-Entry y Binario Desktop
- `npm run build` (`tsc && vite build`):
  - `dist/index.html` (`29.18 kB`) + `dist/ai-copilot.html` (`8.53 kB`) + chunks CSS/JS compilados con `0` errores TypeScript en `1.47s`.
- `cargo check --bin app`:
  - Compilación limpia (`Finished dev profile in 4.49s`), validando `capabilities/ai-copilot.json`, registro `generate_handler!` y `WebviewWindowBuilder`.

## 2. Conclusión
Todas las pruebas unitarias frontend, backend y de persistencia SQLite en memoria pasan sin regresiones tras la integración de Hito 3.
