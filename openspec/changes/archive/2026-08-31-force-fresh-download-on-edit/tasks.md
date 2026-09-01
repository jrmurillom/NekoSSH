**Surface types:** desktop-commands

## 0. Feature Branch (Mandatorio)

- [x] 0.1 Crear y cambiar a la rama `feature/force-fresh-download-on-edit`.

## 1. Implementación Backend de Descarga Fresca

- [x] 1.1 Detener watcher previo y limpiar sesión existente para `(terminal_id, remote_path)` en `start_external_edit` (`app/src-tauri/src/external_edit.rs`).
- [x] 1.2 Forzar la descarga fresca desde SFTP al directorio temporal en cada solicitud de edición sin omitir la transferencia.
- [x] 1.3 Registrar la nueva sesión con el baseline del archivo descargado y levantar un nuevo watcher managed (`spawn_file_watcher_managed`).

## 2. Pruebas Unitarias y de Integración (Harness)

- [x] 2.1 Agregar pruebas unitarias en `app/src-tauri/src/edit_session.rs` o harness simulado para validar el reemplazo limpio de sesión y la actualización del baseline al reabrir un archivo.
- [x] 2.2 Verificar que el nuevo watcher managed se active y detecte cambios tras la re-descarga.

## 3. Verificaciones Mandatorias

- [x] 3.1 Step N+1: Ejecutar suite de pruebas unitarias de Rust (`cargo test`) y generar reporte en `openspec/changes/force-fresh-download-on-edit/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md`.
- [x] 3.2 Step desktop-commands: Ejecutar harness de verificación de comandos desktop y generar reporte en `openspec/changes/force-fresh-download-on-edit/reports/YYYY-MM-DD-step-desktop-commands-verification.md`.
- [x] 3.3 Step Last: Actualizar documentación técnica SSOT si aplica.