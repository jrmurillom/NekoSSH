## Why

En Windows/Tauri WebView, depender de la ruta absoluta del archivo de llave SSH es frágil: el selector de archivos a menudo no expone path real, las barras se escapan mal en IPC y `libssh2` falla con errores genéricos. La solución estable es persistir el material de la llave en SQLite y autenticar desde memoria, sin rutas de SO ni librerías nuevas.

## What Changes

- **BREAKING**: La columna `key_path` en `auth_credentials` se renombra a `private_key` y pasa a almacenar el contenido PEM/texto de la llave privada (no una ruta de archivo). No hay migración de perfiles existentes que usen ruta; esos perfiles se consideran fuera de alcance.
- Al usar "Examinar...", el frontend lee el archivo seleccionado (`FileReader` / blob) y guarda el contenido en `private_key` al persistir el perfil.
- El formulario de perfil **no muestra** el PEM: solo indica que la autenticación por llave está configurada (y permite reemplazarla con Examinar...).
- El backend Rust autentica usando el contenido de `private_key` (archivo temporal efímero + libssh2), sin que el usuario gestione rutas de llave.

## Capabilities

### New Capabilities
<!-- Ninguna capability nueva -->

### Modified Capabilities
- `connection-profiles`: Credenciales por llave almacenan material en `private_key`; UI de perfil oculta el PEM y solo muestra estado "configurado por llave".
- `ssh-terminal`: La sesión SSH autentica con llave desde memoria (`userauth_pubkey_memory`) usando el material persistido, sin depender de rutas de archivo.

## Impact

- **SQLite / schema**: `auth_credentials.key_path` → `auth_credentials.private_key` (contenido de llave).
- **`app/src-tauri/src/lib.rs`**: modelo `Profile`, CRUD SQL, `authenticate_session_once` / connect IPC.
- **`app/src/main.ts`** (+ HTML del modal de perfil si aplica): selector de llave, lectura de contenido, estado visual sin mostrar PEM, payload IPC.
- **Tests** unitarios de perfiles/credenciales que asuman `key_path`.
- **Change previo** `fix-ssh-keyfile` (enfoque por rutas) queda obsoleto respecto a este diseño; no se mezcla.
