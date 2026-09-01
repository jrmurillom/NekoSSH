## Context

En NekoSSH, las transferencias SFTP y el emulador interactivo de terminal (PTY) conviven sobre la misma sesión SSH (Session). Por este motivo, la sesión opera en modo no bloqueante (session.set_blocking(false)). Al realizar una subida de archivos mediante sftp_upload_file_blocking, la función transmite los fragmentos en un bucle, pero una vez transferidos los bytes, retorna de inmediato y destruye el descriptor ssh2::File. En modo no bloqueante, el cierre del descriptor y el vaciado final de la cola del socket pueden retornar EAGAIN (WouldBlock), lo cual es descartado por el Drop de Rust. Esto provoca que archivos binarios como .zip queden incompletos o sin su catálogo central, generando corrupción.

## Goals / Non-Goals

**Goals:**
- Implementar un bucle de vaciado explícito (lush) con reintentos no bloqueantes y bombeo del PTY (pump_pty) en sftp_upload_file_blocking.
- Añadir verificación de tamaño remoto mediante sftp.stat() tras completar la subida para confirmar la integridad byte a byte.
- Mantener la terminal interactiva receptiva y sin congelamientos durante transferencias binarias.
- Proporcionar pruebas unitarias y de simulación (mock/fake) para verificar la integridad de subidas binarias.

**Non-Goals:**
- No implementar transferencia recursiva de directorios completos (se mantiene el alcance de archivos planos individuales).
- No modificar el protocolo de comunicación ni la firma de los comandos Tauri con el frontend.
- No utilizar comandos de Git autónomos ni descargas de dependencias no autorizadas.

## Decisions

### Decisión 1: Bucle explícito de emote.flush() con bombeo de PTY
- **Razón**: ssh2::File requiere vaciar cualquier paquete pendiente en el canal SFTP antes de ser destruido. Un bucle con reintentos limitados (ej. hasta 200 intentos con sleep de 10ms) y ejecución de pump_pty garantiza que los buffers de red se drenen sin bloquear permanentemente el hilo ni degradar el PTY.
- **Alternativas consideradas**:
  - *Cambiar temporalmente a set_blocking(true)*: Descartado porque corrompe el flujo no bloqueante del PTY activo y puede congelar la UI de la terminal.

### Decisión 2: Validación obligatoria con sftp.stat() post-escritura
- **Razón**: Tras la escritura y el vaciado, una consulta stat remota fuerza a libssh2 a sincronizar el estado del archivo con el servidor SSH y permite validar que stat.size == data.len() as u64. Si no coinciden, se retorna un error descriptivo impidiendo falsos positivos de éxito.
- **Alternativas consideradas**:
  - *Calcular hash MD5/SHA256 remoto*: Descartado por requerir ejecución de comandos remotos en el shell (md5sum/sha256sum) que pueden no estar disponibles en todos los entornos o diferir según el SO remoto; sftp.stat() es nativo del protocolo SFTP.

### Decisión 3: Verificación mediante tests con FakeSftpStore y harnesses locales
- **Razón**: Cumpliendo con el estándar del proyecto, no se mutará el host SSH compartido en pruebas de CI o agente; se utilizarán fakes y pruebas unitarias in-process.

## Risks / Trade-offs

- **[Riesgo] Mayor tiempo de espera al final de la subida**: La sincronización y verificación añade entre 5 y 30 ms por archivo.
  - *Mitigación*: Es un retraso imperceptible para el usuario humano pero crítico para la integridad de datos.
- **[Riesgo] Timeout en redes lentas o con alta latencia**: En conexiones muy lentas, el lush podría agotar los intentos.
  - *Mitigación*: Configurar un número de reintentos razonable (ej. 200 iteraciones con backoff) y reportar un error claro y legible si la red no responde.
