**Surface types:** desktop-ui

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/fix-terminal-panel-background` antes de modificar el código de la app.
- [x] 0.2 Verificar que la rama activa en Git sea `feature/fix-terminal-panel-background`.

## 1. Asignación Directa de Fondo en `.terminal-panel`

- [x] 1.1 En [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css), aplicar `background-size: cover` y `background-position: center` sobre `.terminal-panel` con pseudo-elemento `.terminal-panel::before` para controlar la capa de opacidad de la terminal.
- [x] 1.2 En [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), actualizar `applyBackgroundSettings()` para inyectar `backgroundImage` e `--terminal-overlay-opacity` directamente sobre `.terminal-panel`.

## 2. Helper y Pruebas Unitarias

- [x] 2.1 En [app/src/bg-settings-helper.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/bg-settings-helper.ts), implementar `calculateTerminalOverlayOpacity(opacity: number): number`.
- [x] 2.2 En [app/src/bg-settings-helper.test.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/bg-settings-helper.test.ts), agregar pruebas unitarias para `calculateTerminalOverlayOpacity`.

## 3. Mandatory Build Verification (Rust & Vite)

- [x] 3.1 Ejecutar pruebas unitarias (`npm run test` en `app`).
- [x] 3.2 Compilar el frontend (`npm run build` en `app`).
- [x] 3.3 Compilar y verificar el backend Rust (`cargo check` en `app/src-tauri`).
- [x] 3.4 Generar el reporte de validación en `openspec/changes/fix-terminal-panel-background/reports/2026-08-01-step-build-verification.md`.
