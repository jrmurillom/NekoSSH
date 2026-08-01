**Surface types:** desktop-ui, desktop-commands



## 0. Setup: Feature Branch (MANDATORY - FIRST STEP)



- [x] 0.1 Crear/verificar rama `feature/fase3-external-edit-sync` (ya creada en propose si aplica)

- [x] 0.2 Verificar rama actual y working tree listo para cambios de código solo en `app/` (+ docs/openspec de este change)



## 1. Persistencia de preferencias (TDD)



- [x] 1.1 Test-first: migración/esquema `app_preferences` (key/value) y helpers get/set de `preferred_external_editor`

- [x] 1.2 Implementar migración SQLite + commands (o API SQL) de lectura/escritura de preferencia

- [x] 1.3 Tests verdes para get default vacío y set/get round-trip



## 2. Backend SFTP download/upload (TDD)



- [x] 2.1 Test-first: helpers de path temp `edit-sessions/<edit_id>/` + política tamaño (10 MiB) y heurística binaria (sample NUL)

- [x] 2.2 Implementar `sftp_download_file` / `sftp_upload_file` (nombres finales en inglés) sobre Session/SFTP existente por `terminal_id`, chunks no bloqueantes

- [x] ~~(DESCARTADO) 2.3 Tests unitarios + smoke/harness de transfer “si hay fixture local” sin prohibir explícitamente writes al lab SSH — reemplazado por 2.3b y sección 10~~

- [x] 2.3b Tests unitarios de rechazo por tamaño, heurística binaria y path helpers; harness de transfer **solo** con mock/fake SFTP o fixture local (MUST NOT upload/overwrite/delete en el host SSH de pruebas compartido)



## 3. Sesiones de edición + watcher (TDD)



- [x] 3.1 Test-first: registro de edit session (reuse mismo `terminal_id`+`remote_path`), baseline hash/mtime, debounce/coalesce de eventos

- [x] 3.2 Implementar start/stop edit session, watcher (`notify` o equivalente), evento al frontend `edit-session-changed`

- [x] 3.3 Implementar apertura editor preferido + fallback asociación OS

- [x] 3.4 Cleanup best-effort de temp; sweep de huérfanos al startup (TTL según design); no borrar durante upload/confirm

- [x] 3.5 Disconnect mid-edit: stop watchers, cerrar confirm sin subir, aviso, conservar temp según política

- [x] 3.6 Tests verdes de lifecycle (start/reuse/stop/cleanup/disconnect)



## 4. Frontend: explorador + flujo FileZilla + settings



- [x] 4.1 Doble clic en archivo → start edit; doble clic carpeta sin cambio de navegación

- [x] 4.2 Menú contextual B3 ítem “Editar” en archivos

- [x] 4.3 Orquestar: download → open editor → on change → `confirmDialog` A1 “¿Subir al servidor?” → upload → refresh baseline; Cancel/Escape sin subir

- [x] 4.4 Aviso A1 para posible binario; rechazo amable >10 MiB; mensajes de error/disconnect

- [x] 4.5 UI Settings: campo “Editor externo preferido” (path) persistido; labels en español latino

- [x] 4.6 Estilos con tokens existentes; sin `window.confirm` nativo



## 5. Review and Update Existing Unit Tests (MANDATORY)



- [x] 5.1 Revisar tests Rust/TS afectados (SFTP session close, overlays, explorador) y ajustar fixtures

- [x] 5.2 Añadir/ajustar unit tests cubiertos por specs `external-file-edit` y delta `sftp-explorer`



## 6. Run Unit Tests and Verify Local DB (MANDATORY)



- [x] 6.1 Ejecutar `cargo test --manifest-path app/src-tauri/Cargo.toml` (y checks frontend si aplican)

- [x] 6.2 Baseline/restore de SQLite si la migración muta datos de usuario en verificación; documentar N/A o restore

- [x] 6.3 Crear `openspec/changes/fase3-external-edit-sync/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md`



## 7. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)



- [x] ~~(DESCARTADO) 7.1 Invocar download/upload y ciclo edit vía harness/app sin acotar a mock/local — implicaba writes al lab SSH; reemplazado por 7.1b~~

- [x] ~~(DESCARTADO) 7.2 Verificar PTY tras transfer sin exigir mock/local — reemplazado por 7.2b~~

- [x] 7.1b Invocar commands de preferencias + ciclo edit (éxito + error tamaño/path) vía harness/app con **mock/fake SFTP o fixture local only**; MUST NOT upload/replace/delete en el host SSH de pruebas compartido (AGENT MUST EXECUTE)

- [x] 7.2b Verificar (con mock/local) que el modelo de transfer no deja el PTY inutilizable; cleanup de temps **locales** de prueba; documentar N/A si no hay PTY live (AGENT MUST EXECUTE)

- [x] 7.3 Generar `openspec/changes/fase3-external-edit-sync/reports/YYYY-MM-DD-step-desktop-commands-verification.md` (incluir declaración explícita de cero writes al lab SSH)



## 8. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)



- [x] ~~(DESCARTADO) 8.1 Validar “Confirmar sube” contra remoto real del lab — reemplazado por 8.1b (mock/local; sin mutar lab)~~

- [x] 8.1b Validar UX: doble clic archivo / menú Editar → editor abre; guardar → A1 “¿Subir al servidor?”; Cancelar no dispara upload; Confirmar dispara el path de upload **mockeado o documentado como N/A remoto** (sin write al lab). Si hace falta SSH live: solo lectura/download a temp local, o sandbox disposable provisionado por el usuario — default cero writes remotos (AGENT MUST EXECUTE)

- [x] 8.2 Validar settings de editor preferido; rechazo >10 MiB; aviso binario; disconnect mid-edit con aviso — sin mutar archivos del lab SSH (AGENT MUST EXECUTE)

- [x] 8.3 Generar `openspec/changes/fase3-external-edit-sync/reports/YYYY-MM-DD-step-desktop-ui-verification.md` (declarar postura mock/local / cero writes al lab)



## 9. Update Technical Documentation (MANDATORY)



- [x] 9.1 Actualizar `docs/project_scope.md`: Fase 3 = editor externo + sync con confirm; Monaco diferido como Fase 3b

- [x] 9.2 Actualizar `docs/design/ui-layout-contract.md` (Fase 3 sin pestaña Monaco; edición externa + A1 de subida; settings de editor)

- [x] 9.3 Ajustar `DESIGN.md` / README solo si hace falta copy o patrón de confirm de sync

- [x] 9.4 No modificar artefactos del change `chrome-confirms-context-menus`



## 10. Restricción lab SSH en plan de prueba y reportes (MANDATORY)



- [x] 10.1 Asegurar que unit/integration tests de transfer y edit-session usan solo mock/fake SFTP o fixtures locales (sin cliente apuntando writes al lab)

- [x] 10.2 En reportes N+1, desktop-commands y desktop-ui: sección explícita “Lab SSH: cero mutaciones” (qué se mockeó; N/A de write remoto; sin sandbox disposable salvo que el usuario lo provea)

- [x] 10.3 Si el usuario provisiona después un sandbox remoto desechable: documentar path + reglas en el reporte correspondiente; hasta entonces no ejecutar uploads live



## 11. Fix UX: path en confirm de subida (sin overflow)



- [x] 11.1 Extender `confirmDialog` con detalle colapsable (`detailFilename` + `detailFullPath`): filename visible; “ver ruta completa” → textarea readonly

- [x] 11.2 Cablear confirm “Subir cambios” en `main.ts` para usar basename + path completo (dejar de pasar path largo como `impact`)

- [x] 11.3 CSS: dialog sin overflow horizontal; textarea de ruta con wrap/seleccionable; tokens A1 existentes

- [x] 11.4 Actualizar delta spec `external-file-edit` (y DESIGN.md si aplica) con el comportamiento filename + colapsable

- [x] 11.5 `npm run build` en `app/`; nota corta en reports (sin writes al lab SSH)


