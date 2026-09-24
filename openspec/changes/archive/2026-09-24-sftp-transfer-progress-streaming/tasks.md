**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/sftp-transfer-progress-streaming` (previa autorización explícita de Git si aplica)
- [x] 0.2 Verificar el estado limpio del árbol de trabajo

## 1. Backend TDD: Lógica de Progreso, Throttling y Staging Temporal en Rust

- [x] 1.1 Escribir pruebas unitarias en Rust (`external_edit.rs` / `fake_sftp.rs`) para el cálculo de porcentaje seguro en `u64` (incluyendo archivos de `0 bytes` = `100%` y archivos $>4\text{ GiB}$), generación de ruta temporal `.<nombre>.nekossh.part`, throttling temporal de `100ms` y limpieza en caso de fallo
- [x] 1.2 Definir el struct `TransferProgressPayload` (`operation`, `file_name`, `bytes_transferred`, `total_bytes`, `percent`, `speed_bps`) y helpers de emisión con `tauri::ipc::Channel` y `std::time::Instant`
- [x] 1.3 Refactorizar `sftp_upload_file_blocking` para leer en streaming desde disco (`std::fs::File` con búfer fijo `[0u8; 65536]` en memoria $O(1)$), aplicar límite determinista `MAX_WOULD_BLOCK_ATTEMPTS` en `write`, escribir sobre `.nekossh.part`, validar tamaño con `sftp.stat()`, renombrar al destino final y limpiar `.part` en caso de error
- [x] 1.4 Actualizar el comando Tauri `sftp_upload_file` para recibir `on_progress: tauri::ipc::Channel<TransferProgressPayload>` y propagar la telemetría de progreso

## 2. Backend TDD: Integración de Progreso y Staging Atómico en Copiar/Pegar SCP

- [x] 2.1 Escribir pruebas unitarias para el flujo de copia en streaming con reporte de progreso y manejo de limpieza `.nekossh.part` ante fallos intermedios
- [x] 2.2 Actualizar `sftp_copy_between_sessions` para obtener `total_bytes` vía `sftp_src.stat(&source_path)`, escribir sobre `.nekossh.part` en el servidor destino con límite de reintentos en `WouldBlock`, emitir `TransferProgressPayload` (`operation: "scp"`), verificar tamaño final y renombrar atómicamente (o eliminar `.part` si falla)

## 3. Frontend TDD: Helper de Formateo y Estado del Loader (`transfer-progress-helper`)

- [x] 3.1 Crear `app/src/modules/transfer-progress-helper.test.ts` con pruebas unitarias para formateo de bytes (`B`, `KB`, `MB`, `GB`), velocidad (`KB/s`, `MB/s`), porcentaje seguro (`0%..100%`), etiquetas de operación (`Subiendo` vs `Copiando SCP`) y contador de lote `(i/N)`
- [x] 3.2 Implementar `app/src/modules/transfer-progress-helper.ts` haciendo pasar todas las pruebas de Vitest con tipado estricto TypeScript
- [x] 3.3 Actualizar `formatTransferMetrics` e implementar `createExplorerStatusController` en `app/src/modules/transfer-progress-helper.ts` para mostrar `"Preparando transferencia…"` cuando `total_bytes === 0 && percent === 0` y gestionar de forma determinista el ciclo de vida del DOM de `#files-status`

## 4. Frontend UI: Componente de Progreso en `#files-status` e Integración IPC

- [x] 4.1 Añadir estilos CSS en `app/src/styles.css` para la estructura interna de progreso en `.files-status` (encabezado con nombre/lote/porcentaje, barra de progreso `.files-status-progress-bar` con `var(--color-accent-primary)` plano sin *neon glow*, y línea secundaria de bytes/velocidad)
- [x] 4.2 Unificar el semáforo de concurrencia `transferInProgress` en `app/src/main.ts` para proteger simultáneamente `runExplorerUpload` y `handlePasteScp`, así como deshabilitar "pegar scp" en el menú contextual mientras haya una transferencia activa
- [x] 4.3 Conectar `new Channel<TransferProgressPayload>()` de `@tauri-apps/api/core` en `runExplorerUpload` y `handlePasteScp` para actualizar en tiempo real la barra de progreso y porcentaje dentro de `#files-status`
- [x] 4.4 Blindar la ventana de verificación de colisiones (`sftp_list_dir`) en `runExplorerUpload` activando `transferInProgress = true` y el estado `"Verificando destino…"` desde el instante posterior a la confirmación del usuario
- [x] 4.5 Crear la suite de pruebas E2E de experiencia de usuario y DOM (`app/src/modules/transfer-progress-e2e.test.ts`) validando: (a) identidad y reuso de nodos DOM sin recreación en ticks sucesivos, (b) transición `Preparando` $\rightarrow$ `Progreso` $\rightarrow$ `Éxito` $\rightarrow$ `Auto-dismiss (3000ms)`, (c) persistencia de estado `.error` sin auto-dismiss tras fallo intermedio, y (d) exclusión mutua real entre Upload y Pegar SCP

## 5. Review and Update Existing Unit Tests (MANDATORY)

- [x] 5.1 Revisar y actualizar las suites de pruebas existentes en frontend (`vitest`) y backend (`cargo test`) para asegurar compatibilidad total con las nuevas firmas y comportamientos

## 6. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 6.1 Ejecutar `npm test` (Vitest) y `cargo test` (Rust) verificando 100% de aprobación con la nueva suite E2E de ciclo de vida DOM/UX y actualizar reporte en `openspec/changes/sftp-transfer-progress-streaming/reports/2026-09-23-step-6-unit-test-and-db-verification.md`

## 7. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 7.1 Ejecutar ejemplos/harness de verificación de comandos SFTP y transferencia local/mock (`cargo test` / `smoke_edit_session_local`) y documentar evidencia en `openspec/changes/sftp-transfer-progress-streaming/reports/2026-09-23-step-desktop-commands-verification.md`

## 8. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 8.1 Verificar compilación de frontend (`npm run build`) e integridad visual del componente de progreso en el explorador SFTP, documentando evidencia actualizada en `openspec/changes/sftp-transfer-progress-streaming/reports/2026-09-23-step-desktop-ui-verification.md`

## 9. Update Technical Documentation (MANDATORY)

- [x] 9.1 Actualizar documentación técnica si aplica según `docs/documentation-standards.md`
