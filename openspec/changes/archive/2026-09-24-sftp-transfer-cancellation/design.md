## Context

En la arquitectura actual de NekoSSH:
- Las tres operaciones de transferencia (`sftp_download_to_user_path_blocking`, `sftp_upload_file_blocking_with_progress` y `sftp_copy_between_sessions`) corren en hilos `spawn_blocking` iterando en bloques de `64 KiB` (`SFTP_STREAM_CHUNK_BYTES`).
- Cada operación clona el `Arc<Mutex<LiveSsh>>` al iniciar (`with_live_ssh`). Por ello, si el usuario cierra la pestaña de la terminal (`close_ssh_session`), aunque `terminal_id` se elimina del mapa `SshConnections`, el `Arc<Mutex<LiveSsh>>` sigue vivo en el hilo `spawn_blocking` y la transferencia no se entera de que la sesión fue cerrada.
- Asimismo, si el usuario cierra la aplicación (`RunEvent::Exit`), el proceso termina sin que el hilo `spawn_blocking` alcance su bloque de limpieza, dejando archivos `.<nombre>.nekossh.part` huérfanos en el disco local.
- En el frontend, `#files-status.is-progress` muestra el progreso en tiempo real pero no ofrece un botón `[ ✕ ]` con confirmación (`confirmDialog`) para cancelar manualmente.

## Goals / Non-Goals

**Goals:**
- Implementar `ActiveTransferRegistry` en Rust (`edit_util.rs`) con un guardián RAII (`ActiveTransferGuard`) que registre la transferencia activa (`cancel_flag: Arc<AtomicBool>`, `terminal_ids: Vec<String>`, `local_part_path: Option<PathBuf>`) y la desregistre automáticamente al salir del alcance (`Drop`).
- Conectar la verificación `cancel_flag.load(Ordering::Acquire)` en el bucle de `64 KiB` de **Descarga**, **Subida** y **Copia SCP**:
  - Si `cancel_flag` es `true`, cerrar descriptores, borrar `.nekossh.part` (`std::fs::remove_file` local o `sftp_unlink_best_effort` remoto) y devolver `Err(TRANSFER_CANCELLED_ERROR.to_string())`.
- Conectar los **3 disparadores deterministas**:
  1. **UI Manual (`[ ✕ ]` + `confirmDialog`):** Botón `.files-status-progress-cancel` en el encabezado de `#files-status.is-progress` reutilizando referencias DOM (0 reflows en ticks de progreso). Al pulsarlo, abre `confirmDialog` y, si se confirma, invoca `sftp_cancel_transfer`.
  2. **Cierre de Sesión (`close_ssh_session` / `close_all_ssh_connections`):** Invoca `active_transfers.cancel_for_terminal(&terminal_id)` y en el frontend invalida los mensajes en vuelo del `Channel` si la transferencia fue cancelada o la terminal cerrada.
  3. **Cierre de App (`RunEvent::ExitRequested | RunEvent::Exit`):** Invoca `active_transfers.cancel_all_and_cleanup_local_parts()`, señalizando el `AtomicBool` y eliminando síncronamente cualquier `local_part_path` registrado.

**Non-Goals:**
- No pausar/congela el socket TCP mientras el modal `confirmDialog` está abierto esperando respuesta del usuario (la transferencia sigue avanzando y solo se corta si el usuario confirma `"Sí, cancelar"`).

## Decisions

### 1. `ActiveTransferRegistry` con Guardián RAII (`ActiveTransferGuard`) y `AtomicBool`
- **Decisión:** Crear `ActiveTransferRegistry` en `edit_util.rs` protegido por `Arc<Mutex<HashMap<u64, ActiveTransferEntry>>>`. Al iniciar una transferencia se obtiene un `(u64, Arc<AtomicBool>, ActiveTransferGuard)`:
  - En cada bloque de `64 KiB` solo se lee `cancel_flag.load(Ordering::Acquire)` (operación atómica sin lock de `Mutex` en el camino crítico de red/disco, costo $<1\text{ ns}$).
  - Cuando la función de transferencia termina (por éxito, error o cancelación), el `Drop` de `ActiveTransferGuard` remueve automáticamente la entrada del registro.
- **Racional:** Evita contención de `Mutex` en cada bloque de `64 KiB` y garantiza mediante RAII que jamás quede una entrada colgada en el registro aunque ocurra un retorno temprano.

### 2. Limpieza Síncrona de `.nekossh.part` en `RunEvent::Exit`
- **Decisión:** Cuando el hilo de descarga cierra `part_file` ante `cancel_flag == true`, borra `part_path`; además, en `RunEvent::ExitRequested | RunEvent::Exit`, `cancel_all_and_cleanup_local_parts()` activa todos los `cancel_flag`, espera hasta `100ms` en pasos de `5ms` para que cualquier hilo `spawn_blocking` suelte el descriptor `File` en Windows y ejecuta `std::fs::remove_file(&part_path)`.
- **Racional:** En Windows un archivo abierto sin `FILE_SHARE_DELETE` o con handle activo puede requerir que el hilo `spawn_blocking` haga `drop(part_file)` antes de `remove_file`. Al activar `cancel_flag` y dar unos milisegundos para que el bucle de `64 KiB` haga `drop(part_file)` seguido de `remove_file` en el hook de salida, garantizamos **100% de eliminación en Windows** al cerrar la app.

### 3. Botón `[ ✕ ]` Reutilizable en `createExplorerStatusController` y Manejo de Lotes
- **Decisión:**
  - `createExplorerStatusController(filesStatus, { onCancelClick })` crea el botón `<button type="button" class="files-status-progress-cancel" title="Cancelar transferencia" aria-label="Cancelar transferencia">✕</button>` una sola vez dentro de `.files-status-progress-header` junto a `percentEl`, preservando el nodo DOM en todos los ticks de progreso.
  - En `runExplorerUpload`, `handleDownloadFile` y `handlePasteScp`, si el error devuelto contiene `TRANSFER_CANCELLED` (o `userCancelledCurrentTransfer === true`), `runExplorerUpload` rompe inmediatamente el bucle `for (const p of toUpload)` (`break`) para no iniciar los archivos restantes del lote, y muestra `"Transferencia cancelada"` (estado informativo que se auto-oculta a los `3000ms`, no un error rojo persistente).

## Risks / Trade-offs

- **[Risk] Carrera cuando la transferencia termina al 100% justo mientras el usuario tiene abierto el `confirmDialog`** $\rightarrow$ **Mitigation:** Si `transferInProgress` ya es `false` cuando `confirmDialog` resuelve `true`, no se invoca `sftp_cancel_transfer` ni se altera el banner de `"✅ Descarga/Subida completa"`. Además, `sftp_cancel_transfer` es idempotente si no hay transferencias registradas.
- **[Risk] Mensajes residuales en la cola IPC de `Channel<TransferProgressPayload>` después de cancelar o cerrar pestaña** $\rightarrow$ **Mitigation:** En el frontend se mantiene un flag booleano de sesión de transferencia (`isCurrentTransferAborted`); una vez activada la cancelación o cerrada la terminal, `onProgress.onmessage` ignora inmediatamente cualquier paquete rezagado para que jamás re-pinte `.is-progress` sobre el estado limpio o cancelado.
