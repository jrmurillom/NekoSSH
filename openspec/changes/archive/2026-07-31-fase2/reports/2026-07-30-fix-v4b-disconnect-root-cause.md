# Smoke SSH (update post-disconnect)

- Date: 2026-07-30
- Change: fase2

## Por qué seguía roto
La “conexión SFTP dedicada” (2º login) en VPS con un solo login concurrente **tumba el PTY**. Eso encaja con “Conexión Cerrada” al usar la app.

## Corrección aplicada
- Volver a **UNA** Session SSH.
- SFTP = canal subsystem en esa Session.
- **Sin** `set_blocking(true)` (reintentos WouldBlock).
- Al cerrar, la UI muestra el motivo (`EOF` / error de lectura).
- Explorador no lista al conectar si Archivos no está abierto.

## Pruebas intentadas aquí
- `cargo test`: 14 PASS
- `ssh localhost` BatchMode: **Permission denied** (sin clave/password local)
- Smoke contra `vmi908184`: **no posible** sin credenciales en el agente

## Outcome
- Código corregido; smoke remoto sigue bloqueado sin secrets.
- Usuario debe reiniciar app y revalidar tipeo + Archivos.
