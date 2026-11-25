# Verificación Step 6 — Renderizado Desktop UI en Build Compilada

- **Fecha:** 2026-11-25
- **Cambio OpenSpec:** `ai-copilot-clean-hud`

## Verificación Headless contra `app/dist/ai-copilot.html`

- **Entorno:** Servidor estático local sobre `app/dist` (`127.0.0.1:4178`) inspeccionado con Microsoft Edge `--headless --dump-dom`.
- **Aserciones verificadas sobre el DOM hidratado real:**
  - `DOM_LEN=12488`
  - `HAS_HEADER=True` — Única cabecera superior `.copilot-header` presente.
  - `NO_TOOLBAR=True` — Toolbar superior redundante eliminada.
  - `NO_STEPPER=True` — Barra stepper superior separada eliminada.
  - `HAS_DOCK=True` — Dock compositor unificado `.copilot-composer-dock` montado en el pie con `Escucha`, `Modo`, selector de modelo y `Enviar`.
  - `HAS_HERO=True` — Estado vacío único `.copilot-empty-hero` renderizado al iniciar sin tarjetas duplicadas.
  - `HAS_SVG=True` — Iconos Lucide outline hidratados correctamente (`<svg`).
  - `HAS_AUTOCOMPLETE_OFF=True` — Atributo `autocomplete="off"` activo en campos de entrada según `base-standards.md`.
