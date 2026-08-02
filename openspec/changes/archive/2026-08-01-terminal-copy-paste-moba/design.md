## Context

Cada sesión crea un `Terminal` xterm en `main.ts` con `onData` → `write_ssh_input` (buffer ~16ms). No hay hoy `onSelectionChange` ni handler de `contextmenu` en el canvas. La app usa menús B3 en sidebar/explorador; la terminal **no** tiene menú contextual.

## Goals / Non-Goals

**Goals:**

- Auto-copy al seleccionar texto en xterm.
- Clic derecho → paste al PTY vía el mismo camino de input que el teclado.
- Strip solo trailing newline/whitespace al final del string pegado.
- Multilínea interna intacta.
- Ctrl+C sin cambios (SIGINT remoto).

**Non-Goals:**

- Menú contextual en terminal.
- Toggle en preferencias.
- Remapear Ctrl+C / Ctrl+V.
- Cambiar copy de Snippets / `user@host`.

## Decisions

1. **Auto-copy con `term.onSelectionChange`**
   - Si `term.getSelection()` no está vacío → `navigator.clipboard.writeText(...)`.
   - Ignorar selección vacía (clear).
   - Si `writeText` falla: log; no romper la sesión.

2. **Clic derecho = paste**
   - `contextmenu` en el contenedor del terminal (`terminal-canvas-container` o equivalente), `preventDefault`.
   - `clipboard.readText()` → sanitizar → enviar al PTY (reutilizar buffer `onData` / `write_ssh_input`, o `term.paste` si alimenta el mismo `onData`).
   - Solo en el viewport de esa terminal; no afectar sidebar.

3. **Sanitizado de paste**
   - Función pura: quitar al final solo `[\r\n]+` y/o whitespace trailing (`/\s+$/u` o equivalente que cubra `\r`, `\n`, espacio, tab).
   - **No** tocar Enter en medio del texto.
   - Unit test del helper (TDD).

4. **Ctrl+C**
   - No interceptar para copy; el remoto sigue recibiendo interrupt.

5. **Clipboard API**
   - ~~Empezar con `navigator.clipboard`~~ → **Superseded** por Corrección de Ruta (Fix) abajo.

## Risks / Trade-offs

- **[Riesgo] `onSelectionChange` dispara muchas veces al arrastrar** → Mitigación: copiar solo si hay texto; opcional debounce corto si molesta en verify.
- ~~**[Riesgo] Permiso / fallo de `clipboard.readText`**~~ → Resuelto con plugin Tauri (ver Fix).
- **[Riesgo] Paste multilínea ejecuta varias líneas en el shell** → Aceptado: solo se pide strip del final.
- **[Riesgo] Clic derecho en zona header de la pestaña (status)** → Mitigación: listener solo en canvas xterm, no en toda la pestaña.
- **[Trade-off] Sin menú “Pegar”** → Aceptado (estilo Moba).

## Migration Plan

- Frontend + registro del plugin clipboard en Tauri; sin migración de datos.
- Rollback: quitar plugin + listeners; volver a sin gestos (o a `navigator.clipboard` si hiciera falta).

## Open Questions

- (ninguna bloqueante; acordado en explore)

### Corrección de Ruta (Fix) — clipboard nativo Tauri

**Problema:** `navigator.clipboard` en el WebView pide permiso al usuario (sobre todo al leer/pegar). Eso no es aceptable para el flujo Moba.

**Estrategia:**
1. Añadir `@tauri-apps/plugin-clipboard-manager` (npm) + `tauri-plugin-clipboard-manager` (Cargo).
2. Registrar `.plugin(tauri_plugin_clipboard_manager::init())` en `lib.rs` (mismo patrón que opener/sql).
3. En gestos de terminal: `writeText` / `readText` del plugin (no `navigator.clipboard`).
4. Fallos: catch + log; no prompt de permiso del browser.
5. Snippets / copy `user@host` fuera de alcance de este fix (pueden seguir con navigator por ahora).

**Zombie:** llamadas `navigator.clipboard.*` en el setup xterm de `main.ts` → reemplazar (§5).
