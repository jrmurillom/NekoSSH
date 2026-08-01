**Surface types:** desktop-ui, desktop-commands



## 0. Setup: Feature Branch (MANDATORY - FIRST STEP)



- [x] 0.1 Crear y cambiar a la rama `feature/manager-profiles` desde el estado actual de trabajo

- [x] 0.2 Verificar rama actual y working tree listo para cambios en `app/`



## 1. Schema y migración



- [x] 1.1 Añadir migración SQL: tabla `connection_folders` + columna `folder_id` en `profiles` + carpeta default + backfill

- [x] 1.2 Actualizar modelos Rust / schema init de tests en memoria



## 2. Backend CRUD



- [x] 2.1 Commands: list/create/update/delete folders (idempotente delete con cascade)

- [x] 2.2 Adaptar create/update/list de conexiones para exigir/exponer `folder_id`; listado árbol o folders+children

- [x] 2.3 Tests unitarios rusqlite: carpeta CRUD, conexión en carpeta, cascade, migración backfill



## 3. Frontend árbol e inline edit



- [x] 3.1 UI árbol sidebar: carpetas (chevron + folder Lucide + nombre) y conexiones anidadas

- [x] 3.2 Acción agregar carpeta (icono carpeta); crear conexión en contexto de carpeta

- [x] 3.3 Rename inline carpeta (Enter/Escape); wiring a `update_folder`

- [x] 3.4 Confirmar delete carpeta mostrando impacto; editar/eliminar/conectar conexión como hoy adaptado al árbol



## 4. Review and Update Existing Unit Tests (MANDATORY)



- [x] 4.1 Ajustar tests CRUD de perfiles afectados por `folder_id` / carpetas



## 5. Run Unit Tests and Verify State (MANDATORY)



- [x] 5.1 Ejecutar `cargo test` (+ build frontend si aplica)

- [x] 5.2 Report `openspec/changes/manager-profiles/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md` (baseline/restore DB si se muta DB de usuario en smoke)



## 6. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)



- [x] 6.1 Verificar commands de folders/conexiones (harness o runtime) (AGENT MUST EXECUTE)

- [x] 6.2 Report `openspec/changes/manager-profiles/reports/YYYY-MM-DD-step-desktop-commands-verification.md`



## 7. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)



- [x] 7.1 Validar árbol, inline rename, add folder, create connection under folder (AGENT MUST EXECUTE)

- [x] 7.2 Report `openspec/changes/manager-profiles/reports/YYYY-MM-DD-step-desktop-ui-verification.md`



## 8. Update Technical Documentation (MANDATORY)



- [x] 8.1 Actualizar `docs/design/ui-layout-contract.md` (árbol de conexiones / carpetas)

- [x] 8.2 Actualizar README / DESIGN si hay copy o patrones de inline edit / icono add-folder

## 9. Fix UI: tarjeta de conexión compacta

- [x] 9.1 Quitar label `SSH (Contraseña|Llave|túnel)` de la tarjeta; reducir padding/altura CSS
- [x] 9.2 Aplicar color cyan (`--color-cyan-electric`) a la línea `user@host:port`
- [x] 9.3 Icono Lucide Copy al final de esa línea; copiar `user@host` al clipboard (stopPropagation)
- [x] 9.4 Abrir sesión SSH solo con doble clic en la tarjeta (no con click simple); editar/eliminar sin cambios de intención
- [x] 9.5 Actualizar ui-layout-contract / DESIGN si hace falta; re-verificar build frontend

## 10. Fix UI: scrollbars Cyber-Sakura globales

- [x] 10.1 Generalizar estilos scrollbar del template (`.modal-content`) a todos los scrolls de la app (webkit)
- [x] 10.2 Añadir `scrollbar-width` / `scrollbar-color` (Firefox) alineados a tokens sakura
- [x] 10.3 Documentar en SSOT `docs/design/DESIGN.md` la regla: scrollbars temáticos globales, sin track OS blanco
- [x] 10.4 Re-verificar build frontend / que files-tree y sidebar usen el estilo

## 11. Fix UI: collapse click en fila de carpeta

- [x] 11.1 Click en toda `.folder-row` alterna expand/collapse (no solo el chevron)
- [x] 11.2 Garantizar que `+` y basurero reciban el click (`stopPropagation` + stacking/`pointer-events` si hace falta)
- [x] 11.3 Verificar que rename inline / input no dispare toggle; build frontend OK

