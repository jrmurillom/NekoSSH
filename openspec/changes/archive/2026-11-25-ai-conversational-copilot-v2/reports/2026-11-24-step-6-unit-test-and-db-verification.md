# Reporte de Pruebas Unitarias y Base de Datos (Step 6)

- **Change:** `ai-conversational-copilot-v2`
- **Fecha:** 2026-11-24
- **Superficies:** `desktop-ui`, `desktop-commands`

## 1. Suites Ejecutadas

### 1.1 Frontend (`npm test` — Vitest v3.2.7)
- **Resultado:** `18 passed (18)` archivos de prueba, `129 passed (129)` tests exitosos, `0` fallidos (`Duration: 811ms`).
- **Suites nuevas/actualizadas:**
  - `src/ai-copilot/markdown-lite.test.ts` (3 tests): escapado previo XSS (`<script>`, `<img onerror>`), tablas GFM, listas ordenadas/desordenadas y bloques de código con botón copiar.
  - `src/ai-copilot/copilot-state.test.ts` (9 tests): reglas de auto-ejecución acotada (`MAX_AUTO_CHAIN_STEPS = 6`), redacción natural de tarjetas explicativas (`buildCommandQuestionText`), etapas de 4 fases (`activeFourPhaseStage`), Pin Shell y edición inline.

### 1.2 Backend Rust (`cargo test` — `app_lib`)
- **Resultado global unitario:** `140 passed; 0 failed; 1 ignored` (`finished in 0.08s`).
- **Prueba real contra `agy.EXE` (`smoke_real_agy_probe_models_y_stream_json -- --ignored --nocapture`):** `1 passed; 0 failed` (`finished in 17.15s`).
- **Nuevas validaciones TDD cubiertas:**
  - `ai::risk_guard::tests::recon_permite_comandos_diagnostico_puro_y_flags_entrecomillados`: valida `2>/dev/null` y `2>&1` como lectura inocua.
  - `ai::risk_guard::tests::evaluate_proposal_clasifica_y_alerta_sin_bloquear`: clasifica riesgos y emite `audit_warning` explicativo sin arrojar `Err`.
  - `ai::prompt::tests::ensambla_constitucion_de_4_fases_y_directivas`: comprueba Constitución operativa de 4 fases con *dry-run* y *Health Check*.
  - `ai::schema::tests::sobre_v2_convierte_propuesta_con_purpose_y_mensaje_sin_bloquear_campos_opcionales`: valida esquema v2 (`phase`, `message`, `step`) y degradación segura.
  - `ai::orchestrator::tests::orquestador_eleva_riesgo_y_adjunta_alerta_sin_bloquear_en_modo_flexible`: confirma cero bloqueos duros ante comandos no verificados o mutantes.
  - `ai::pty_tap::tests::seed_visible_screen_solo_siembra_cuando_escucha_esta_on_y_se_purga_al_apagar`: garantiza privacidad `OFF = 0 bytes` y sembrado seguro en `ON`.

### 1.3 Compilación de Bundles y Binario
- `npm run build` (`tsc && vite build`): `dist/index.html` (`29.18 kB`), `dist/ai-copilot.html` (`9.75 kB`), `dist/assets/aiCopilot-DjWsXfPd.css` (`14.56 kB`) y `dist/assets/aiCopilot-BBVAwO-E.js` (`27.66 kB`) en `1.45s` con `0` errores TypeScript.
- `cargo check --bin app`: compilación limpia en `2.25s`.

## 2. Conclusión
Todas las pruebas frontend, backend, persistencia SQLite en memoria e integración real contra `agy.EXE` pasan en verde.
