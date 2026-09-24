# Reporte de Verificación: Step 5, 6 y 7 — Pruebas Unitarias, Comandos Desktop y UI E2E (`sftp-file-download-streaming`)

**Fecha:** 2026-09-23  
**Cambio:** `sftp-file-download-streaming`  
**Estado:** APROBADO (100% PASS — Cero Tolerancia a Fallos)

## 1. Suite de Pruebas Backend (Rust `cargo test`)
- **Comando:** `cargo test` (en `app/src-tauri`)
- **Resultado:** `64 passed; 0 failed; 0 ignored` en `0.04s`.
- **Nuevas pruebas verificadas:**
  - `edit_util::tests::generacion_ruta_staging_part_local`: Verifica la construcción determinista de `.<filename>.nekossh.part` en el mismo directorio destino local elegido por el usuario.
  - `fake_sftp::tests::fake_streaming_download_emite_progreso_y_limpia_part_en_fallo`:
    - Comprueba que `download_to_local` (usado por `Editar` / `start_external_edit`) mantiene su protección de `10 MiB`, mientras que `stream_download_to_local_with_progress` permite descargar archivos mayores a `10 MiB` (`11 MiB` en la prueba) sin límite de tamaño.
    - Comprueba que ante un corte simulado de red a los `128 KiB`, el archivo `.nekossh.part` local es eliminado automáticamente (`!part_local.exists()`) y cualquier archivo previo en la ruta destino permanece intacto.
    - Comprueba la emisión de `TransferProgressPayload` con `operation: "download"` (`0%` inicial y `100%` final) y el renombrado atómico al destino final.
  - `fake_sftp::tests::fake_streaming_download_soporta_log_en_crecimiento_y_rechaza_truncamiento`:
    - Comprueba que si un archivo se trunca (`written_size < initial_stat_size`), la descarga falla y elimina automáticamente el archivo temporal `.nekossh.part`.
    - Comprueba que si un archivo remoto vivo (`/var/log/nginx/access.log`) recibe nuevas líneas mientras se descarga (`written_size > initial_stat_size` al alcanzar `EOF` limpio), la descarga finaliza con éxito conservando el contenido completo.

## 2. Suite de Pruebas Frontend y Ciclo de Vida E2E DOM/UX (`npm test`)
- **Comando:** `npm test` (`vitest run`)
- **Resultado:** `12 passed (12)` archivos de prueba, `83 passed (83)` tests en `563ms`.
- **Cobertura destacada:**
  - `transfer-progress-helper.test.ts`: Verifica el encabezado `"Descargando <archivo>"` para `operation: "download"`.
  - `transfer-progress-e2e.test.ts`:
    - Verifica el flujo completo de descarga de un archivo de `10.00 GB` (`Preparando transferencia…` $\rightarrow$ `Descargando cluster-snapshot.tar.zst` al `64%` con `6.40 GB / 10.00 GB • 64.0 MB/s` $\rightarrow$ `✅ Descarga completa: cluster-snapshot.tar.zst` $\rightarrow$ auto-cierre a los `3000ms`).
    - Verifica que `transferInProgress` bloquea cualquier intento concurrente de subida o segunda descarga desde el instante en que se abre el diálogo nativo `"Guardar como…"` (`sftp_pick_download_path`), y libera el cerrojo inmediatamente en el bloque `finally` si el usuario cancela el diálogo.

## 3. Compilación de Producción (`npm run build`)
- **Comando:** `npm run build` (`tsc && vite build`)
- **Resultado:** Compilación limpia (`0` errores TypeScript, bundles generados en `1.24s`).
