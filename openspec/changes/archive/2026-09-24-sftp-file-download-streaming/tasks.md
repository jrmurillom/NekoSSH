**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/sftp-file-download-streaming` (previa autorización explícita de Git si aplica)
- [x] 0.2 Verificar el estado limpio del árbol de trabajo

## 1. Backend TDD: Staging Local `.nekossh.part`, Selector Nativo (`rfd`) y Descarga Streaming O(1)

- [x] 1.1 Añadir `rfd = "0.15"` en `app/src-tauri/Cargo.toml` y escribir pruebas unitarias en `app/src-tauri/src/edit_util.rs` para `build_local_staging_part_path(local_path: &Path) -> PathBuf`
- [x] 1.2 Escribir prueba unitaria e integración en `app/src-tauri/src/fake_sftp.rs` (`fake_streaming_download_emite_progreso_y_limpia_part_en_fallo`) que valide la descarga por bloques sin límite de 10 MiB, la emisión de `TransferProgressPayload` (`operation: "download"`), el renombrado atómico local y la eliminación automática del `.nekossh.part` local ante fallos intermedios
- [x] 1.3 Implementar `sftp_pick_download_path(default_name: String) -> Result<Option<String>, String>` y `sftp_download_to_user_path_blocking` / `sftp_download_file_with_progress` en `app/src-tauri/src/external_edit.rs` (con búfer fijo `[0u8; 65536]`, `MAX_WOULD_BLOCK_ATTEMPTS = 400`, bombeo `pump_pty`, staging `.nekossh.part`, validación de integridad y `std::fs::rename`), manteniendo `sftp_download_file_blocking` 100% intacta para `start_external_edit` (`Editar`)
- [x] 1.4 Registrar `sftp_pick_download_path` y `sftp_download_file_with_progress` en `tauri::generate_handler!` dentro de `app/src-tauri/src/lib.rs`
- [x] 1.5 Ajustar validación post-EOF en `sftp_download_to_user_path_blocking` y `FakeSftpStore` para soportar archivos remotos en crecimiento (logs vivos: `written_size >= expected_size`) rechazando truncamientos (`written_size < expected_size`), y añadir prueba unitaria en `fake_sftp.rs` (`fake_streaming_download_soporta_log_en_crecimiento_y_rechaza_truncamiento`)

## 2. Frontend TDD: Soporte de Operación `"download"` y Ciclo de Vida E2E en `#files-status`

- [x] 2.1 Actualizar `app/src/modules/transfer-progress-helper.ts` (`TransferProgressPayload.operation` = `"upload" | "scp" | "download"` y etiqueta `"Descargando"` en `formatTransferHeader`) y ampliar `app/src/modules/transfer-progress-helper.test.ts`
- [x] 2.2 Ampliar `app/src/modules/transfer-progress-e2e.test.ts` para cubrir el ciclo de vida DOM/UX completo de una descarga (`Preparando transferencia…` $\rightarrow$ `Descargando <archivo> (GB/s)` $\rightarrow$ `✅ Descarga completa: <archivo>` $\rightarrow$ auto-cierre a los `3000ms`, y limpieza/error persistente si falla)
- [x] 2.3 Añadir prueba E2E en `app/src/modules/transfer-progress-e2e.test.ts` que valide la exclusión mutua de `transferInProgress` mientras el diálogo `"Guardar como…"` (`sftp_pick_download_path`) está abierto y su liberación inmediata al cancelar

## 3. Frontend UI: Acción `"Descargar"` en Menú Contextual SFTP y Conexión IPC

- [x] 3.1 Añadir el ícono `download` en `app/src/icons.ts` (si no existe en `AppIcons`) y agregar la opción `{ id: "download", label: "Descargar", icon: AppIcons.download }` en el menú contextual de archivos remotos (`!node.isDir`) en `app/src/main.ts` cuando `!transferInProgress`
- [x] 3.2 Implementar `handleDownloadFile(node: ExplorerNodeState)` en `app/src/main.ts` que invoca `sftp_pick_download_path`, aborta limpiamente si el usuario cancela el diálogo, o activa `transferInProgress = true` y ejecuta `sftp_download_file_with_progress` con `new Channel<TransferProgressPayload>()` conectado a `setExplorerTransferProgress`
- [x] 3.3 Blindar `handleDownloadFile` en `app/src/main.ts` activando `transferInProgress = true` antes de invocar `sftp_pick_download_path` dentro de un único bloque `try / finally`

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar y asegurar que todas las pruebas unitarias existentes de edición externa (`edit_session`, `external_edit`, `fake_sftp`) y de frontend pasen sin regresiones

## 5. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 5.1 Ejecutar `npm test` (Vitest) y `cargo test` (Rust) verificando 100% de aprobación y actualizar evidencia en `openspec/changes/sftp-file-download-streaming/reports/`

## 6. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Verificar compilación y ejecución de comandos de backend en Rust (`cargo test`) y documentar evidencia en `openspec/changes/sftp-file-download-streaming/reports/`

## 7. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 7.1 Verificar compilación de producción del frontend (`npm run build`) e integridad visual en `#files-status`, documentando evidencia en `openspec/changes/sftp-file-download-streaming/reports/`

## 8. Update Technical Documentation (MANDATORY)

- [x] 8.1 Actualizar documentación técnica si aplica según `docs/documentation-standards.md`
