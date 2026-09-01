**Surface types:** desktop-commands

## 0. Feature Branch (Mandatorio)

- [x] 0.1 Crear y cambiar a la rama `feature/fix-sftp-binary-upload`.

## 1. Implementación Backend de Subida Segura

- [x] 1.1 Implementar bucle de vaciado explícito `remote.flush()` con reintentos no bloqueantes y `pump_pty` en `sftp_upload_file_blocking` (`app/src-tauri/src/external_edit.rs`).
- [x] 1.2 Implementar verificación de integridad post-subida mediante `sftp.stat()` comparando `stat.size` con el tamaño local en bytes (`data.len() as u64`).
- [x] 1.3 Asegurar manejo de errores descriptivos si ocurre `WouldBlock` agotado o discrepancia en el tamaño reportado.

## 2. Pruebas Unitarias y de Integración (Harness)

- [x] 2.1 Agregar pruebas unitarias en `app/src-tauri/src/fake_sftp.rs` o test suite local para simular la subida y verificación de integridad de archivos binarios (`.zip` ficticio con cabeceras y bytes nulos).
- [x] 2.2 Verificar que el comportamiento ante errores de tamaño o I/O falle de manera segura sin alterar el PTY.

## 3. Verificaciones Mandatorias

- [x] 3.1 Step N+1: Ejecutar suite de pruebas unitarias de Rust (cargo test) y generar reporte en openspec/changes/fix-sftp-binary-upload/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md.
- [x] 3.2 Step desktop-commands: Ejecutar harness de verificación de comandos desktop y generar reporte en openspec/changes/fix-sftp-binary-upload/reports/YYYY-MM-DD-step-desktop-commands-verification.md.
- [x] 3.3 Step Last: Actualizar documentación técnica SSOT si aplica.
