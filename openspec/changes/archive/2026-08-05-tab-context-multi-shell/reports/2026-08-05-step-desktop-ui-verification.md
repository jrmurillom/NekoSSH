# Verificación desktop-ui — contexto multi-shell (PARCIAL)

**Change:** tab-context-multi-shell
**Fecha:** 2026-08-05
**Rama:** `feature/tab-context-multi-shell`
**Estado:** parcial — verificación estructural ejecutada; checklist visual interactivo pendiente

## Qué se ejecutó

La app estaba corriendo en modo dev (`tauri dev` con HMR) durante la implementación. Se comprobó que el servidor de desarrollo sirve el código nuevo del contexto multi-shell:

```
GET http://localhost:1420/src/main.ts
  term-grid          presente
  addChildShell      presente
  closeChildShell    presente
  focusShellPane     presente
  shell-grid-helper  presente
  term-cell-close    presente
  MAX_CHILD_SHELLS   presente

GET http://localhost:1420/src/styles.css
  .term-grid         presente
  .term-cell         presente
  cells-4            presente
  term-add-shell     presente
```

Complementado con:

- `vitest run` — 33 tests (8 nuevos de `shell-grid-helper`: límite de hijos, densidades `cells-1..4`, etiqueta de hijo, foco tras cerrar celda).
- `tsc && vite build` — sin errores de tipos ni de bundle.
- Smoke real contra servidor SSH (ver report desktop-commands): 4 logins concurrentes, eco aislado por shell, resize por sesión, cierre de hijo sin afectar al resto.

## Qué queda pendiente (requiere interacción en la ventana)

Estos puntos dependen de render real y no fueron automatizables en este entorno (no hay automatización de UI para la ventana Tauri):

- [ ] Botón `+` abre celdas: 1 → 2 → 3 (forma T) → 4 (2×2), y queda deshabilitado con 3 hijos.
- [ ] Cerrar un hijo con `×` reacomoda el grid y el explorador SFTP sigue operativo.
- [ ] Cerrar la pestaña con hijos pide confirmación y cierra todas las sesiones.
- [ ] Caída del padre: contexto desconectado, hijos cerrados, Ctrl+R reconecta solo el padre (vuelve a 1 celda).
- [ ] SFTP con un hijo enfocado sigue apuntando al padre; “Abrir en Terminal” hace `cd` en el padre.
- [ ] Fondo configurable y glow sakura permanecen en `.terminal-panel` (una sola superficie) con 1–4 celdas y no se recorta texto por el `border-radius`.

## Decisión

Se acordó dejar este paso documentado como **parcial**; el checklist visual se cierra en una pasada posterior sobre la app en ejecución.
