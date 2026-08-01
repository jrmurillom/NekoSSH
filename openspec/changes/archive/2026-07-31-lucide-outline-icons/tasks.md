**Surface types:** desktop-ui

## 0. Setup: Feature Branch (MANDATORY - FIRST STEP)

- [x] 0.1 Crear y cambiar a la rama `feature/lucide-outline-icons` desde el estado actual de trabajo
- [x] 0.2 Verificar rama actual y working tree listo para cambios de código en `app/`

## 1. Dependencia y helper de iconos

- [x] 1.1 Añadir dependencia `lucide` en `app/package.json` e instalar
- [x] 1.2 Crear módulo helper (p. ej. `app/src/icons.ts`) que cree SVG Lucide outline con `stroke=currentColor`, `fill=none`, tamaño/stroke-width configurables
- [x] 1.3 Estilos base `.icon` (y variantes muted/danger si aplica) enlazados a tokens CSS de tema

## 2. Sustituir iconos en la UI

- [x] 2.1 Explorador: Subir / Ir / Actualizar + chevron expand/collapse + Folder/File en nodos
- [x] 2.2 Perfiles: Editar / Eliminar (quitar emojis)
- [x] 2.3 Pestañas de terminal: botón cerrar; otros botones icónicos existentes (Plus/nuevo si aplica)
- [x] 2.4 Verificar `title`/`aria-label` en controles solo-icono

## 3. Review and Update Existing Unit Tests (MANDATORY)

- [x] 3.1 Revisar si hay tests frontend/unitarios afectados; ajustar o documentar N/A si no existe suite UI

## 4. Run Unit Tests and Verify State (MANDATORY)

- [x] 4.1 Ejecutar tests relevantes (`cargo test` si no se tocó Rust; build/frontend check en `app/`)
- [x] 4.2 Report `openspec/changes/lucide-outline-icons/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md` (DB N/A)

## 5. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 5.1 Validar en runtime (o build + inspección): iconos outline visibles, sin emojis residuales en chrome tocado, color hereda tema (AGENT MUST EXECUTE)
- [x] 5.2 Generar `openspec/changes/lucide-outline-icons/reports/YYYY-MM-DD-step-desktop-ui-verification.md`

## 6. Update Technical Documentation (MANDATORY)

- [x] 6.1 Actualizar `docs/design/DESIGN.md` con convención Lucide outline + `currentColor`/tokens
- [x] 6.2 Actualizar `docs/design/ui-layout-contract.md` (y README si aplica) sobre iconografía de chrome
