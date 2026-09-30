# Diseño Técnico: Subida Recursiva de Carpetas por Arrastrar y Soltar en SFTP

## Context

En NekoSSH, la interacción de arrastrar y soltar hacia el panel de archivos remotos (`#files-panel`) entrega una lista de rutas absolutas del sistema de archivos local (`paths: string[]`). Actualmente, `runExplorerUpload` en [`app/src/main.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts#L1442) itera ciegamente sobre cada ruta y llama al comando `sftp_upload_file`. En el backend de Rust, [`app/src-tauri/src/external_edit.rs`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/external_edit.rs#L339) abre el archivo con `std::fs::File::open(local_path)`. Si una ruta corresponde a un directorio (como `assets/`, `img/` o `_next/`), la llamada falla en Windows con `Access is denied (os error 5)` y la carpeta es omitida junto con todo su contenido interno.

## Goals / Non-Goals

**Goals:**
- Permitir al usuario arrastrar carpetas enteras o selecciones mixtas de archivos y carpetas directamente al explorador SFTP.
- Escanear localmente el árbol arrastrado para obtener un resumen preciso (cantidad de directorios, archivos y bytes totales) para el diálogo de confirmación.
- Garantizar la creación idempotente y recursiva de directorios remotos en el servidor SFTP (`sftp.mkdir`).
- Mantener la transferencia secuencial en streaming de 64 KiB ($O(1)$ en memoria RAM), asegurando que el consumo añadido permanezca por debajo de 5 MB sin importar el volumen de archivos.
- Preservar la emisión de progreso en tiempo real (porcentaje, conteo `i/N`, velocidad) y la cancelación cooperativa inmediata con eliminación de archivos temporales `.nekossh.part`.

**Non-Goals:**
- Sincronización bidireccional continua (file watcher / daemon en vivo).
- Empaquetado tar/zip remoto ejecutando comandos en el shell PTY (la operación se mantiene 100% sobre el canal SFTP existente para no interferir ni enviar texto invasivo al shell del usuario).

## Decisions

### 1. Escaneo previo en backend: `sftp_scan_local_items`
- **Racional:** Recorrer el sistema de archivos local desde Rust es órdenes de magnitud más rápido que hacerlo en JavaScript y no bloquea el hilo de la UI. El comando devuelve:
  ```rust
  pub struct LocalUploadPlan {
      pub total_files: usize,
      pub total_dirs: usize,
      pub total_bytes: u64,
      pub items: Vec<UploadItem>,
  }
  ```
  Donde cada `UploadItem` contiene la ruta local absoluta y la ruta relativa respecto al destino remoto.
- **Alternativas consideradas:**
  - *Subida ciega en Rust de una sola llamada:* Impide que el diálogo de confirmación de la UI muestre con exactitud cuántos archivos y carpetas se van a transferir, reduciendo la transparencia al usuario.

### 2. Creación remota de directorios idempotente: `sftp_ensure_dir_recursive`
- **Racional:** SFTP no tiene un equivalente nativo a `mkdir -p`. Para crear rutas como `/var/www/html/techpeople/_next/static/chunks`, se debe verificar o crear secuencialmente cada segmento intermedio:
  ```rust
  fn ensure_remote_dir(sftp: &ssh2::Sftp, remote_dir: &str) -> Result<(), String>
  ```
  Si `sftp.stat()` indica que la carpeta ya existe, no se hace nada. Si no existe, se ejecuta `sftp.mkdir(path, 0o755)`.
- **Alternativas consideradas:**
  - *Lanzar `mkdir -p` por el PTY:* Invade la terminal activa del usuario, altera el historial de comandos y falla si el usuario está en una sesión interactiva (ej. `top` o `nano`).

### 3. Pipeline de Transferencia en Streaming $O(1)$
- **Racional:** Los archivos descubiertos en el árbol se transfieren uno a uno utilizando la lógica ya validada de `sftp_upload_file_blocking_with_progress`:
  - Bloques de 64 KiB.
  - Archivo temporal staging `.<nombre>.nekossh.part`.
  - Bombeo continuo del PTY (`pump_pty`) para mantener activo el transporte SSH y evitar falsos `transport read`.
  - Renombrado atómico final.

## Risks / Trade-offs

- **[Riesgo] Enlaces simbólicos rotos o circulares en carpetas locales:**  
  *Mitigación:* Durante el escaneo local en Rust, se ignoran enlaces simbólicos que apunten a ancestros (prevención de bucles infinitos) y se aplica un límite de profundidad de seguridad (`MAX_DEPTH = 32`).
- **[Riesgo] Carpetas con decenas de miles de archivos pequeños (ej. `node_modules`):**  
  *Mitigación:* El diálogo de confirmación previo advierte la cantidad exacta de archivos antes de iniciar. Si el usuario confirma y la red es lenta, puede pulsar el botón Cancelar (`[ ✕ ]`) en cualquier momento para detener la transferencia de forma limpia.

---

### Corrección de Ruta (Fix): Matriz Exhaustiva de Robustez Comercial y Simulación E2E

Para alcanzar un estándar comercial del 100% libre de puntos ciegos, se expande la estrategia de validación técnica en tres frentes:

1. **Matriz de Casos Borde y Negativos en Backend (`upload_scan.rs`):**
   - Asegurar que árboles con carpetas vacías no fallen ni desincronicen el conteo de directorios vs archivos.
   - Manejo de archivos de 0 bytes.
   - Preservación íntegra de caracteres UTF-8, espacios y símbolos en rutas Windows/POSIX.
   - Aserción de errores controlados para rutas inexistentes y protección contra rebasamiento del límite de profundidad (32 niveles).

2. **Suite de Integración E2E en Frontend (`explorer-folder-upload.e2e.test.ts`):**
   - Simular el ciclo de vida completo de la interacción de usuario con mocks de DOM e IPC de Tauri.
   - Aserción de telemetría de lote `[i/N]`, cálculo de impacto en diálogo modal y comportamiento de cancelación y omisión en colisiones.

3. **Certificación de Regresión Real (Caso `techpeople`):**
   - Validar programáticamente una réplica exacta de 125+ archivos con la jerarquía real del bundle web `techpeople` que provocó el reporte inicial.

