## Why

En entornos Windows, la autenticación SSH con llaves privadas (como archivos `.pem` exportados de PuTTYgen u OpenSSH) falla con un error genérico `[Session(-1)] unknown error` en NekoSSH. Esto ocurre debido a dos fallas principales:
1. Incompatibilidad de `libssh2` en Windows para auto-derivar o ubicar el archivo de clave pública (`.pub`) asociado cuando se pasa una ruta de archivo.
2. Posible corrupción o pérdida de escapado de caracteres de barras invertidas (`\`) en las rutas de archivo de Windows enviadas desde el frontend al backend por Tauri IPC.

## What Changes

- **Validación preventiva de ruta en Rust**: Antes de iniciar la autenticación de clave privada, el backend de Rust comprobará explícitamente si el archivo de la llave existe en el disco (`Path::new(&path).exists()`). Si no existe, lanzará un error descriptivo e informativo del archivo no encontrado en lugar de pasarle una ruta inválida a `libssh2`.
- **Carga de llaves SSH en Memoria**: Modificar el backend de Rust para leer la llave privada (y opcionalmente la pública si existe) a un string en memoria y autenticarse usando la función `userauth_pubkey_memory` en lugar de `userauth_pubkey_file`. Esto elimina de raíz el problema de compatibilidad criptográfica de `libssh2` en Windows.
- **Normalización de Paths en el Frontend**: Normalizar la ruta de la llave en el frontend (sustituyendo barras invertidas `\` por barras normales `/`) antes de enviarla al backend o base de datos, garantizando que el path sea seguro contra pérdidas de escape de caracteres en JSON/Tauri.

## Capabilities

### New Capabilities
<!-- Ninguna nueva capability, solo robustez sobre las existentes -->

### Modified Capabilities
- `connection-profiles`: El CRUD de perfiles y la selección de llaves privadas normalizarán la ruta a barras normales `/`.
- `ssh-terminal`: La autenticación de llave en el backend de Rust validará la existencia física de la llave privada en el disco y la cargará en memoria (`userauth_pubkey_memory`) con fallback al par de llaves si existe archivo `.pub`.

## Impact

- **`app/src-tauri/src/lib.rs`**:
  - Modificar la función interna de conexión SSH para leer el archivo de la llave privada a string, validar su existencia de antemano y aplicar la autenticación mediante memoria.
  - Buscar si existe el archivo `.pub` al lado de la clave privada para pasarlo de forma opcional.
- **`app/src/main.ts`** (o el componente/modal de creación de perfiles):
  - Asegurar la normalización de la ruta de la llave reemplazando `\` por `/` al seleccionarla o guardarla en el formulario.
