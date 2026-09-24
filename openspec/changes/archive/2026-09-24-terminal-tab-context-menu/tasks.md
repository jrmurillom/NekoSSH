**Surface types:** desktop-ui

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/terminal-tab-context-menu` (previa autorización explícita de Git si aplica)
- [x] 0.2 Verificar el estado limpio del árbol de trabajo

## 1. Overlay Context Menu: Soporte de Ítems Deshabilitados (`disabled?: boolean`)

- [x] 1.1 Extender `ContextMenuItem` en `app/src/overlays.ts` con `disabled?: boolean` para renderizar `btn.disabled = true`, `aria-disabled="true"` y clase `.is-disabled` bloqueando su activación por clic
- [x] 1.2 Añadir estilos planos `.chrome-context-item.is-disabled` en `app/src/styles.css` siguiendo `docs/design/DESIGN.md`

## 2. Módulo Puro y Suite de Pruebas Unitarias / E2E DOM (`terminal-tab-menu-helper`)

- [x] 2.1 Crear `app/src/modules/terminal-tab-menu-helper.ts` con `buildTabContextMenuState`, `resolveTabsToClose`, `buildBulkTabCloseConfirm` y `resolveNextActiveTabAfterBulkClose`
- [x] 2.2 Crear `app/src/modules/terminal-tab-menu-helper.test.ts` validando: (a) habilitación/deshabilitación en primera pestaña, pestaña intermedia, última pestaña y pestaña única, (b) cálculo exacto de IDs a cerrar en `close-left`, `close-right`, `close-others`, `close` y `close-all`, (c) textos de `confirmDialog` consolidado para 1 y N sesiones SSH activas, (d) transición determinista de foco (`switchActiveTerminal` hacia la pestaña clicada cuando la activa previa fue cerrada), y (e) flujo E2E simulado con confirmación positiva y cancelación

## 3. Integración en Barra de Pestañas (`app/src/main.ts`)

- [x] 3.1 Añadir iconos direccionales si aplica en `app/src/icons.ts` y conectar el listener `contextmenu` en `.term-tab` dentro de `openTerminalTab` (`app/src/main.ts`) con `handleTerminalTabContextMenu` y `closeTerminalTabsSubset`

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar y asegurar que todas las suites de pruebas existentes en frontend y backend pasen sin regresiones

## 5. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 5.1 Ejecutar `npm test` (Vitest) y `cargo test` (Rust) verificando 100% de aprobación y generar reporte en `openspec/changes/terminal-tab-context-menu/reports/`

## 6. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Verificar compilación de producción del frontend (`npm run build`) e integridad del menú contextual de pestañas, documentando evidencia en `openspec/changes/terminal-tab-context-menu/reports/`

## 7. Update Technical Documentation (MANDATORY)

- [x] 7.1 Actualizar documentación técnica si aplica según `docs/documentation-standards.md`
