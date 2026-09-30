**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/sftp-recursive-folder-upload` previa autorización del usuario
- [x] 0.2 Verificar rama activa en el repositorio

## 1. Escaneo Recursivo Local en Backend (TDD)

- [x] 1.1 Crear pruebas unitarias en Rust para la recolección recursiva de archivos locales, cálculo de rutas relativas y metadatos (archivos, carpetas, bytes)
- [x] 1.2 Implementar módulo de escaneo local en Rust (`LocalUploadPlan`, `UploadItem`) con protección contra loops de symlinks y límite de profundidad
- [x] 1.3 Exponer comando Tauri `sftp_scan_local_upload_items` registrado en el runtime de la aplicación

## 2. Creación Idempotente de Directorios Remotos en SFTP

- [x] 2.1 Implementar en Rust la función `sftp_ensure_remote_dir` para crear secuencialmente jerarquías de directorios remotos ausentes (`mkdir -p` en SFTP)
- [x] 2.2 Integrar la creación automática de carpetas destino previas a la transferencia streaming de cada archivo en `sftp_upload_file`
- [x] 2.3 Ejecutar pruebas unitarias de backend (`cargo test`) para certificar la creación correcta y manejo de errores de permisos

## 3. Integración en Frontend: Helpers y Flujo de Subida

- [x] 3.1 Actualizar `app/src/modules/explorer-drop-helper.ts` con funciones para formatear la confirmación de carpetas y colecciones mixtas
- [x] 3.2 Añadir pruebas en `app/src/modules/explorer-drop-helper.test.ts` validando el formateo detallado (número de carpetas, archivos y tamaño)
- [x] 3.3 Refactorizar `runExplorerUpload` en `app/src/main.ts` para ejecutar el escaneo previo, mostrar confirmación transparente con conteo real y subir recursivamente el lote con emisión de progreso

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar suite completa de pruebas unitarias existentes en frontend (`npm run test`) y backend (`cargo test`) asegurando cero regresiones

## 5. Run Unit Tests and Verify Local State (MANDATORY)

- [x] 5.1 Ejecutar suite completa de tests de frontend y backend capturando resultados
- [x] 5.2 Documentar reporte de pruebas en `openspec/changes/sftp-recursive-folder-upload/reports/2026-09-29-step-5-unit-test-and-db-verification.md`

## 6. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Invocar y verificar el comando `sftp_scan_local_upload_items` sobre directorios de prueba locales (incluyendo estructuras complejas tipo web bundle)
- [x] 6.2 Generar reporte de verificación de comandos en `openspec/changes/sftp-recursive-folder-upload/reports/2026-09-29-step-6-desktop-commands-verification.md`

## 7. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 7.1 Verificar el comportamiento visual del diálogo de confirmación y el progreso en tiempo real del lote completo en `#files-status`
- [x] 7.2 Generar reporte de verificación de UI en `openspec/changes/sftp-recursive-folder-upload/reports/2026-09-29-step-7-desktop-ui-verification.md`

## 8. Update Technical Documentation (MANDATORY)

- [x] 8.1 Actualizar documentación permanente del proyecto en `docs/` reflejando la capacidad de subida recursiva de carpetas en SFTP

## 9. Matriz de Casos Borde y Negativos en Backend (Rust)

- [x] 9.1 Implementar prueba `scan_empty_directory_tree` validando el registro correcto de carpetas sin archivos
- [x] 9.2 Implementar prueba `scan_zero_byte_files` verificando que archivos vacíos no rompan el cálculo de bytes ni la transferencia
- [x] 9.3 Implementar prueba `scan_utf8_spaces_and_special_chars` asegurando soporte de rutas con espacios, tildes y símbolos
- [x] 9.4 Implementar prueba `scan_non_existent_path_returns_err` verificando el retorno de error explicativo
- [x] 9.5 Implementar prueba `scan_depth_limit_protection` verificando que el árbol corte limpiamente al exceder el límite de 32 niveles

## 10. Suite de Integración E2E Frontend (Vitest)

- [x] 10.1 Implementar `app/src/modules/explorer-folder-upload.e2e.test.ts` simulando el flujo completo de arrastre, confirmación y subida por lotes
- [x] 10.2 Implementar prueba E2E de cancelación cooperativa a mitad de la subida por lotes
- [x] 10.3 Implementar prueba E2E de colisiones en la raíz con omisión selectiva de carpetas y sus archivos anidados

## 11. Certificación de Regresión Real (Caso techpeople)

- [x] 11.1 Implementar prueba de estrés reproduciendo la estructura exacta de `techpeople` (125+ archivos en `_next`, `assets`, `img`) verificando cero errores de `os error 5` y cálculo exacto de bytes

## 12. Re-ejecución Completa y Generación de Reportes Finales

- [x] 12.1 Re-ejecutar suite completa de tests (`npm run test` y `cargo test`) asegurando 100% de aprobación
- [x] 12.2 Generar reporte de certificación final en `openspec/changes/sftp-recursive-folder-upload/reports/2026-09-29-step-12-exhaustive-robustness-verification.md`
