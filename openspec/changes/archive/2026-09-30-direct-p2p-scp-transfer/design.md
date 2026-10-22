## Context

NekoSSH cuenta con una función de portapapeles inter-sesión denominada "Copiar SCP" y "Pegar SCP". Sin embargo, su implementación actual en [`external_edit.rs`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/external_edit.rs#L1705-L1950) consiste en un relé síncrono de bloques de 64 KiB transferidos vía SFTP a través de la máquina local del usuario. 

Para archivos grandes (como dumps de bases de datos de 10 GB o directorios de producción masivos), este diseño sufre una penalización catastrófica de rendimiento por la suma de latencias ($RTT_A + RTT_B$), bloqueos de mutex en el runtime y la asimetría del ancho de banda de subida residencial, resultando en velocidades de ~500 KB/s (más de 5 horas para 10 GB).

El objetivo de este diseño es elevar NekoSSH a nivel comercial de alta gama mediante una arquitectura de transferencia directa punto a punto (P2P) entre servidores, aprovechando la infraestructura de red de los datacenters (100 MB/s a 1 GB/s) y automatizando la autenticación con las credenciales ya almacenadas.

## Goals / Non-Goals

**Goals:**
- **Transferencia P2P directa a velocidad de datacenter:** Orquestar la transferencia directamente del Servidor A al Servidor B utilizando los motores nativos del sistema operativo remoto (`rsync` preferente con fallback a `scp`).
- **Autenticación desatendida y segura:** Inyectar temporalmente en el servidor origen la credencial del servidor destino (clave privada aislada en memoria RAM `/dev/shm` o `tmpfs` con permisos `0600`), garantizando su destrucción atómica al terminar o abortar.
- **Telemetría real en UI:** Parsear en tiempo real el `stdout` del proceso remoto para reflejar velocidad exacta en MB/s, porcentaje y tiempo restante (ETA) en `#files-status`.
- **Cancelación cooperativa inmediata:** Soporte de aborto instantáneo vía guardias RAII, interrumpiendo el proceso en el servidor origen y limpiando archivos parciales y credenciales.
- **Fallback a Tubería Streaming (Pipe):** Si los servidores no tienen ruta IP directa entre sí (ej. redes privadas detrás de NAT sin salto), recurrir a un stream de sockets Unix directo (`cat`/`tar pipe`) sin el overhead de SFTP bloqueante.

**Non-Goals:**
- No se pretende instalar software permanentemente en los servidores remotos ni modificar su configuración global de SSH (`/etc/ssh/sshd_config`).
- No se persigue sincronización bidireccional continua (solo copia/transferencia puntual bajo demanda del usuario).

## Decisions

### Decisión 1: Orquestación del proceso nativo (`rsync` con fallback a `scp`)

- **Alternativa A (Descartada):** Continuar con SFTP local en chunks mayores (ej. 2 MB). Aunque mejora un poco, los 10 GB siguen pasando por el internet doméstico del usuario, quedando topados a su velocidad de subida (~3 MB/s).
- **Alternativa B (Elegida):** Ejecutar en el Servidor A un canal `exec` con:
  ```bash
  rsync -avP --inplace -e "ssh -p <PORT> -i <KEY> -o StrictHostKeyChecking=no" <SRC> user@<HOST>:<DEST>
  ```
  Si `rsync` no existe en el PATH del servidor origen (ej. imágenes Linux mínimas), fallback transparente a:
  ```bash
  scp -P <PORT> -i <KEY> -o StrictHostKeyChecking=no <SRC> user@<HOST>:<DEST>
  ```
- **Razón:** `rsync` proporciona `--inplace` (no duplica espacio en disco si se interrumpe), reanudación automática ante fallos de conexión y telemetría estructurada nativa de progreso.

### Decisión 2: Ciclo de vida y seguridad de la credencial efímera

- **Ubicación:** Se escribe la clave privada en `/dev/shm/.nekossh-p2p-<UUID>` (sistema de archivos en RAM de Linux, evitando escrituras en disco físico). Si `/dev/shm` no existe o no tiene permisos, fallback a directorio temporal `mktemp -d` con `chmod 700` y archivo `chmod 600`.
- **Ciclo de vida RAII en Rust (`P2PCredentialGuard`):**
  - Implementa el trait `Drop`. En cuanto la tarea de Tauri termina (por éxito, error o cancelación del usuario), ejecuta de forma síncrona en el canal de control:
    `shred -u <PATH> 2>/dev/null || rm -f <PATH>`
  - Garantiza que ninguna clave privada ajena quede persistida en el servidor origen.

### Decisión 3: Telemetría de progreso desacoplada por canal Tauri

- El backend en Rust consume de forma no bloqueante el canal `stdout` del comando remoto.
- Un parser en Rust extrae de las líneas de progreso de `rsync`/`scp`:
  - Porcentaje transferido.
  - Velocidad instantánea (formateada en MB/s o KB/s).
  - Tiempo estimado de finalización (ETA).
- Estos datos se envían directamente al canal IPC existente `TransferProgressPayload`, actualizando el indicador visual `#files-status` en el frontend.

### Decisión 4: Sondeo de conectividad previa y Fallback de Tubería

- Antes de lanzar el comando de transferencia, NekoSSH realiza una verificación de ruta TCP rápida (timeout 2 segundos) desde el Servidor A hacia el host y puerto del Servidor B:
  `timeout 2 bash -c "</dev/tcp/<HOST_B>/<PORT_B>" 2>/dev/null`
- Si devuelve código 0: Se ejecuta la **Transferencia Directa P2P**.
- Si falla (timeout, firewall o red no enrutada): Se activa inmediatamente el **Fallback de Tubería Streaming**, conectando el `stdout` de un `cat <SRC>` en el Servidor A con el `stdin` de un `cat > <DEST>` en el Servidor B a través de buffers asíncronos en Rust, informando al usuario en el status: `"Red directa no disponible: transfiriendo vía túnel de streaming"`.

## Risks / Trade-offs

- **[Riesgo: Servidores con shell no estándar o restringido (rbash, ash)]** 
  ➔ *Mitigación:* Los comandos de preparación de clave y chequeo de red utilizan sintaxis POSIX estándar compatible con `sh`, `ash`, `dash` y `bash`.
- **[Riesgo: Cancelación abrupta o desconexión del cliente NekoSSH]** 
  ➔ *Mitigación:* Se registra el PID del proceso remoto en el Servidor A al iniciar (`echo $$; exec rsync ...`). Si NekoSSH se cierra o se cancela la operación, se envía señal de terminación (`kill -TERM <PID>`) antes de limpiar las credenciales.
- **[Riesgo: Host key verification prompt en SSH remoto]** 
  ➔ *Mitigación:* Se añade `-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null` en la invocación de `ssh` dentro de `rsync`/`scp` para prevenir que el proceso se quede colgado esperando confirmación interactiva en `tty`.
