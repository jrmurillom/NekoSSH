# Reporte de Verificación: Step 6 — Pruebas Unitarias, Integración y E2E DOM/UX

**Fecha:** 2026-09-23  
**Cambio:** `sftp-transfer-progress-streaming`  
**Estado:** APROBADO (100% PASS — Cero Tolerancia a Fallos)

## 1. Suite de Pruebas Frontend (Vitest + E2E DOM/UX)
- **Comando:** `npm test` (`vitest run`)
- **Resultado:** `12 passed (12)` archivos de prueba, `81 passed (81)` tests en `911ms`.
- **Cobertura `transfer-progress-helper.test.ts` (5 suites):**
  - Formateo legible de bytes (`0 B`, `KB`, `MB`, `GB`) y velocidades (`B/s`, `KB/s`, `MB/s`).
  - Acotación segura de porcentajes (`0..100`, manejo de `NaN` y negativos).
  - Construcción de encabezados diferenciados (`Subiendo <archivo>`, `Subiendo <archivo> (2/5)`, `Copiando SCP <archivo>`).
  - Estado Pre-flight (`total_bytes === 0 && percent === 0` -> `"Preparando transferencia…"`) diferenciado de archivo vacío completado (`total_bytes === 0 && percent === 100` -> `"0 B / 0 B"`).
- **Cobertura `transfer-progress-e2e.test.ts` (4 suites de ciclo de vida DOM y UX):**
  - **Reutilización estricta de referencias DOM:** Comprobado que a lo largo de 20 ticks consecutivos de progreso (`5% -> 100%`), `filesStatus.replaceChildren` se invoca exactamente `1` vez (`replaceChildrenCalls === 1`) y los nodos `.files-status-progress-title`, `.files-status-progress-percent`, `.files-status-progress-bar` y `.files-status-progress-meta` conservan su identidad exacta (`===`), garantizando `0` reflows estructurales.
  - **Ciclo de vida UX completo:** Validada la secuencia `Verificando destino… -> Preparando transferencia… -> Streaming 50% -> ✅ Éxito -> Auto-cierre a los 3000ms (+300ms limpieza de clase)`.
  - **Persistencia de estado de error:** Validado que un fallo a mitad de transferencia cambia inmediatamente a `.error` (`❌ ...`) eliminando `.is-progress`, y permanece visible (`is-visible` + `error`) incluso tras `15000ms` sin auto-ocultarse.
  - **Exclusión mutua de concurrencia (`transferInProgress`):** Validado que desde el instante en que se confirma el diálogo de subida (incluso durante `Verificando destino…` antes del primer byte), cualquier intento concurrente de `Pegar scp` o segundo Drag & Drop es rechazado hasta que el bloque `finally` libera el cerrojo.

## 2. Suite de Pruebas Backend (Rust Cargo Test)
- **Comando:** `cargo test` (en `app/src-tauri`)
- **Resultado:** `61 passed; 0 failed; 0 ignored` en `0.06s`.
- **Cobertura destacada:**
  - `edit_util::tests::calculo_porcentaje_seguro_cero_bytes_y_archivos_grandes`: `0 bytes -> 100%` sin división por cero y archivos $>4\text{ GiB}$ (`10 GiB`) mediante aritmética `u128` sin desbordamiento.
  - `edit_util::tests::throttling_temporal_100ms_y_emision_forzada`: Emisión garantizada en `0%` y `100%`, y supresión de ticks intermedios dentro de la ventana de `100ms`.
  - `edit_util::tests::generacion_ruta_staging_part`: Creación de ruta `.nekossh.part` en el mismo directorio remoto.
  - `fake_sftp::tests::fake_streaming_upload_emite_progreso_y_limpia_part_en_fallo`: Verifica emisión de eventos de progreso (`0%..100%`), renombrado atómico al destino final y borrado automático (`unlink`) del archivo `.nekossh.part` si ocurre un corte a mitad de transferencia.
  - `fake_sftp::tests::fake_streaming_scp_copy_emite_progreso_y_limpia_part_en_fallo`: Verifica copia inter-sesión mediante streaming por bloques con limpieza `.nekossh.part` ante fallo.
