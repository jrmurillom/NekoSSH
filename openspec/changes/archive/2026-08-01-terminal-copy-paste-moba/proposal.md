## Why

La terminal SSH no ofrece el flujo rápido tipo MobaXterm: seleccionar para copiar y clic derecho para pegar. Al pegar un comando con Enter al final, ese salto se envía al PTY y ejecuta sin querer. Hace falta ese gesto y limpiar solo el Enter/vacío final del texto pegado.

## What Changes

- Al seleccionar texto en el viewport xterm, copiar automáticamente al clipboard.
- Clic derecho en el viewport de terminal: pegar desde el clipboard (sin menú contextual en terminal).
- Antes de pegar: quitar solo saltos de línea / vacío **al final** del texto (`\n`, `\r`, `\r\n`, espacios/tabs finales). Saltos internos (multilínea) se conservan.
- Ctrl+C sigue siendo interrupt al remoto (SIGINT); no se redefine como “copiar”.
- Sin preferencia/toggle en este slice (comportamiento siempre activo).

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `ssh-terminal`: Gestos de clipboard en el emulador xterm (auto-copy por selección; paste por clic derecho con strip de trailing newline/whitespace).

## Impact

- Frontend: `app/src/main.ts` (setup de cada `Terminal` / canvas) + helper de strip.
- Tauri: plugin `clipboard-manager` (npm + Cargo + registro en `lib.rs`) para read/write del portapapeles del SO **sin** prompt de permiso del WebView.
- Docs: `ui-layout-contract.md` (gestos de terminal).
- Fuera de alcance: menú contextual en terminal; cambiar Ctrl+C; prefs de toggle; migrar Snippets/`user@host` al plugin (opcional después).
