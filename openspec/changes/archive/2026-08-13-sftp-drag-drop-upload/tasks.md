**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/sftp-drag-drop-upload`
- [x] 0.2 Verificar rama actual con `git branch --show-current`

## 1. Lógica pura de destino y colisiones (TDD)

- [x] 1.1 Escribir unit tests de un helper de arrastre: resolver destino (carpeta → su ruta; archivo → carpeta contenedora; sin fila → cwd) y detectar colisiones de nombre contra un listado dado
- [x] 1.2 Implementar el helper (p. ej. `app/src/modules/explorer-drop-helper.ts`) hasta que los tests pasen
- [x] 1.3 Añadir tests del formateo del texto de confirmación (un archivo vs. varios) y ajustarlo hasta que pasen

## 2. Dataset en las filas del árbol

- [x] 2.1 En `buildExplorerNodeEl` (`app/src/main.ts`), escribir `row.dataset.path` y `row.dataset.isDir` en cada fila, sin alterar el filtro por nombre existente

## 3. Overlay alineado a tokens del tema

- [x] 3.1 Reemplazar el color fijo de `.explorer-dropzone` en `app/src/styles.css` por composición basada en tokens del tema (fondo, borde y transparencia), conservando `backdrop-filter` y `pointer-events: none`
- [x] 3.2 Verificar que `.files-node-row.drag-over` use únicamente tokens del acento activo
- [x] 3.3 Sustituir el icono `📥` de `#explorer-dropzone` por el icono Lucide `upload` según el spec `ui-icons`

## 4. Listeners de arrastre

- [x] 4.1 Suscribir el evento de arrastre del webview de Tauri v2 (`onDragDropEvent`) en la inicialización de `app/src/main.ts`
- [x] 4.2 En arrastre sobre el panel: mostrar overlay solo si la pestaña Archivos está activa y hay sesión conectada; resolver destino con `document.elementFromPoint` normalizando coordenadas por `devicePixelRatio`
- [x] 4.3 Resaltar la carpeta destino y actualizar el texto de destino del overlay
- [x] 4.4 Al salir o cancelar el arrastre: ocultar overlay y limpiar el resaltado

## 5. Flujo de subida con confirmaciones

- [x] 5.1 Al soltar: ocultar overlay y pedir confirmación con `confirmDialog` indicando nombre (o cantidad) y ruta destino
- [x] 5.2 Detectar colisiones listando el destino con `sftp_list_dir` y pedir confirmación de sobreescritura por archivo existente
- [x] 5.3 Subir secuencialmente con `sftp_upload_file` usando el `terminal_id` del shell padre, reportando avance con `setExplorerStatus`
- [x] 5.4 Tolerar fallos individuales: continuar el lote y resumir al final los archivos fallidos
- [x] 5.5 Refrescar el explorador al terminar si se subió al menos un archivo

## 6. Review and Update Existing Unit Tests (MANDATORY)

- [x] 6.1 Revisar tests existentes del explorador y del filtro por nombre; ajustarlos si el `dataset` o el render cambian su comportamiento

## 7. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 7.1 Ejecutar `npm test` y `npx tsc --noEmit` en `app/`
- [x] 7.2 Documentar en `openspec/changes/sftp-drag-drop-upload/reports/YYYY-MM-DD-step-7-unit-test-and-db-verification.md` (DB: N/A si no hay persistencia tocada)

## 8. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 8.1 Verificar `sftp_upload_file` y `sftp_list_dir` con harness o mock local, sin mutar el host SSH compartido de pruebas
- [x] 8.2 Report en `openspec/changes/sftp-drag-drop-upload/reports/YYYY-MM-DD-step-8-desktop-commands-verification.md`

## 9. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [ ] 9.1 Verificar en la app: overlay al arrastrar, destino por carpeta/archivo/fondo, resaltado, confirmación de subida, confirmación de sobreescritura, progreso, fallo parcial, refresco y overlay acorde al tema
- [ ] 9.2 Report en `openspec/changes/sftp-drag-drop-upload/reports/YYYY-MM-DD-step-9-desktop-ui-verification.md`

## 10. Update Technical Documentation (MANDATORY)

- [x] 10.1 Actualizar los SSOT tocados (`docs/design/DESIGN.md` si cambian tokens del overlay, y documentación del explorador) según `documentation-standards.md`; documentar N/A si no aplica
