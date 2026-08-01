**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Feature Branch (MANDATORY - FIRST STEP)

- [x] 0.1 Crear y cambiar a la rama `feature/fase4-snippets-manager` (ya creada en propose)
- [x] 0.2 Verificar rama actual y working tree listo para cambios en `app/`

## 1. Persistencia SQLite + módulo Rust (TDD)

- [x] 1.1 Test-first: `ensure_snippets_schema` + CRUD categorías/snippets (cascade) en SQLite in-memory
- [x] 1.2 Migración `004_snippets.sql` (tablas `snippet_categories` / `snippets`) y registro en plugin SQL de `lib.rs`
- [x] 1.3 Módulo `snippets.rs`: helpers list/create/update/delete + seed demo idempotente (Apache, Tomcat, Permisos)
- [x] 1.4 Integrar `ensure_snippets_schema` en el path rusqlite de arranque/commands (junto a folders/preferences)

## 2. Commands IPC (desktop-commands)

- [x] 2.1 Exponer commands: list/create/delete categorías; list/create/update/delete snippets; ensure seed
- [x] 2.2 Registrar handlers en `generate_handler!` y smoke/unit que cubran éxito + error de validación
- [x] 2.3 Filtro opcional por `category_id` y `query` (título/cuerpo) en listado

## 3. Modal UI + trigger footer (desktop-ui)

- [x] 3.1 Botón Snippets en `sidebar-footer` (icono Lucide + label); abre/cierra modal; sin atajo de teclado
- [x] 3.2 Markup/CSS del modal glass reutilizando patrón profile-modal (ancho cómodo para lista)
- [x] 3.3 Chips (Todas + categorías), lista plana, búsqueda; wiring a commands
- [x] 3.4 Formularios in-modal: nueva categoría, nuevo snippet, editar título/cuerpo
- [x] 3.5 Acción Copiar → `navigator.clipboard.writeText(body)` + manejo de error (sin PTY)
- [x] 3.6 Acción Eliminar snippet/categoría → `confirmDialog` A1 (impacto cascade en categorías)

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar tests Rust/TS afectados; ajustar fixtures si el schema de arranque cambia
- [x] 4.2 Añadir/ajustar pruebas del seed (vacío vs ya poblado) y cascade

## 5. Run Unit Tests and Verify State (MANDATORY)

- [x] 5.1 Ejecutar suite del área (`cargo test` módulos snippets/preferences/lib + `npm run build` si aplica)
- [x] 5.2 Baseline/restore SQLite si se mutó DB de usuario; documentar N/A si solo in-memory
- [x] 5.3 Report: `openspec/changes/fase4-snippets-manager/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md`

## 6. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Invocar commands de snippets (list/create/update/delete/seed) vía harness/CLI/tests de integración; casos error
- [x] 6.2 Verificar persistencia y restaurar estado si se usó DB real
- [x] 6.3 Report: `openspec/changes/fase4-snippets-manager/reports/YYYY-MM-DD-step-desktop-commands-verification.md`

## 7. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 7.1 Arrancar app; abrir modal desde footer; chips + búsqueda; CRUD; copiar; delete con A1
- [x] 7.2 Verificar seed demo y que no hay shortcut ni inserción PTY
- [x] 7.3 Report: `openspec/changes/fase4-snippets-manager/reports/YYYY-MM-DD-step-desktop-ui-verification.md`

## 8. Update Technical Documentation (MANDATORY)

- [x] 8.1 Actualizar `docs/project_scope.md`: Fase 4a snippets (este alcance); Fase 4b Petdex diferido
- [x] 8.2 Actualizar `docs/design/ui-layout-contract.md`: fila Fase 4 — botón footer + modal snippets; sin tercera columna
- [x] 8.3 Ajustar `DESIGN.md` solo si faltan tokens de chips/lista (preferir reutilizar existentes)

## 9. Fix UX modal (preview aprobada)

Referencia: `docs/design/preview-snippets-modal.html`

- [x] 9.1 Alinear search + botón “+ Snippet” a tokens/chrome del modal de perfil (Cyber-Sakura); CSS solo `#snippets-modal` / `.snippets-*`
- [x] 9.2 Separación visual entre filas de la lista (divider / contenedor de lista)
- [x] 9.3 Quitar repetición de categoría en cada fila; mostrar título + comando/body
- [x] 9.4 Reemplazar `window.prompt` de nueva categoría por panel/form **in-modal** (mismo patrón que nuevo snippet)
- [x] 9.5 `npm run build` + verificación visual vs preview; nota corta en `reports/` si aplica

## 10. Fix toolbar layout (paridad con preview)

- [x] 10.1 Corregir CSS del toolbar **solo** bajo `#snippets-modal`: search a la izquierda (`flex: 1; min-width: 0`), “+ Snippet” a la derecha (`flex-shrink: 0`); quitar `width: 100%` que rompe la fila
- [x] 10.2 Confirmar que **ningún** selector global de `styles.css` fue modificado en este fix
- [x] ~~10.3~~ ~~(INCOMPLETO) build solo no basta — ver §11~~

## 11. Fix toolbar — paridad real con preview (gate)

- [x] 11.1 Diagnosticar por qué `#snippets-search` no se ve / no comparte fila con “+ Snippet” (inspect CSS bajo `#snippets-modal` only)
- [x] 11.2 Corregir layout para igualar `docs/design/preview-snippets-modal.html` (search izquierda + botón derecha, misma fila); **sin** tocar selectores globales
- [x] 11.3 Evidence gate: documentar en report que search y botón coexisten en una fila (descripción o captura); **no** marcar done solo con `npm run build`
- [x] 11.4 `npm run build` OK tras el fix

## 12. Fix select categoría (tema Cyber-Sakura)

- [x] 12.1 Estilar `#snippets-modal .snippets-select` (y `option` / `color-scheme`) para paridad con `.snippets-field` y `preview-snippets-modal.html`; **sin** tocar `select` global
- [x] 12.2 Alinear preview HTML del select si hace falta
- [x] 12.3 `npm run build` + nota en `reports/` (evidencia: select temático en form nuevo snippet)

## 13. Fix footer — engrane + Snippets temático

Referencia visual: `docs/design/preview-footer-gear-snippets.html`  
(§12 y §13 implementados en el mismo apply; ver `reports/2026-07-31-fix-footer-gear-and-select.md`.)

- [x] 13.1 Crear propuesta visual HTML standalone (`docs/design/preview-footer-gear-snippets.html`): footer mínimo + Snippets temático + engrane con popover (editor/fondo/opacidad)
- [x] 13.2 Gate de aprobación: usuario revisa el preview HTML antes de implementar en app — aprobado por usuario 2026-07-31 vía `/opsx:fix`
- [x] 13.3 Implementar footer: ocultar prefs inline; añadir icono engrane; popover/panel con editor externo, fondo y opacidad (misma capacidad; CSS scoped)
- [x] 13.4 Restyle del botón Snippets del footer al tema Cyber-Sakura (tokens; sin regressión del modal)
- [x] 13.5 `npm run build` + evidencia en `reports/` (footer mínimo cerrado + popover abierto; Snippets temático)
