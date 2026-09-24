**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/sftp-transfer-cancellation` (previa autorización explícita de Git si aplica)
- [x] 0.2 Verificar el estado limpio del árbol de trabajo

## 1. Backend TDD: `ActiveTransferRegistry` con Guardián RAII y Cancelación en `FakeSftpStore`

- [x] 1.1 Implementar `ActiveTransferRegistry`, `ActiveTransferGuard` (RAII `Drop`), `TRANSFER_CANCELLED_ERROR` (`"TRANSFER_CANCELLED"`) y los métodos `register(terminal_ids, local_part_path)`, `cancel_all()`, `cancel_for_terminal(terminal_id)` y `cancel_all_and_cleanup_local_parts()` en `app/src-tauri/src/edit_util.rs`, junto con pruebas unitarias exhaustivas (`registro_raii_y_cancelacion_por_terminal_y_exit`)
- [x] 1.2 Extender `FakeSftpStore` en `app/src-tauri/src/fake_sftp.rs` para aceptar un `cancel_flag: Option<&AtomicBool>` en los 3 motores (`upload`, `download` y `scp`) y escribir la prueba de integración `fake_cancelacion_en_tres_operaciones_limpia_part_inmediatamente` verificando la interrupción instantánea y el borrado de `.nekossh.part` (local y remoto)

## 2. Backend: Integración en los 3 Motores SFTP, `close_ssh_session` y `RunEvent::Exit`

- [x] 2.1 Integrar `ActiveTransferRegistry` y la comprobación `cancel_flag.load(Ordering::Acquire)` dentro del bucle de `64 KiB` de `sftp_download_to_user_path_blocking`, `sftp_upload_file_blocking_with_progress` y `sftp_copy_between_sessions` en `app/src-tauri/src/external_edit.rs`, además de exponer el comando Tauri `sftp_cancel_transfer`
- [x] 2.2 Registrar `ActiveTransferRegistry` en `.manage(...)` y `sftp_cancel_transfer` en `tauri::generate_handler!` dentro de `app/src-tauri/src/lib.rs`, conectando `cancel_for_terminal` en `close_ssh_session` y `cancel_all_and_cleanup_local_parts` en `RunEvent::ExitRequested | RunEvent::Exit`

## 3. Frontend TDD & E2E DOM/UX: Botón `[ ✕ ]`, `confirmDialog` y Supresión de Ticks Post-Cancelación

- [x] 3.1 Actualizar `createExplorerStatusController` en `app/src/modules/transfer-progress-helper.ts` para renderizar y reutilizar el botón `.files-status-progress-cancel` (`[ ✕ ]`) en `.files-status-progress-header`, exponer `isTransferCancelledError(err)` y añadir pruebas unitarias en `app/src/modules/transfer-progress-helper.test.ts`
- [x] 3.2 Ampliar `app/src/modules/transfer-progress-e2e.test.ts` validando: (a) presencia y reutilización de nodo DOM del botón `.files-status-progress-cancel` sin recreación en ticks sucesivos, (b) clic en `[ ✕ ]` con rechazo en `confirmDialog` (`"Seguir transfiriendo"`) que mantiene viva la transferencia hasta el 100%, (c) clic en `[ ✕ ]` con confirmación (`"Sí, cancelar"`) que detiene la transferencia, interrumpe lotes restantes en subida múltiple y muestra `"Transferencia cancelada"`, (d) condición de carrera donde la transferencia llega al 100% mientras el `confirmDialog` está abierto (no-op idempotente), y (e) cierre de pestaña SSH que silencia ticks residuales del `Channel`

## 4. Frontend UI: Estilos `.files-status-progress-cancel` y Conexión en `main.ts`

- [x] 4.1 Añadir estilos CSS para `.files-status-progress-cancel` en `app/src/styles.css` respetando `docs/design/DESIGN.md` (sin *neon glow*, hover semántico de peligro suave)
- [x] 4.2 Conectar el manejador `requestCancelActiveTransfer()` con `confirmDialog` e `invoke("sftp_cancel_transfer")` en `app/src/main.ts`, integrando el corte de lotes en `runExplorerUpload`, el manejo de `isTransferCancelledError` en `handleDownloadFile`/`handlePasteScp`, y la invalidación de transferencia en `closeTerminalTab` / `closeAllTerminals`

## 5. Review and Update Existing Unit Tests (MANDATORY)

- [x] 5.1 Revisar y asegurar que todas las suites de pruebas existentes en frontend y backend pasen sin regresiones

## 6. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 6.1 Ejecutar `npm test` (Vitest) y `cargo test` (Rust) verificando 100% de aprobación y generar reporte en `openspec/changes/sftp-transfer-cancellation/reports/`

## 7. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 7.1 Verificar compilación y ejecución de comandos de backend en Rust (`cargo test`) y documentar evidencia en `openspec/changes/sftp-transfer-cancellation/reports/`

## 8. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 8.1 Verificar compilación de producción del frontend (`npm run build`) e integridad visual de `#files-status.is-progress` con el botón `[ ✕ ]`, documentando evidencia en `openspec/changes/sftp-transfer-cancellation/reports/`

## 9. Update Technical Documentation (MANDATORY)

- [x] 9.1 Actualizar documentación técnica si aplica según `docs/documentation-standards.md`
