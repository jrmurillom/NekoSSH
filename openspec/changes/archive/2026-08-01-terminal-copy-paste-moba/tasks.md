**Surface types:** desktop-ui, desktop-commands

## 0. Setup

- [x] 0.1 Crear/cambiar a la rama `feature/terminal-copy-paste-moba` antes de tocar código de app

## 1. Helper de sanitizado (TDD)

- [x] 1.1 Test-first: función que elimina solo trailing newline/whitespace al final; multilínea interna intacta
- [x] 1.2 Implementar el helper mínimo que pasa los tests

## 2. Gestos en xterm

- [x] 2.1 `onSelectionChange` → auto-copy con `clipboard.writeText` si hay selección
- [x] 2.2 `contextmenu` en el canvas → `preventDefault` → `readText` → sanitizar → input al PTY (mismo camino que teclado)
- [x] 2.3 Confirmar que Ctrl+C no se intercepta como copy; clic derecho no abre menú

## 3. Tests y verificación

- [x] 3.1 Revisar/ajustar unit tests del área; mantener tests del helper
- [x] 3.2 Ejecutar tests + build; DB N/A; report `reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md`
- [x] 3.3 Validación desktop-ui (agente): auto-copy; paste clic derecho; strip Enter final; multilínea interna; report `reports/YYYY-MM-DD-step-desktop-ui-verification.md`

## 4. Documentación

- [x] 4.1 Actualizar `ui-layout-contract.md` (gestos de terminal: selección=copy, clic derecho=paste con strip final)
- [x] 4.2 `DESIGN.md` solo si hace falta mencionar el gesto (sin look nuevo de componente) — N/A: sin componente visual nuevo; gestos documentados en layout contract

## 5. Fix: clipboard nativo Tauri (opsx:fix — sin prompt WebView)

- [x] 5.1 Añadir `@tauri-apps/plugin-clipboard-manager` (npm) y `tauri-plugin-clipboard-manager` (Cargo)
- [x] 5.2 Registrar el plugin en `lib.rs` (patrón opener/sql)
- [x] 5.3 Reemplazar `navigator.clipboard` en gestos de terminal por `writeText`/`readText` del plugin
- [x] 5.4 Re-verificar: copy/paste terminal sin diálogo de permiso; strip intacto; report addendum en `reports/`
- [x] 5.5 Actualizar `ui-layout-contract.md` (clipboard vía plugin Tauri, no prompt WebView)
