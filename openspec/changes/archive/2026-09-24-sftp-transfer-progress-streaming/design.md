## Context

En el estado actual de NekoSSH, el explorador de archivos SFTP soporta dos flujos de transferencia hacia un servidor remoto:
1. **Subida desde host local por Drag & Drop (`sftp_upload_file`):** Implementada en `app/src-tauri/src/external_edit.rs` (`sftp_upload_file_blocking`) y orquestada en `app/src/main.ts` (`runExplorerUpload`). Actualmente lee el archivo completo en memoria RAM mediante `std::fs::read(local_path)`, escribe directamente sobre la ruta final truncándola de inmediato, carece de límite de reintentos en `WouldBlock` durante la escritura de bloques y no emite eventos de progreso al frontend.
2. **Copia inter-sesión SCP (`sftp_copy_between_sessions`):** Implementada en `external_edit.rs` y orquestada en `main.ts` (`handlePasteScp`). Aunque utiliza un búfer fijo de 64 KiB para transferir entre dos sesiones SFTP activas, no consulta el tamaño total de origen (`sftp.stat()`), no utiliza escritura atómica temporal, carece de timeout en `WouldBlock` de escritura, no comparte el semáforo de concurrencia con Drag & Drop y no reporta progreso al frontend.

En la interfaz de usuario, ambos flujos muestran únicamente un texto estático en el overlay `#files-status` (`Subiendo X…` o `Copiando X…`), sin barra de progreso, porcentaje, bytes transferidos ni velocidad.

## Goals / Non-Goals

**Goals:**
- **Consumo de memoria constante $O(1)$:** Realizar la lectura local en `sftp_upload_file_blocking` mediante `std::fs::File` y un búfer fijo de 64 KiB (`[0u8; 65536]`), eliminando la asignación del archivo completo en RAM.
- **Tolerancia cero a fallos y atomicidad:**
  - Escribir en un archivo temporal oculto en el mismo directorio destino (`.<nombre>.nekossh.part`), ejecutar `flush()`, cerrar el descriptor, validar con `sftp.stat()` que el tamaño remoto coincida exactamente con los bytes esperados, y renombrar al destino final (`sftp.rename` con soporte de sobreescritura).
  - En caso de fallo de lectura, escritura, timeout o discrepancia de tamaño, eliminar automáticamente el archivo `.nekossh.part` residual (`sftp.unlink`) sin corromper un archivo preexistente en el destino.
  - Establecer un límite máximo de reintentos consecutivos (`MAX_WOULD_BLOCK_ATTEMPTS = 400`, ~2 segundos de bloqueo continuo sin transferir un solo byte) en los bucles `write` de ambos flujos para impedir bloqueos infinitos de hilos ante caídas de red.
- **Telemetría en tiempo real vía `tauri::ipc::Channel`:**
  - Transmitir `TransferProgressPayload` desde Rust hacia el frontend con *throttling* de `100ms` (garantizando emisión en `0%` inicial y `100%` final), incluyendo `operation` (`"upload"` | `"scp"`), `file_name`, `bytes_transferred`, `total_bytes`, `percent` (`0..=100`) y `speed_bps`.
- **Loader y barra de progreso visual Cyber-Sakura:**
  - Renderizar dentro del overlay flotante `#files-status` un componente estructurado con encabezado (operación, archivo, índice de lote y porcentaje), barra de progreso visual acoplada a los tokens del tema activo (`var(--color-accent-primary)`, sin *neon glow*) y pie de métricas (`MB / MB • MB/s`), sin desplazar el árbol `#files-tree`.
- **Exclusión mutua unificada en UI:**
  - Unificar el flag `uploadInProgress` en un bloqueo de transferencia compartido (`transferInProgress`) que proteja tanto `runExplorerUpload` como `handlePasteScp`.

**Non-Goals:**
- Reemplazar el motor `ssh2` (`libssh2`) por `russh` o introducir dependencias externas en `Cargo.toml` / `package.json`.
- Abrir conexiones TCP/SSH secundarias que requieran re-autenticación del usuario.
- Subida recursiva de carpetas completas (fuera del alcance actual de archivos individuales/lotes de archivos).

## Decisions

### 1. Canal IPC: `tauri::ipc::Channel<TransferProgressPayload>` vs `app.emit()`
- **Decisión:** Utilizar `tauri::ipc::Channel<TransferProgressPayload>` (como `Option<Channel<TransferProgressPayload>>` para preservar retrocompatibilidad).
- **Racional:** En Tauri v2, `Channel` es el mecanismo nativo punto-a-punto optimizado para flujos de datos de alta frecuencia asociados a un comando `invoke`. Evita contaminar el bus de eventos global de la ventana, no requiere gestión manual de `listen`/`unlisten` y libera recursos automáticamente al concluir la promesa del `invoke`.
- **Alternativa descartada:** `app.emit("sftp-upload-progress", ...)` — genera *broadcast* innecesario y riesgo de *listeners* huérfanos si ocurre una excepción en el frontend.

### 2. Lectura en Streaming con Búfer Fijo de 64 KiB (`[0u8; 65536]`)
- **Decisión:** Abrir el archivo local con `std::fs::File::open(local_path)`, obtener `total_bytes = file.metadata()?.len()`, y leer iterativamente en un arreglo fijo de 64 KiB.
- **Racional:** Garantiza complejidad espacial $O(1)$ en RAM (64 KiB constantes tanto para un archivo de 1 KB como para una imagen ISO de 10 GB) y equilibra el tamaño de ventana de paquetes SFTP con el ciclo de `pump_pty()` del canal interactivo.
- **Alternativa descartada:** `std::fs::read` — provoca picos de memoria proporcionales al tamaño del archivo y riesgo de cierre abrupto por OOM.

### 3. Staging Atómico (`.<nombre>.nekossh.part`) y Limpieza en Fallo
- **Decisión:** Construir una ruta temporal hermana en el mismo directorio remoto (ej. `/var/www/.backup.zip.nekossh.part`). Escribir todos los bloques y hacer `flush()` sobre el archivo `.part`, cerrar el handle, verificar el tamaño mediante `sftp.stat(&part_path)` y aplicar el reemplazo sobre `remote_path` (intentando `sftp.rename` con `RenameFlags::OVERWRITE | RenameFlags::ATOMIC | RenameFlags::NATIVE` y, si el servidor SFTP v3 rechaza el flag de sobreescritura cuando el destino ya existe, ejecutando `sftp.unlink(remote_path)` seguido de `sftp.rename(&part_path, remote_path, None)`). Si cualquier paso previo al renombrado falla, se invoca `sftp.unlink(&part_path)` en modo *best-effort* antes de retornar el error.
- **Racional:** Evita que una caída de red al 50% deje un archivo truncado o corrupto con el nombre definitivo en el servidor destino.

### 4. Throttling Temporal de 100ms y Aritmética Segura en `u64`
- **Decisión:** Evaluar `Instant::now().duration_since(last_emit) >= Duration::from_millis(100)` antes de enviar actualizaciones intermedias por el canal, emitiendo siempre el estado inicial (`0%`) y el estado final (`100%`). Si `total_bytes == 0`, el porcentaje se resuelve directamente a `100%` sin división por cero.
- **Racional:** Limita la tasa de refresco en la Webview a un máximo de ~10 FPS durante transferencias rápidas en red local, manteniendo fluidez visual sin saturar el hilo principal de UI.

```
┌─────────────────────────────────────────────────────────────────────────┐
│ FLUJO DE TRANSFERENCIA BLINDADO (Upload Local & Copy SCP)               │
├─────────────────────────────────────────────────────────────────────────┤
│ 1. Obtener total_bytes (File::metadata() o sftp_src.stat())             │
│ 2. Emitir progreso 0% por Channel                                       │
│ 3. Crear remoto temporal: .<filename>.nekossh.part                      │
│ 4. Loop (buffer 64 KiB):                                                │
│      ├─ Leer chunk (con pump_pty + control WouldBlock)                  │
│      ├─ Escribir chunk en .part (con pump_pty + timeout WouldBlock)     │
│      └─ Si elapsed >= 100ms -> Emitir progreso (% / MB / MB/s)          │
│ 5. Flush + Drop handle + Verificar sftp_tgt.stat(.part) == total_bytes  │
│ 6. Rename atómico (.part -> destino final) y emitir 100%                │
│    (Si ocurre error en pasos 4-5: sftp_tgt.unlink(.part) y propagar Err)│
└─────────────────────────────────────────────────────────────────────────┘
```

## Risks / Trade-offs

- **[Risk] Servidores SFTP v3 antiguos que no soportan `SSH_FXP_EXTENDED` (`posix-rename@openssh.com`) para sobreescribir en `rename`** → **Mitigation:** Implementar fallback automático: si `sftp.rename` con flags de sobreescritura falla porque el archivo destino ya existe, eliminar el destino previo (`sftp.unlink(remote_path)`) inmediatamente después de haber validado al 100% el archivo `.nekossh.part` y reintentar `sftp.rename(&part_path, remote_path, None)`.
- **[Risk] Permisos de directorio restringidos donde el usuario puede modificar un archivo existente pero no crear archivos nuevos en el directorio padre (ej. `/etc/app.conf` en edición externa)** → **Mitigation:** Si `sftp.create(&part_path)` falla por permisos o si la operación proviene de reemplazo directo donde no es posible crear `.part`, conservar la clasificación de error `Permission denied` intacta para que el flujo de elevación `sudo` existente (`classify_upload_error`) funcione sin regresiones.
- **[Risk] Saturación del DOM en actualizaciones de progreso** → **Mitigation:** Además del throttling de `100ms` en Rust, reutilizar los nodos DOM internos de `#files-status` durante la transferencia activa (actualizando únicamente `textContent` y `style.width` de la barra) en lugar de recrear el árbol DOM en cada tick.

### Corrección de Ruta (Fix)

Tras la auditoría de experiencia de usuario (UX) y cobertura de pruebas E2E, se incorporan tres ajustes arquitectónicos:

1. **Estado Pre-Flight de UX (`"Preparando transferencia…"`):**
   - Mientras el backend negocia la apertura del canal SFTP o consulta `stat` antes del primer bloque, el payload inicial `{ bytes_transferred: 0, total_bytes: 0, percent: 0 }` debe renderizar `"Preparando transferencia…"` en la línea secundaria de métricas en lugar de `"0 B / 0 B"`, evitando la percepción visual errónea de un archivo vacío.
2. **Cierre de Ventana de Concurrencia en Verificación de Colisiones (`sftp_list_dir`):**
   - En `runExplorerUpload`, inmediatamente después de que el usuario confirma el diálogo de subida, se activa `transferInProgress = true` (con bloque `try / finally` envolviendo también `sftp_list_dir` y los diálogos de reemplazo) y se muestra `"Verificando destino…"` en `#files-status` para evitar una ventana silenciosa sin bloqueo ni feedback.
3. **Controlador DOM Modular y Suite de Integración E2E de Ciclo de Vida (`ExplorerStatusController`):**
   - Encapsular la máquina de estados del overlay `#files-status` (`setStatus`, `setTransferProgress`, timer de `3000ms`, cache de nodos DOM `statusProgressRefs`) y validar con una suite de pruebas E2E de ciclo de vida DOM/UX (`transfer-progress-e2e.test.ts`):
     - Preservación de identidad de nodos DOM (`barEl`, `titleEl`, `percentEl`, `metaEl`) a través de múltiples eventos consecutivos de `Channel`.
     - Transición completa `Preparando transferencia…` $\rightarrow$ `Progreso (0% -> 50% -> 100%)` $\rightarrow$ `Éxito (✅)` $\rightarrow$ `Auto-dismiss a los 3000ms`.
     - Transición a `Error (❌)` al fallar a mitad de transferencia y verificación de que el mensaje de error **persiste** tras superar los `3000ms`.
     - Verificación de bloqueo de concurrencia E2E (`transferInProgress`) impidiendo que `handlePasteScp` y `runExplorerUpload` se ejecuten en paralelo y asegurando liberación del candado en `finally`.

