**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Feature Branch (MANDATORY - FIRST STEP)

- [x] 0.1 Crear y cambiar a la rama `feature/fase2` desde el estado actual de trabajo
- [x] 0.2 Verificar rama actual y working tree listo para cambios de código en `app/`

## 1. Backend: Canal SFTP concurrente

- [x] 1.1 Añadir estado Rust para handles SFTP por `terminal_id` y abrirlo al conectar (misma auth que el PTY)
- [x] 1.2 Cerrar/liberar SFTP en `close_ssh_session` y en fallos de conexión
- [x] 1.3 Implementar command `sftp_list_dir` (path → entradas nombre/tipo/metadata mínima)

## 2. Backend: Cwd y cd para sync

- [x] 2.1 Implementar command para obtener cwd remoto (preferir exec/`pwd` fuera del PTY visible)
- [x] 2.2 Implementar o reutilizar envío de `cd` seguro al PTY (escaping de paths) y devolver cwd confirmado

## 3. Frontend: Explorador en sidebar

- [x] 3.1 Habilitar pestaña Archivos; empty state sin sesión; árbol lazy con tokens planos (sin glow)
- [x] 3.2 Conectar listado SFTP al expandir nodos y al abrir Archivos con sesión activa
- [x] 3.3 Menú contextual “Abrir en Terminal” → cd + refresco del explorador
- [x] 3.4 Listener/poll de cwd: al cambiar el directorio en terminal, actualizar selección/ruta del árbol

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar tests CRUD/SSH existentes afectados por el nuevo estado SFTP y ajustar mocks/fixtures si aplica
- [x] 4.2 Añadir pruebas unitarias Rust para listado SFTP mockeable o capa de path-escaping/`cd` según diseño

## 5. Run Unit Tests and Verify State (MANDATORY)

- [x] 5.1 Ejecutar `cargo test --manifest-path app/src-tauri/Cargo.toml` (y subset frontend si existe)
- [x] 5.2 Documentar baseline/restore N/A o DB si se tocó persistencia; crear `openspec/changes/fase2/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md`

## 6. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Verificar commands SFTP/cwd/cd vía harness o app runtime (AGENT MUST EXECUTE)
- [x] 6.2 Generar `openspec/changes/fase2/reports/YYYY-MM-DD-step-desktop-commands-verification.md`

## 7. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 7.1 Validar tab Archivos, árbol, Abrir en Terminal y sync tras `cd` (AGENT MUST EXECUTE)
- [x] 7.2 Generar `openspec/changes/fase2/reports/YYYY-MM-DD-step-desktop-ui-verification.md`

## 8. Update Technical Documentation (MANDATORY)

- [x] 8.1 Actualizar `docs/design/ui-layout-contract.md` (Fase 2: explorador + sync)
- [x] 8.2 Revisar `README.md` / docs si hay comandos o flujos nuevos de usuario

## 9. Fix: Cwd real del PTY + path bar (Ir / Actualizar)

- [x] ~~9.1 ~~(DESCARTADO)~~ sustituido por 10.1 — sonda PTY con marcadores descartada~~
- [x] ~~9.2 ~~(DESCARTADO)~~ cwd vía sonda PTY; reemplazado por OSC 7 (10.2)~~
- [x] ~~9.3 ~~(DESCARTADO)~~ sync post-sonda; reemplazado por parseo OSC 7 (10.3)~~
- [x] ~~9.4 ~~(DESCARTADO)~~ path bar se replanifica en 10.4~~
- [x] ~~9.5 ~~(DESCARTADO)~~ tests de marcadores; reemplazado por 10.5 (OSC 7)~~
- [x] ~~9.6 ~~(DESCARTADO)~~ report se rehace en 10.6~~
- [x] ~~9.7 ~~(DESCARTADO)~~ desktop-commands se rehace en 10.7~~
- [x] ~~9.8 ~~(DESCARTADO)~~ desktop-UI se rehace en 10.8~~
- [x] ~~9.9 ~~(DESCARTADO)~~ docs se rehace en 10.9~~

## 10. Fix v2: Sesión única + OSC 7 + path bar

- [x] 10.1 Refactor: una sola Session SSH (PTY+SFTP); eliminar restos de 2º login / `SftpSessions`
- [x] 10.2 Frontend: parsear OSC 7 en salida PTY → actualizar `explorerCwd` y árbol; opcional one-liner al conectar para bash/zsh
- [x] 10.3 Quitar sync basado en `exec pwd` / poll / sonda PTY con marcadores
- [x] 10.4 UI: input de ruta + icono **Ir** (SFTP list) + icono **Actualizar**
- [x] 10.5 Unit tests: parser OSC 7 (path desde secuencia) + path escaping; sin tests zombies de marcadores exec-cwd
- [x] 10.6 `cargo test` + report unit/db (fix v2)
- [x] 10.7 Desktop-commands (AGENT MUST EXECUTE) + report (fix v2): SFTP list / ssh_cd / Ir path
- [x] 10.8 Desktop-UI (AGENT MUST EXECUTE) + report (fix v2): (1) OSC 7 o shell con cd → explorador; (2) path+Ir; (3) Actualizar; (4) teclado estable tras SFTP
- [x] 10.9 Docs: `ui-layout-contract.md` / README — OSC 7 + path bar; sin promesa de sync vía exec pwd

## 11. Fix v3: Árbol de archivos navegable

> Navegación del árbol = **SFTP**. OSC 7 = sync terminal→explorador (no sustituye al árbol).

- [x] 11.1 Fix: OSC 7 / refresh no debe destruir expand/collapse si el path no cambió
- [x] 11.2 Chevron = expand/collapse lazy (SFTP); click en carpeta = abrir (path bar + listar esa ruta vía SFTP)
- [x] 11.3 Control Subir al padre; empty/error/loading en árbol
- [x] 11.4 CSS: panel Archivos con altura/scroll usable para el árbol
- [x] 11.5 Ocultar o silenciar hook OSC 7 en el PTY (no ensuciar la terminal)
- [x] 11.6 Desktop-UI (AGENT MUST EXECUTE) + report fix v3: expand, collapse, abrir carpeta, subir, Ir; OSC mismo path no resetea
- [x] 11.7 Docs layout/README: navegación del árbol (expand vs abrir vs subir; SFTP vs OSC 7)

## 12. Fix v4: PTY solo + SFTP dedicado + smoke real (SIN seguir-cd)

> Alcance: (1) terminal estable (2) árbol SFTP navegable (4) pruebas reales. **Seguir cd = fuera.**

- [x] 12.1 Refactor: Session PTY exclusiva; quitar SFTP/`set_blocking` de esa Session
- [x] 12.2 Conexión SFTP dedicada (viva con la terminal); abrir/cerrar con el ciclo de vida; error claro si MaxSessions falla sin tumbar PTY
- [x] 12.3 `sftp_list_dir` usa solo la conexión SFTP; árbol: expand/abrir/subir/Ir/Actualizar/Abrir en Terminal funcionales
- [x] 12.4 Quitar sync OSC 7 → explorador e inyección de hooks en PTY (código/docs)
- [x] 12.5 Unit tests path/escaping; `cargo test` + report honesto
- [x] 12.6 Smoke SSH real (AGENT MUST EXECUTE): tipear sin caída + listar/navegar árbol; si no hay credenciales, documentar BLOQUEO y no marcar PASS falso — report fix v4
- [x] 12.7 Docs: ui-layout/README — dos conexiones; sin promesa de seguir-cd

## 13. Fix v5: Cerrar conexiones SSH con pestaña / app

> Objetivo: ninguna Session SSH queda viva de forma deliberada al cerrar pestaña(s) o la ventana.

- [x] 13.1 Backend: `close_ssh_session` debe quitar del mapa, cerrar canal PTY, `session.disconnect` (o drop ordenado) y ser idempotente si ya no existe
- [x] 13.2 Backend: al salir de la app / `CloseRequested` / `ExitRequested`, cerrar **todas** las entradas de `SshConnections` antes de terminar
- [x] 13.3 Frontend: al cerrar pestaña (y “cerrar todas”), siempre invocar cleanup; limpiar explorador si estaba bound a ese `terminal_id`; no dejar listeners huérfanos
- [x] 13.4 Race-safe: el hilo reader PTY tolera remove del mapa durante close (sin panic; emite closed una sola vez)
- [x] 13.5 Unit/smoke: test o harness que abre sesión (o simula mapa), cierra pestaña/app path, y verifica mapa vacío; documentar en report fix v5 (honesto si smoke remoto no aplica)
- [x] 13.6 Docs/spec delta: requisito de lifecycle close en `specs/ssh-terminal` + nota breve en README/ui-layout si hace falta
