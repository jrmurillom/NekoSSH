# Propuesta: Subida Recursiva de Carpetas por Arrastrar y Soltar en Explorador SFTP

## Why

Actualmente, cuando el usuario arrastra carpetas o una selección de archivos que incluye subdirectorios (por ejemplo, el bundle de un sitio web o build estático como `techpeople` que contiene `assets/`, `img/` y `_next/`), el cliente NekoSSH intenta abrir cada ruta como un archivo plano mediante `std::fs::File::open()`. En sistemas operativos como Windows, esto produce inmediatamente un error de acceso (`Access is denied / os error 5`), abortando la transferencia de dichas carpetas y dejando sin subir todo su contenido interno (decenas o cientos de archivos faltantes). Habilitar la subida recursiva de directorios resuelve esta limitación crítica de usabilidad, permitiendo desplegar proyectos y carpetas completas directamente al servidor remoto sin pasos manuales ni necesidad de recurrir a archivos `.zip` o terminales auxiliares.

## What Changes

- **Inspección y Recorrido Recursivo Local:** El backend analiza los elementos arrastrados; si detecta un directorio, recorre recursivamente su árbol local identificando la estructura de carpetas y archivos con sus rutas relativas.
- **Creación Recursiva de Directorios Remotos en SFTP:** Antes de iniciar la transferencia de cada archivo, el sistema verifica y crea las carpetas remotas necesarias (`sftp.mkdir`) en el servidor destino.
- **Transferencia en Streaming $O(1)$ sin consumo excesivo de RAM:** Los archivos dentro de las carpetas se transfieren secuencialmente en bloques de 64 KiB reutilizando la infraestructura de streaming existente, garantizando que el consumo de memoria permanezca por debajo de 5 MB de RAM sin importar la cantidad de archivos.
- **Confirmación con Conteo Real de Elementos:** El modal de confirmación antes de la subida informa al usuario con precisión: *"Subir carpeta 'nombre' (X carpetas, Y archivos · Z MB) a /ruta/destino"*.
- **Progreso Global Unificado y Cancelación:** El indicador visual de estado en el explorador reporta el progreso ponderado sobre el total real de archivos y bytes, manteniendo la capacidad de cancelación cooperativa y limpieza de archivos temporales (`.nekossh.part`).

## Capabilities

### New Capabilities
<!-- Ninguna nueva capacidad de alto nivel; se extiende sftp-explorer -->

### Modified Capabilities
- `sftp-explorer`: Extiende el requerimiento de arrastrar y soltar para soportar la detección, creación de directorios remotos y transferencia recursiva de carpetas completas manteniendo streaming $O(1)$.

## Impact

- **Backend Rust:** [`app/src-tauri/src/external_edit.rs`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/external_edit.rs) (nuevas funciones de paseo de directorios locales y `sftp_upload_dir` / `sftp_ensure_remote_dir`).
- **Frontend TypeScript:** [`app/src/modules/explorer-drop-helper.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/modules/explorer-drop-helper.ts) y [`app/src/main.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts) (`runExplorerUpload`).
- **IPC Commands:** Comandos Tauri para inspeccionar ítems arrastrados (o subida recursiva directa) con reporte de progreso unificado.
- **Tests:** Pruebas unitarias en backend Rust y frontend TypeScript (Vitest).
