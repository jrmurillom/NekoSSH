**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear rama `feature/fix-ssh-keyfile` y cambiar a ella
- [x] 0.2 Verificar rama actual con `git branch --show-current`

## 1. Validación preventiva de existencia de llave en Rust

- [x] 1.1 Modificar la función de conexión SSH en `app/src-tauri/src/lib.rs` para verificar la existencia del archivo de la llave privada utilizando `std::path::Path::new(kp).exists()` antes de inicializar la conexión
- [x] 1.2 Retornar el error real de E/S (std::io::Error) o de la librería libssh2 en lugar de un error genérico (ej. propagar `e.to_string()` del error original de lectura o de la sesión)

## 2. Autenticación robusta de clave por memoria

- [x] 2.1 Cambiar el método de autenticación por llave en `app/src-tauri/src/lib.rs`: en lugar de `sess.userauth_pubkey_file`, leer el archivo de la llave privada a un string (`std::fs::read_to_string`) y propagar el error de lectura exacto si falla
- [x] 2.2 Intentar leer también la llave pública asociada (buscando el archivo `.pub` en la misma ruta si existe) para pasarlo de forma opcional
- [x] 2.3 Invocar `sess.userauth_pubkey_memory` con los contenidos en memoria para autenticar la conexión, y propagar el error exacto de libssh2 si falla la autenticación

## 3. Normalización de rutas de llave en el Frontend

- [x] 3.1 Modificar el componente o formulario de perfil de conexiones en `app/src/main.ts` (o donde se procese el path seleccionado de la llave privada) para normalizar las barras invertidas `\` a barras normales `/`
- [x] 3.2 Asegurar que al guardar o actualizar un perfil, la ruta guardada en SQLite use barras normales `/`

## 4. Review and Run Existing Unit Tests (MANDATORY)

- [x] 4.1 Ejecutar la suite de pruebas unitarias existentes (`cargo test` y `npm run test`) para garantizar que el refactor no introduce regresiones, evitando añadir pruebas dummy/relleno sin información concreta
- [x] 4.2 Report: `openspec/changes/fix-ssh-keyfile/reports/2026-08-04-step-4-unit-test-verification.md`

## 5. Desktop Commands Verification (MANDATORY — AGENT MUST EXECUTE)

- [x] 5.1 Probar el comando de conexión de Tauri con rutas inexistentes y validar que retorna el error de existencia
- [x] 5.2 Probar conexión exitosa simulada o unitaria si aplica
- [x] 5.3 Report: `openspec/changes/fix-ssh-keyfile/reports/2026-08-04-step-5-desktop-commands-verification.md`

## 6. Desktop UI Verification (MANDATORY — AGENT MUST EXECUTE)

- [x] 6.1 Iniciar la app de escritorio y validar que al seleccionar una llave privada desde el explorador del SO, la ruta se autocompleta normalizada con barras `/`
- [x] 6.2 Conectar exitosamente al servidor SSH usando la llave en formato OpenSSH (`4p_key_neko.pem`) y comprobar que la conexión se abre sin el error de Session(-1)
- [x] 6.3 Report: `openspec/changes/fix-ssh-keyfile/reports/2026-08-04-step-6-desktop-ui-verification.md`

## 7. Update Technical Documentation (MANDATORY)

- [x] 7.1 Actualizar las especificaciones principales y docs relevantes si el comportamiento de guardado de llaves sufrió cambios
