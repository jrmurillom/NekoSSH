# Smoke SSH Verification (fix v4)

- Date: 2026-07-30
- Change: fase2
- Agent: Cursor Auto (opsx-apply)

## Alcance requerido
1. Tipear en PTY sin que se cierre la sesión (con explorador listando).
2. Navegar árbol SFTP (expand / abrir / subir / Ir).

## Resultado: BLOQUEO

- **No hay credenciales/host de prueba** en el entorno del agente (sin `.env` ni secrets de SSH).
- Por eso **NO** se ejecutó smoke SSH real.
- **NO** se afirma PASS de estabilidad en vivo.

## Qué sí se verificó en código
- PTY: Session exclusiva (`LivePty`); sin `set_blocking`/SFTP sobre ella.
- SFTP: segunda Session (`LiveSftp`) abierta al conectar; listado solo ahí.
- Si SFTP falla al abrir: error en `SftpOpenErrors`; PTY no se cierra por eso.
- Frontend: sin listener `ssh-cwd` / sin inyección de hooks.

## Para desbloquear
Usuario/agente con host+user+auth: conectar, tipear varias teclas mientras Archivos lista, expandir/abrir carpeta, confirmar que PTY sigue vivo. Entonces actualizar este report a PASS con evidencia.

## Outcome
- Status: **BLOQUEO** (smoke no ejecutado)
- Unit/build aparte: PASS (ver report N+1 v4)
