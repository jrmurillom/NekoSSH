## Context
Actualmente, las conexiones que utilizan claves privadas en Windows arrojan un error desconocido `[Session(-1)]` debido a que `libssh2` falla al derivar automáticamente o encontrar las claves públicas. Además, las rutas de los archivos pueden sufrir un escape incorrecto de barras invertidas cuando se transfieren desde la interfaz de usuario (UI) o la base de datos hacia el backend.

## Goals / Non-Goals
**Goals:**
- Solucionar el error de conexión al usar claves SSH en entornos Windows.
- Proveer una validación temprana y un mensaje de error claro cuando el archivo de clave no existe.
- Garantizar que las rutas de los archivos sean consistentes y correctamente formateadas entre el frontend y el backend.

**Non-Goals:**
- Reemplazar la dependencia de `libssh2` por otra biblioteca de SSH.
- Añadir soporte para nuevos algoritmos de claves SSH no soportados actualmente.

## Decisions
El backend en Rust DEBE validar la existencia del archivo de la clave antes de intentar cualquier proceso de autenticación, utilizando `std::path::Path::new(kp).exists()`. Si no existe, el sistema DEBE arrojar un error descriptivo.

#### Escenario: Validación temprana de ruta inexistente
Dado que el usuario inicia una conexión con una ruta de clave inválida
Cuando el sistema verifica la existencia del archivo
Entonces el sistema DEBE rechazar la conexión de inmediato y mostrar un mensaje de error claro.

El sistema DEBE cargar la clave privada (y la pública si existe) directamente a la memoria mediante `std::fs::read_to_string` y autenticar usando `userauth_pubkey_memory` en vez de `userauth_pubkey_file`. Esta decisión previene que `libssh2` intente gestionar directamente los descriptores de archivo y la búsqueda automática de claves públicas, lo cual es propenso a fallar en Windows.

#### Escenario: Autenticación mediante claves en memoria
Dado que se proporciona una ruta válida para la clave
Cuando el sistema procede a autenticar
Entonces el sistema DEBE leer el archivo a la memoria y utilizar `userauth_pubkey_memory` exitosamente.

El código TypeScript en el frontend DEBE sanitizar todas las rutas de claves SSH ingresadas o almacenadas, reemplazando las barras invertidas por barras diagonales antes de guardar o enviar la información.

#### Escenario: Sanitización de rutas de Windows
Dado que el usuario ingresa una ruta nativa de Windows (ej. `C:\mis\claves\id_rsa`)
Cuando la UI procesa esta entrada
Entonces el sistema DEBE transformarla a un formato normalizado (ej. `C:/mis/claves/id_rsa`) para evitar problemas de escape de caracteres.

## Risks / Trade-offs
- **Riesgo:** Exposición temporal de material criptográfico sensible en la memoria del proceso de Rust.
  **Mitigación:** La memoria de estas cadenas es manejada de forma efímera durante el establecimiento de la sesión. Sin embargo, no se implementa un borrado seguro de la memoria (`zeroize`) en esta iteración, asumiendo el mismo nivel de riesgo que existía con el enfoque anterior.
- **Trade-off:** Reemplazar el manejo interno de archivos de la biblioteca en favor del manejo manual aumenta levemente el tamaño del código de integración, pero ofrece a cambio un control mucho más determinista sobre el ciclo de conexión y los errores resultantes.
