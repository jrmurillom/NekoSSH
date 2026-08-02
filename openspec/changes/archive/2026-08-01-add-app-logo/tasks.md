**Surface types:** desktop-ui

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/add-app-logo` antes de modificar el código de la app.
- [x] 0.2 Verificar que la rama activa en Git sea `feature/add-app-logo`.

## 1. Setup y Preparación de Recursos

- [x] 1.1 Copiar el icono `128x128.png` desde `app/src-tauri/icons/` hacia `app/src/assets/logo.png`.

## 2. Estructura HTML

- [x] 2.1 Modificar [app/index.html](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html) para envolver el título "NekoSSH" y el subtítulo dentro de un contenedor con clase `brand-container`.
- [x] 2.2 Agregar la etiqueta `<img>` para el logo apuntando a `/src/assets/logo.png` con la clase `brand-logo`.

## 3. Estilos CSS (Look & Feel)

- [x] 3.1 Añadir reglas CSS para `.brand-container`, `.brand-logo` y `.brand-text` en [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css).
- [x] 3.2 Aplicar el alineamiento horizontal, tamaño máximo de `32px` y el efecto de resplandor `drop-shadow` con el color de neón sakura.

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar y ajustar las pruebas unitarias existentes que puedan verse afectadas por el cambio.

## 5. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 5.1 Ejecutar la suite de pruebas unitarias (`npm run test` en la carpeta `app`).
- [x] 5.2 Generar el reporte de validación en `openspec/changes/add-app-logo/reports/2026-08-01-step-5-unit-test-and-db-verification.md` indicando el éxito de los tests y que el estado de base de datos es N/A (sin persistencia en este cambio).

## 6. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Arrancar la aplicación de escritorio y validar visualmente que el logo se renderiza de forma horizontal a la izquierda del título, alineado correctamente y con total nitidez.
- [x] 6.2 Crear el reporte de evidencia visual en `openspec/changes/add-app-logo/reports/2026-08-01-step-desktop-ui-verification.md` detallando la validación de la interfaz.

## 7. Update Technical Documentation (MANDATORY)

- [x] 7.1 Actualizar el contrato de layout [docs/design/ui-layout-contract.md](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/docs/design/ui-layout-contract.md) y [docs/design/DESIGN.md](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/docs/design/DESIGN.md) para registrar el contenedor del logo y el ajuste de márgenes del header del sidebar.
