**Surface types:** desktop-ui, desktop-commands

**Dependencia:** change hermano `fase3-external-edit-sync` (flujo confirm → upload SFTP normal). No reescribir ese ciclo; solo delta post-fallo.

## 0. Setup: Feature Branch (MANDATORY - FIRST STEP)

- [x] 0.1 Crear/verificar rama `feature/external-edit-sudo-retry`
- [x] 0.2 Verificar que el árbol incluye el flujo fase3 de edit-session + upload (base para el delta); working tree listo para cambios solo en `app/` (+ docs/openspec de este change)

## 1. Clasificación de errores de upload (TDD)

- [x] 1.1 Test-first: helper que clasifica fallos SFTP/upload como `permission_denied` (candidato a sudo) vs. no elevables (disconnect, not found, etc.)
- [x] 1.2 Implementar clasificación + superficie de error estructurada hacia el frontend (`elevatable` / kind)
- [x] 1.3 Tests verdes: permission → elevable; disconnect/otros → no elevable

## 2. Path elevado: temp remoto + sudo -n cp (TDD)

- [x] 2.1 Test-first: builder del comando no interactivo (`sudo -n cp` o equivalente) con quoting seguro de paths; rechazo de NUL
- [x] 2.2 Implementar upload a temp remoto writable + exec corto en la misma Session + cleanup best-effort del temp remoto
- [x] 2.3 Command/flag explícito (p. ej. `edit_session_upload_with_sudo` o `elevated: true`) que orquesta el path elevado y mapea fallos a `sudo_password_required` / `sudo_failed` / éxito
- [x] 2.4 Tests unitarios + harness **solo** con mock/fake SFTP/exec o fixture local (MUST NOT upload/exec/delete en el host SSH de pruebas compartido)

## 3. Frontend: dialog «Subir con sudo» + errores (TDD / UI)

- [x] 3.1 Tras fallo elevable del upload normal post-confirm: mostrar A1 con primaria «Subir con sudo» + cancelar (reusar `overlays.ts`)
- [x] 3.2 Aceptar → invocar path elevado una vez; éxito → baseline como upload normal; no apilar dialogs
- [x] 3.3 Cancel/Escape en oferta sudo → no eleva; temp dirty conservado; watching puede continuar
- [x] 3.4 Fallo elevado (password / no TTY / sudo failed) → alert error en español; sin UI de password; temp dirty conservado; sin reintento silencioso
- [x] 3.5 Fallos no elevables → alert sin ofrecer sudo
- [x] 3.6 Copy y estilos con tokens existentes; sin `window.confirm` nativo

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar tests Rust/TS de edit-session / upload / overlays afectados y ajustar fixtures
- [x] 4.2 Añadir/ajustar unit tests cubiertos por el delta `external-file-edit` (oferta sudo, path elevado, fallo limpio)

## 5. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 5.1 Ejecutar `cargo test --manifest-path app/src-tauri/Cargo.toml` (y checks frontend si aplican)
- [x] 5.2 Documentar N/A de DB si este change no muta SQLite (sin migración nueva esperada)
- [x] 5.3 Crear `openspec/changes/external-edit-sudo-retry/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md`

## 6. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Invocar clasificación + command/flag elevado (éxito mock, `sudo_password_required`, `sudo_failed`, no-elevable) vía harness/app con **mock/fake only**; MUST NOT mutar el lab SSH (AGENT MUST EXECUTE)
- [x] 6.2 Verificar que el exec elevado mockeado no deja el modelo PTY/Session en estado roto; cleanup de temps **locales** de prueba (AGENT MUST EXECUTE)
- [x] 6.3 Generar `openspec/changes/external-edit-sudo-retry/reports/YYYY-MM-DD-step-desktop-commands-verification.md` (declarar cero writes al lab SSH)

## 7. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 7.1 Validar UX mock/local: fallo de upload elevable → A1 «Subir con sudo»; Cancel no eleva; Aceptar dispara path mockeado; error elevado muestra alert sin password UI — **sin writes al lab** (AGENT MUST EXECUTE)
- [x] 7.2 Validar que upload normal exitoso no muestra oferta sudo; fallo no elevable no muestra oferta sudo (AGENT MUST EXECUTE)
- [x] 7.3 Generar `openspec/changes/external-edit-sudo-retry/reports/YYYY-MM-DD-step-desktop-ui-verification.md` (postura mock/local / cero writes al lab)

## 8. Update Technical Documentation (MANDATORY)

- [x] 8.1 Actualizar `docs/project_scope.md` (o nota de Fase 3) con reintento opcional «Subir con sudo» tras fallo de permisos — sin always-on ni password UI
- [x] 8.2 Ajustar `ui-layout-contract.md` / `DESIGN.md` / README solo si hace falta copy del dialog de elevación
- [x] 8.3 No modificar ni archivar artefactos de `fase3-external-edit-sync` ni otros changes hermanos en este paso

## 9. Restricción lab SSH en plan de prueba y reportes (MANDATORY)

- [x] 9.1 Asegurar que unit/integration del path sudo usan solo mock/fake SFTP/exec o fixtures locales (sin cliente apuntando writes/exec destructivo al lab)
- [x] 9.2 En reportes N+1, desktop-commands y desktop-ui: sección explícita «Lab SSH: cero mutaciones» (qué se mockeó; N/A de write remoto)
- [x] 9.3 Si el usuario provisiona después un sandbox remoto desechable: documentar path + reglas en el reporte; hasta entonces no ejecutar uploads/sudo live contra el lab compartido
