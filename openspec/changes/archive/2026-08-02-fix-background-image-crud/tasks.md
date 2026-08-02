**Surface types:** desktop-ui

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/fix-background-image-crud` antes de modificar el código de la app.
- [x] 0.2 Verificar que la rama activa en Git sea `feature/fix-background-image-crud`.

## 1. Ajuste de Capas CSS (Z-Index)

- [x] 1.1 En [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css), ajustar `.bg-overlay-layer` a `z-index: -2` y `.bg-image-layer` a `z-index: -1` para permitir que la imagen se renderice visiblemente sobre la base oscura.
- [x] 1.2 En [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css) y [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), habilitar la transparencia en `.terminal-panel` (background glassmorphism traslúcido) y xterm (`allowTransparency: true` y `theme.background: "transparent"`) para transparentar la terminal sobre la imagen de fondo.

## 2. Soporte de Rutas Locales con `convertFileSrc`

- [x] 2.1 En [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), importar `convertFileSrc` desde `@tauri-apps/api/core`.
- [x] 2.2 Actualizar `applyBackgroundSettings()` para convertir rutas locales con `convertFileSrc()` antes de asignarlas a `backgroundImage`.
- [x] 2.3 En [app/src-tauri/capabilities/default.json](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/capabilities/default.json), agregar el permiso `"core:asset:default"` para habilitar la carga de archivos locales desde el protocolo de activos de Tauri.
- [x] 2.4 En [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), implementar `FileReader.readAsDataURL` al seleccionar una imagen para garantizar su despliegue inmediato en el WebView2.

## 3. Controles CRUD, Normalización de Iconografía y Corrección Anti-Desbordamiento en Preferencias

- [x] 3.1 En [app/index.html](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html), reestructurar `#prefs-popover` eliminando etiquetas de texto en botones y usando icon-buttons compactos (`btn-icon` / `btn-icon-action` / `btn-secondary`) tanto para Editor Preferido (Input + Examinar SO + Guardar) como para Fondo de Imagen (Input + Examinar SO + Aplicar + Quitar).
- [x] 3.2 En [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css), ajustar `.prefs-popover`, `.config-row` e `input` con `min-width: 0`, `flex-shrink` y contenedores compactos para eliminar al 100% cualquier desbordamiento horizontal.
- [x] 3.3 En [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), inicializar la iconografía Lucide (`setButtonIcon`) para los botones de preferencias y conectar los event handlers de exploración nativa del SO (`btn-browse-bg`, `file-input-bg`, `btn-browse-editor`, `file-input-editor`, `btn-clear-bg`, `btn-apply-bg`, `btn-save-editor-pref`).

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Crear el módulo helper `app/src/bg-settings-helper.ts` y su suite de pruebas unitarias `app/src/bg-settings-helper.test.ts` para verificar la resolución de URLs (convertFileSrc local vs remotas vs limpias) y formateo de opacidad.
- [x] 4.2 Ejecutar la suite de pruebas unitarias en `app/src` con Vitest.

## 5. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 5.1 Ejecutar la suite de pruebas unitarias (`npm run test` en la carpeta `app`).
- [x] 5.2 Generar el reporte de validación en `openspec/changes/fix-background-image-crud/reports/2026-08-01-step-5-unit-test-and-db-verification.md`.

## 6. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Compilar el frontend (`npm run build`) y verificar que todos los bundles compilen sin errores.
- [x] 6.2 Crear el reporte de evidencia visual en `openspec/changes/fix-background-image-crud/reports/2026-08-01-step-desktop-ui-verification.md` detallando la prueba de selección, renderizado y borrado de la imagen de fondo.

## 7. Update Technical Documentation (MANDATORY)

- [x] 7.1 Actualizar la documentación técnica en `docs/design/DESIGN.md` si aplica.
