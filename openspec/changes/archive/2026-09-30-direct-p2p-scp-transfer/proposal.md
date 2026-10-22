## Why

El mecanismo actual de "Copiar / Pegar SCP" entre sesiones activas no ejecuta una transferencia punto a punto (P2P) real; en su lugar, implementa un puente relé por SFTP a través de la máquina local en bloques secuenciales síncronos de 64 KiB. Para transferir archivos de gran volumen (como volcados de base de datos o bundles de 10 GB), este diseño degrada la velocidad a ~500 KB/s debido a la suma de latencias de ida y vuelta ($RTT_A + RTT_B$), requiriendo horas de espera e inutilizando la función.

Es imperativo implementar una arquitectura de transferencia directa P2P de grado industrial que ejecute `rsync` o `scp` nativamente en el Servidor A apuntando al Servidor B a velocidad de datacenter (100 MB/s - 1 GB/s), inyectando de forma automatizada y segura las credenciales que NekoSSH ya almacena, con telemetría en tiempo real y fallback a tubería streaming Unix (`pipe`) si los servidores se encuentran en redes físicamente incomunicadas.

## What Changes

- **Transferencia nativa directa P2P:** Orquestación de comandos `rsync` (con reanudación `--inplace` y compresión) o `scp` nativo ejecutados directamente en el servidor origen hacia el destino.
- **Inyección segura de credenciales en memoria/sesión:** NekoSSH suministra la identidad del servidor destino (clave privada en memoria `tmpfs`/canal seguro `chmod 600` o variable de sesión) para autenticar la conexión sin solicitar contraseñas manuales ni persistir claves ajenas en el disco del servidor origen.
- **Limpieza atómica y RAII:** Eliminación garantizada del artefacto de credencial temporal inmediatamente al terminar, cancelar o abortar la operación.
- **Telemetría y progreso real:** Captura y parseo continuo del flujo de salida (`stdout` de rsync/scp) para emitir a la UI el porcentaje exacto, velocidad de transferencia en MB/s y tiempo restante estimado (ETA).
- **Fallback a Tubería Streaming (Pipe):** Si el servidor origen no tiene ruta de red hacia el servidor destino (firewalls o redes privadas no enrutadas), el sistema recurre a un streaming directo de sockets Unix (`cat / tar pipe` sobre SSH) sin overhead de SFTP bloqueante ni contención de mutex.
- **Claridad de interfaz:** Reemplazo de las etiquetas ambiguas en el explorador por una experiencia transparente de "Transferencia inter-servidor" con indicación visual del modo de transporte (Directo P2P o Tubería de Alta Velocidad).

## Capabilities

### New Capabilities

<!-- Ninguna nueva capacidad de alto nivel requerida -->

### Modified Capabilities

- `sftp-explorer`: Sustituye la transferencia inter-sesión basada en relé SFTP por transferencia directa P2P con inyección de identidad remota, parseo de telemetría de stdout y fallback de tubería streaming.

## Impact

- **Backend (Rust):** Módulo `app/src-tauri/src/external_edit.rs` o nuevo módulo especializado `p2p_transfer.rs`, integración con canales `exec` de `ssh2`, gestión de archivos temporales seguros remotos y emisión de eventos de progreso enriquecidos.
- **Frontend (TypeScript):** `app/src/main.ts` y helpers de transferencia para consumir la telemetría detallada de velocidad (MB/s) y ETA.
- **Seguridad:** Eliminación garantizada de claves temporales remotas con permisos restrictivos y aislamiento en RAM.
- **Rendimiento:** Reducción drástica del tiempo de transferencia para archivos de 10 GB de más de 5 horas a menos de 2 minutos en redes de datacenter con conectividad directa.
