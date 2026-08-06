**Surface types:** desktop-ui

## 0. Setup: rama de feature (MANDATORY)

- [x] 0.1 Crear rama `feature/per-theme-wallpaper` y verificar rama actual

## 1. Modelo de persistencia por tema

- [x] 1.1 Helper puro: mapa `{ themeId → { url, label, opacity } }`, get/set/clear + migración desde claves globales `nekossh-bg-*`
- [x] 1.2 Tests unitarios: CRUD por tema, aislamiento entre temas, migración one-shot, tema sin entry

## 2. Cableado en UI / applyTheme

- [x] 2.1 `persistBackground` / clear / opacity guardan bajo el tema activo
- [x] 2.2 `applyTheme` carga y aplica wallpaper (+ sincroniza inputs del popover) del tema seleccionado
- [x] 2.3 Boot/`initSettings`: migrar legacy y aplicar fondo del tema restaurado

## 3. Revisión de pruebas unitarias (MANDATORY)

- [x] 3.1 Revisar tests existentes de bg-settings afectados

## 4. Ejecutar pruebas unitarias y estado de datos (MANDATORY)

- [x] 4.1 Ejecutar suite frontend; documentar estado de datos (N/A BD — solo localStorage)
- [x] 4.2 Report en `openspec/changes/per-theme-wallpaper/reports/YYYY-MM-DD-step-4-unit-test-and-db-verification.md`

## 5. Verificación de UI de escritorio (MANDATORY - AGENT MUST EXECUTE)

- [x] 5.1 Checklist: fondo en tema A; cambiar a B (vacío o distinto); volver a A recupera A; quitar solo afecta activo; boot restaura
- [x] 5.2 Report en `openspec/changes/per-theme-wallpaper/reports/YYYY-MM-DD-step-desktop-ui-verification.md`

## 6. Documentación técnica (MANDATORY)

- [x] 6.1 Actualizar `docs/design/DESIGN.md` (wallpaper por tema + migración) según documentation-standards
