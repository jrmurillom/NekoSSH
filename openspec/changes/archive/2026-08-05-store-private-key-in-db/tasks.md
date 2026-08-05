**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear rama `feature/store-private-key-in-db` (o cambiar a ella si ya existe) y dejar atrás el enfoque a medias de `fix-ssh-keyfile` en el working tree
- [x] 0.2 Verificar rama actual con `git branch --show-current`

## 1. Schema y modelo: `key_path` → `private_key`

- [x] 1.1 Actualizar el schema SQLite de `auth_credentials` renombrando/reemplazando `key_path` por `private_key` (TEXT, contenido de llave; sin migración de rutas)
- [x] 1.2 Actualizar el struct/modelo Rust y el tipo TS del perfil para usar `private_key` en lugar de `key_path`
- [x] 1.3 Actualizar INSERT/SELECT/UPDATE del CRUD de perfiles y tests unitarios de perfiles que referencien el campo antiguo

## 2. Autenticación SSH desde memoria

- [x] 2.1 En `authenticate_session_once` (o equivalente), exigir `private_key` no vacío cuando `auth_type` es llave; error claro si falta
- [x] 2.2 Autenticar escribiendo el material a un archivo temporal efímero, llamando `userauth_pubkey_file`, eliminando el temp y propagando el error real de libssh2
- [x] 2.3 Eliminar dependencia de rutas de llave del usuario; la fuente de verdad es `private_key` en BD

## 3. Frontend: cargar llave y ocultar PEM

- [x] 3.1 En "Examinar...", leer el contenido del archivo seleccionado (`FileReader`/blob) y retenerlo en estado del formulario para el save
- [x] 3.2 Al guardar/actualizar, enviar `private_key` con el contenido; si no hubo archivo nuevo, conservar el material ya guardado
- [x] 3.3 Sustituir el input de ruta/PEM por un indicador de UI ("Llave privada configurada" / pendiente) sin mostrar el PEM
- [x] 3.4 Ajustar el payload de connect IPC para pasar `private_key` (contenido) en lugar de path

## 4. Review and Run Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar y ajustar pruebas unitarias afectadas (`cargo test`, `npm run test`) al nuevo campo y auth en memoria
- [x] 4.2 Ejecutar la suite y documentar resultado (incl. estado DB si hubo mutación; restore si aplica)
- [x] 4.3 Report: `openspec/changes/store-private-key-in-db/reports/2026-08-04-step-4-unit-test-and-db-verification.md`

## 5. Desktop Commands Verification (MANDATORY — AGENT MUST EXECUTE)

- [x] 5.1 Probar save/update de perfil con `private_key` poblado y verificar persistencia en SQLite
- [x] 5.2 Probar connect con material válido (éxito) y con `private_key` vacío (error claro)
- [x] 5.3 Report: `openspec/changes/store-private-key-in-db/reports/2026-08-04-step-5-desktop-commands-verification.md`

## 6. Desktop UI Verification (MANDATORY — AGENT MUST EXECUTE)

- [x] 6.1 Crear/editar perfil: Examinar llave, confirmar que la UI solo muestra "configurado por llave" y no el PEM
- [x] 6.2 Guardar, reabrir el modal y verificar que el indicador sigue sin revelar PEM y que la llave se conserva
- [x] 6.3 Conectar al servidor con la llave guardada y verificar sesión sin error genérico de Session(-1)
- [x] 6.4 Report: `openspec/changes/store-private-key-in-db/reports/2026-08-04-step-6-desktop-ui-verification.md`

## 7. Update Technical Documentation (MANDATORY)

- [x] 7.1 Actualizar docs/SSOT relevantes si describen `key_path` o auth por ruta de archivo
