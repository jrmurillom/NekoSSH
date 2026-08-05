**Surface types:** desktop-ui, desktop-commands

## 0. Setup: rama de feature (MANDATORY)

- [x] 0.1 Crear rama `feature/tab-context-multi-shell` y verificar rama actual

## 1. Modelo de contexto (frontend)

- [x] 1.1 Introducir `TabContext` / `ShellPane` (padre + hijos, focusedId, máx. 3 hijos) y migrar el mapa actual de pestañas sin cambiar UX de 1 celda
- [x] 1.2 Al conectar un perfil, crear contexto con shell padre y un solo `terminal_id` (comportamiento actual preservado)
- [x] 1.3 Enrutar eventos `ssh-stdout` / `ssh-closed` / `ssh-error` / `ssh-connected` al pane correcto por `terminal_id`

## 2. Ciclo de vida padre/hijos

- [x] 2.1 Acción “nuevo shell”: mismo perfil, nuevo `terminal_id`, `start_ssh_session`; deshabilitar al llegar a 3 hijos
- [x] 2.2 Cerrar hijo: `close_ssh_session` solo del hijo + reflow del grid + foco al padre o vecino
- [x] 2.3 Cerrar pestaña: confirmación si padre o algún hijo está conectado; cerrar todas las Sessions del contexto
- [x] 2.4 Caída del padre: marcar contexto desconectado, cerrar hijos, aviso + Ctrl+R reconecta solo el padre (layout 1 celda)

## 3. Layout de cuadrícula

- [x] 3.1 CSS `.term-grid` densidades `cells-1|2|3|4` (2 cols; T; 2×2) dentro de `.terminal-panel` sin fondo por celda
- [x] 3.2 Celdas: foco visual sakura, × solo en hijos, badge mínimo en padre; FitAddon por celda al cambiar layout/resize
- [ ] 3.3 Verificar fondo/opacidad/glow del panel unificado con 1–4 celdas (texto no recortado por radius) — pendiente: validación visual en ventana (ver report desktop-ui parcial)

## 4. Foco, I/O y SFTP

- [x] 4.1 Click/foco de celda dirige `onData`, resize y writes al `focusedTerminalId`
- [x] 4.2 Rebind explorador SFTP / clipboard SCP / “Abrir en Terminal” al `parentTerminalId` del contexto activo
- [x] 4.3 Al cambiar de pestaña, restaurar cwd/caché del contexto y rebind SFTP al padre

## 5. Revisión de pruebas unitarias (MANDATORY)

- [x] 5.1 Revisar y ajustar pruebas existentes afectadas por el modelo padre/hijos

## 6. Ejecutar pruebas unitarias y estado de datos (MANDATORY)

- [x] 6.1 Ejecutar `cargo test` en `app/src-tauri` y build del frontend; capturar estado de `nekossh.db` pre/post y restaurar si hubo mutación
- [x] 6.2 Report en `openspec/changes/tab-context-multi-shell/reports/2026-08-05-step-6-unit-test-and-db-verification.md`

## 7. Verificación de commands IPC (MANDATORY - AGENT MUST EXECUTE)

- [x] 7.1 Ejercitar `start_ssh_session` / `write_ssh_input` / `resize_ssh_pty` / `close_ssh_session` con varios `terminal_id` del mismo perfil (éxito + error)
- [x] 7.2 Report en `reports/2026-08-05-step-desktop-commands-verification.md`

## 8. Verificación de UI de escritorio (MANDATORY - AGENT MUST EXECUTE)

- [ ] 8.1 Ejecutar checklist: 1–4 celdas, cerrar hijo, cerrar tab, caída padre + Ctrl+R, SFTP con hijo enfocado, fondo transparente — PARCIAL (estructural hecho; visual pendiente)
- [x] 8.2 Report en `reports/2026-08-05-step-desktop-ui-verification.md`

## 9. Documentación técnica (MANDATORY)

- [x] 9.1 Actualizar `docs/design/ui-layout-contract.md` con el patrón de grid interno y padre estático
