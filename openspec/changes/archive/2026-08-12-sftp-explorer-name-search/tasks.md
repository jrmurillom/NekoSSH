**Surface types:** desktop-ui

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/sftp-explorer-name-search`
- [x] 0.2 Verificar rama actual con `git branch --show-current`

## 1. Lógica de filtro (TDD)

- [x] 1.1 Añadir módulo puro de filtro (p. ej. `explorer-name-filter.ts`) con: match case-insensitive por substring; un nodo visible si coincide o si algún descendiente bajo expansión/cargado coincide
- [x] 1.2 Escribir unit tests del filtro (match, empty query = todo, padre ancla, nodos colapsados no inspeccionados como hijos ocultos, sin coincidencias)
- [x] 1.3 Implementar hasta que los tests pasen

## 2. UI del filtro en Archivos

- [x] 2.1 Añadir en `app/index.html` el campo de filtro y el botón × en/bajo `#files-toolbar`
- [x] 2.2 Estilos en `app/src/styles.css` alineados al panel Archivos / tokens existentes; × visible cuando hay texto
- [x] 2.3 Estado vacío “(sin coincidencias)” cuando el filtro no deja filas

## 3. Integración en el explorador

- [x] 3.1 Estado `explorerNameFilter` (o equivalente) y listeners de input / × en `app/src/main.ts`
- [x] 3.2 Aplicar el filtro en `renderExplorerTree` / construcción de nodos sin mutar `explorerRoot` ni invocar SFTP
- [x] 3.3 Conservar el texto del filtro al navegar/expandir/actualizar; solo × o campo vacío limpian

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar tests existentes del explorador / helpers afectados y ajustarlos si hace falta

## 5. Run Unit Tests and Verify Local DB (MANDATORY)

- [x] 5.1 Ejecutar unitarios del área tocada (filtro + suite frontend relevante)
- [x] 5.2 Documentar en `openspec/changes/sftp-explorer-name-search/reports/YYYY-MM-DD-step-5-unit-test-and-db-verification.md` (DB: N/A — sin persistencia en este change)

## 6. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [ ] 6.1 Arrancar la app y verificar: filtrar nivel actual, filtrar hijos de carpeta expandida, padre ancla, × limpia, sin coincidencias, sin listado SFTP extra al filtrar
- [ ] 6.2 Report en `openspec/changes/sftp-explorer-name-search/reports/YYYY-MM-DD-step-6-desktop-ui-verification.md`

## 7. Update Technical Documentation (MANDATORY)

- [ ] 7.1 Actualizar SSOT tocados si aplica (`docs/` / notas de explorador) según `documentation-standards.md`; si no hay cambio de docs de producto, documentar N/A en el report o en esta tarea
