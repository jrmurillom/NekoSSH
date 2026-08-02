**Surface types:** desktop-ui

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/style-terminal-tabs` antes de modificar el código de la app.
- [x] 0.2 Verificar que la rama activa en Git sea `feature/style-terminal-tabs`.

## 1. Estilos de Pestañas y Fusión

- [x] 1.1 Modificar la clase `.term-tab` en [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css) para aplicar el redondeo superior de `12px` y el fondo translúcido Cyber-Sakura.
- [x] 1.2 Actualizar `.term-tab.active` con posicionamiento superpuesto (`margin-bottom: -1px`) y fondo unificado.
- [x] 1.3 Agregar el pseudo-elemento `.term-tab.active::after` para ocultar la línea superior del panel y fundir ambas capas.

## 2. Estilos del Panel Contenedor de Terminal

- [x] 2.1 Modificar `.terminal-panel` en [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css) para aplicar `border-radius: var(--border-radius-lg, 24px)`.
- [x] 2.2 Agregar el borde sakura y el efecto de resplandor `box-shadow` continuo en `.terminal-panel` respetando los tokens y el principio sin glow externo a la consola.
- [x] 2.3 Ajustar `.terminal-canvas-container` para aplicar un padding de seguridad de `24px` protegiendo las esquinas inferiores.

## 3. Review and Update Existing Unit Tests (MANDATORY)

- [x] 3.1 Revisar y ajustar las pruebas unitarias que puedan verse afectadas por los cambios.

## 4. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 4.1 Ejecutar la suite de pruebas unitarias (`npm run test` en la carpeta `app`).
- [x] 4.2 Generar el reporte de validación en `openspec/changes/style-terminal-tabs/reports/2026-08-01-step-4-unit-test-and-db-verification.md` detallando el resultado de los tests and que el estado de base de datos es N/A.

## 5. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 5.1 Compilar la aplicación y validar visualmente el redondeo unificado, la ausencia de líneas divisorias en la pestaña activa y que el texto no se recorte en esquinas.
- [x] 5.2 Crear el reporte de evidencia visual en `openspec/changes/style-terminal-tabs/reports/2026-08-01-step-desktop-ui-verification.md` detallando la validación del renderizado.

## 6. Update Technical Documentation (MANDATORY)

- [x] 6.1 Actualizar el contrato estructural [docs/design/ui-layout-contract.md](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/docs/design/ui-layout-contract.md) para registrar las dimensiones del padding de la terminal y la unificación de pestañas.
- [x] 6.2 Modificar el manual visual [docs/design/DESIGN.md](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/docs/design/DESIGN.md) para documentar el caso de excepción del glow del contenedor unificado y los tokens asociados.

## 7. Corrección de Margen Exterior para Visibilidad del Glow v2 (Fix Real)

- [x] 7.1 Revertir el `padding` incorrecto de `.main-display-area` y restaurar `overflow: hidden`.
- [x] ~~(DESCARTADO) 7.2 Aplicar `margin` + `calc()` en `.terminal-panel` — generaba desbordamiento en WebView de Tauri y panel negro por conflicto con xterm.js~~
- [x] 7.3 Alinear `.terminal-tabs-bar` con `padding: 12px 20px 0 20px`.
- [x] 7.4 Re-compilar la aplicación (`npm run build`) verificando que el bundle compile limpiamente.
- [x] 7.5 Actualizar el reporte de evidencia visual detallando el fix aplicado correctamente.

## 8. Fix v3 — Posicionamiento Absoluto del Panel y Revertir Canvas Padding

- [x] 8.1 Refactor: cambiar `.terminal-panel` de `position: relative` + `width: calc(100% - 40px)` + `height: calc(100% - 20px)` + `margin` a `position: absolute; top: 0; left: 20px; right: 20px; bottom: 20px; width: auto; height: auto;` — elimina desbordamiento en WebView de Tauri.
- [x] 8.2 Revertir `.terminal-canvas-container` de `padding: 24px` a `padding: 0` — el padding era la causa del panel negro (xterm renderiza su viewport con `position: absolute; top: 0; left: 0` ignorando el padding CSS del padre).
- [x] 8.3 Re-compilar el frontend con `npm run build`.
- [x] 8.4 Actualizar el reporte de evidencia visual con los resultados definitivos.

## 9. Fix v4 — Corrección de border-radius superior-izquierdo del Panel

- [x] 9.1 En [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css), cambiar el `border-radius` de `.terminal-panel` de `var(--border-radius-lg, 24px)` uniforme a `0 var(--border-radius-lg, 24px) var(--border-radius-lg, 24px) var(--border-radius-lg, 24px)` — la esquina superior-izquierda queda cuadrada para conectar flush con la pestaña activa.
- [x] 9.2 Re-compilar el frontend con `npm run build` y verificar que compila limpiamente.
- [x] 9.3 Actualizar el reporte de evidencia visual documentando el fix aplicado.
