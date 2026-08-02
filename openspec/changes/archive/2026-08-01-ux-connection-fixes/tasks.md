**Surface types:** desktop-ui

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/ux-connection-fixes` antes de modificar el código de la app.
- [x] 0.2 Verificar que la rama activa en Git sea `feature/ux-connection-fixes`.

## 1. Árbol de Conexiones Colapsado por Defecto

- [x] 1.1 Modificar `loadProfiles()` en [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts) para remover la auto-expansión inicial de `expandedFolderIds` y asegurar que el árbol inicie colapsado al abrir la app.

## 2. Remoción de Fondo Persistente en Categorías Padres

- [x] 2.1 En [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css), actualizar `.connection-tree .folder-row.is-active-context` a `background: transparent;` para prevenir la retención del tinte rosa al hacer clic.

## 3. Diálogo de Confirmación para Cierre de Conexiones Vivas

- [x] 3.1 Actualizar `closeTerminalSession` en [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts) para verificar `activeTerm.isConnected` y solicitar confirmación mediante `confirmDialog` antes de desconectar.
- [x] 3.2 Actualizar `closeAllTerminals` en [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts) para solicitar confirmación global una sola vez si existen conexiones vivas.

## 4. Inhabilitación Global del Menú Contextual Nativo del Navegador

- [x] 4.1 Registrar un listener global `document.addEventListener("contextmenu", (e) => e.preventDefault())` en [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts) para bloquear el menú contextual por defecto del navegador en zonas no utilizadas.

## 5. Selector Nativo del SO para Llaves Privadas

- [x] 5.1 Agregar el botón "Examinar..." con selector de archivos nativo en `#auth-key-group` en [app/index.html](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html) y [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css).
- [x] 5.2 Implementar el event handler en [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts) para abrir el explorador nativo del SO y auto-completar `#prof-key-path`.

## 6. Review and Update Existing Unit Tests (MANDATORY)

- [x] 6.1 Revisar y ajustar las pruebas unitarias que puedan verse afectadas por estos cambios de UX en [app/src](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src).

## 7. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 7.1 Ejecutar la suite de pruebas unitarias (`npm run test` en la carpeta `app`).
- [x] 7.2 Generar el reporte de validación en `openspec/changes/ux-connection-fixes/reports/2026-08-01-step-7-unit-test-and-db-verification.md` detallando el resultado de los tests y estado de DB.

## 8. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 8.1 Compilar el frontend (`npm run build`) y verificar que todos los bundles compilen sin errores.
- [x] 8.2 Crear el reporte de evidencia visual en `openspec/changes/ux-connection-fixes/reports/2026-08-01-step-desktop-ui-verification.md` detallando la verificación del árbol colapsado, confirmaciones de cierre, selector nativo y bloqueo de clic derecho.

## 9. Update Technical Documentation (MANDATORY)

- [x] 9.1 Actualizar la documentación técnica en `docs/design/ui-layout-contract.md` y `docs/design/DESIGN.md` si aplica.
