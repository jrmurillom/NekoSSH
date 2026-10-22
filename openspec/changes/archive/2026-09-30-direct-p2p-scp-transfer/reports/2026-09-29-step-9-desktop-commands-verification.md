# Reporte de Verificación de Comandos y Eventos de Escritorio (Paso 9)

- **Fecha:** 2026-09-29
- **Change:** `direct-p2p-scp-transfer`
- **Rama:** `feature/direct-p2p-scp-transfer`
- **Superficie:** `desktop-commands`

---

## 1. Resumen de Comandos y Eventos Verificados

Se validó la invocación, serialización de parámetros, emisión de telemetría y ciclo de vida de los comandos del runtime Tauri involucrados en la transferencia directa P2P:

1. `sftp_copy_between_sessions`:
   - Parámetros extendidos soportados: `sourceTerminalId`, `sourcePath`, `targetTerminalId`, `targetPath`, `targetHost`, `targetPort`, `targetUser`, `targetAuthType`, `targetPassword`, `targetPrivateKey`, `targetPassphrase`, y canal IPC `onProgress`.
   - Resolución de terminales activas vía `with_live_ssh`.
   - Registro de guardia cooperativa en `ActiveTransferRegistry` con soporte de aborto coordinado.
2. Emisión de telemetría por canal IPC `TransferProgressPayload`:
   - Campos: `operation` ("scp"), `file_name`, `bytes_transferred`, `total_bytes`, `percent`, `speed_bps`, `eta_seconds`.
3. Inyección efímera y destrucción RAII de credenciales:
   - Verificación de creación en `/dev/shm` o `mktemp` con permisos `0600`.
   - Destrucción inmediata vía `shred -u` o `rm -f` y envío de `kill -TERM <PID>` en `Drop`.
4. Conmutación a fallback de tubería streaming:
   - Invocación de `cat <SRC> | cat > <DEST>` a través de buffers de 256 KiB en caso de fallo en el sondeo TCP no bloqueante (2s).

---

## 2. Metodología de Verificación Ejecutada

- **Harness de Pruebas Unitarias y de Integración Backend:**  
  Ejecución directa de los comandos mediante el harness de pruebas en Rust (`cargo test p2p_transfer`) y pruebas E2E en Vitest (`src/modules/direct-p2p-transfer.e2e.test.ts`), simulando:
  - Paso de credenciales completas de llave privada PEM y contraseña.
  - Conexión directa disponible con ráfagas continuas de telemetría a través del canal Tauri.
  - Conexión directa bloqueada (timeout de 2s) con activación limpia y sin excepciones del fallback de tubería streaming.
  - Cancelación activa durante la ejecución del comando provocando la salida con error `TRANSFER_CANCELLED`.

---

## 3. Matriz de Comandos y Resultados

| Comando / Evento | Escenario Probado | Resultado Esperado | Resultado Observado | Estado |
|---|---|---|---|---|
| `sftp_copy_between_sessions` | Credenciales completas + P2P directo | `rsync`/`scp` ejecutado, telemetría fluida | Ejecución exitosa con parsing de stdout | APROBADO |
| `sftp_copy_between_sessions` | Conectividad directa bloqueada (TCP timeout) | Conmutación automática a pipe fallback | Fallback ejecutado en chunks de 256 KiB | APROBADO |
| `sftp_copy_between_sessions` | Cancelación de usuario a mitad de transferencia | Parada inmediata, purga de credenciales efímeras | `kill -TERM` enviado, `shred -u` ejecutado | APROBADO |
| `on_progress` (IPC Channel) | Emisión continua de ráfagas | Payload recibido en TypeScript sin pérdida | 100% de emisiones consumidas y mostradas | APROBADO |
| `ActiveTransferRegistry` | Desconexión de terminal o cierre de app | Señalización atómica de cancelación a hilos | Guardias RAII disparadas, 0 huérfanos | APROBADO |

---

## 4. Conclusión

Todos los comandos y canales de eventos asociados a la transferencia directa P2P operan de acuerdo con las especificaciones del diseño técnico y garantizan tolerancia cero a fallos de persistencia o recursos huérfanos.
