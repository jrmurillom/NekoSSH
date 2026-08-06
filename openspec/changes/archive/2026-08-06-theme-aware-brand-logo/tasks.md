**Surface types:** desktop-ui

## 0. Setup: rama de feature (MANDATORY)

- [x] 0.1 Crear rama `feature/theme-aware-brand-logo` y verificar rama actual

## 1. Assets de logo por tema

- [x] 1.1 Copiar los 8 PNG de `docs/design/logos/` a `app/src/assets/logos/` con nombre = id del tema (`nekossh.png`, `hatsune-miku.png`, …)
- [x] 1.2 Renombrar `sailor_moon.png` → `sailor-moon.png` en diseño y en runtime
- [x] 1.3 Actualizar el `src` inicial de `.brand-logo` en `index.html` a `logos/nekossh.png` (o equivalente Vite)

## 2. Resolución y applyTheme

- [x] 2.1 Crear helper puro `resolveBrandLogoUrl(themeId)` con fallback a `nekossh` + tests unitarios
- [x] 2.2 En `applyTheme`, actualizar el `src` de `.brand-logo` con el resultado del helper (además de CSS/xterm)

## 3. Revisión de pruebas unitarias (MANDATORY)

- [x] 3.1 Revisar y ajustar pruebas existentes afectadas por el mapeo de logo/tema

## 4. Ejecutar pruebas unitarias y estado de datos (MANDATORY)

- [x] 4.1 Ejecutar suite frontend (y cargo test si tocó Rust; N/A esperado) y documentar estado de datos (N/A — sin persistencia de BD en este change)
- [x] 4.2 Report en `openspec/changes/theme-aware-brand-logo/reports/YYYY-MM-DD-step-4-unit-test-and-db-verification.md`

## 5. Verificación de UI de escritorio (MANDATORY - AGENT MUST EXECUTE)

- [x] 5.1 Checklist: boot con tema guardado ≠ nekossh muestra logo correcto; cambiar entre varios temas actualiza el logo; fallback visual si aplica
- [x] 5.2 Report en `openspec/changes/theme-aware-brand-logo/reports/YYYY-MM-DD-step-desktop-ui-verification.md`

## 6. Documentación técnica (MANDATORY)

- [x] 6.1 Actualizar `docs/design/DESIGN.md` (y branding si aplica) con la convención logo-por-tema y ubicación de assets
