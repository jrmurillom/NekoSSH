## Context

Tras `manager-profiles`, el árbol del sidebar (`.connection-tree`) usa filas de carpeta con padding generoso y `.profile-item` con fondo/borde permanentes — se lee como tarjetas, no como lista. El DOM jerárquico (carpeta → hijos) es correcto; el commit de snippets no causó el problema. El contrato (`ui-layout-contract.md`) ya pide densidad de lista. Existe preview estático aprobado como SSOT visual: `docs/design/preview-connection-tree-dense.html` (usuario puede seguir ajustándolo; el apply debe igualar ese preview al cerrar UI).

Stack: CSS/markup Vanilla en `app/`; sin cambios de modelo ni IPC.

## Goals / Non-Goals

**Goals:**

- Filas densas para carpetas y conexiones alineadas al preview (altura, gap, hover, tipografía).
- Jerarquía legible: indent de hijos + `border-left` guía + bloques por carpeta.
- Conservar 100% del comportamiento actual (CRUD UI, expand/collapse, vacío, copy, doble clic, menús, rename).
- CSS preferentemente bajo `.connection-tree` (y descendientes) para no romper modal snippets ni chrome global.
- Actualizar lenguaje del layout contract («tarjeta» → fila densa) donde describa el árbol.

**Non-Goals:**

- Footer, engrane, snippets, modal de snippets.
- Cambios de SQLite, migraciones, commands Rust, IPC.
- Rediseño del explorador SFTP (`files-tree`).
- Nuevas features de árbol (multi-select, drag-and-drop, subcarpetas).

## Decisions

1. **SSOT visual = preview denso**
   - **Decisión:** Gate de aceptación UI = coincidencia visual con `docs/design/preview-connection-tree-dense.html` (panel «Después»), no con el mock «Antes».
   - **Alternativa:** reinterpretar DESIGN.md sin preview → rechazada; el usuario pidió preview-first y lo trata como referencia.
   - **Nota:** si el preview se tweakea antes del apply, re-leer el archivo y alinear; no inventar otra densidad.

2. **CSS scoped, clases existentes**
   - **Decisión:** Ajustar reglas existentes (`.connection-tree`, `.folder-row`, `.folder-children`, `.profile-item`, etc.) anidando o cualificando bajo `.connection-tree` cuando el selector sea ambiguo. Evitar renombrar masivo de clases salvo necesidad del preview.
   - **Alternativa:** nuevas clases `.conn-row` / `.tree-dense` en runtime → solo si el markup actual no puede expresar el preview sin chocarse con estilos legacy; preferir reutilizar.
   - **HARD:** no tocar selectores de snippets/modal ni estilos globales de `.btn-icon` salvo overrides scoped.

3. **Presentación vs interacción**
   - **Decisión:** Solo polish visual/markup. Interacciones (doble clic conectar, menú, `+` con stopPropagation, expand en click de fila, copy `user@host`) permanecen.
   - **En reposo:** borde/fondo transparentes o casi nulos; hover/selected con tint sakura suave (como preview).
   - **Conexión:** min-height ~36px, tipografía nombre ~0.86rem, endpoint mono ~0.68rem cyan.
   - **Carpeta:** min-height ~28px, gap árbol ~1px entre filas.

4. **Docs permanentes en el mismo change**
   - **Decisión:** En tasks de documentación, actualizar `ui-layout-contract.md` § árbol/tarjeta de conexión a «fila densa»; tocar `DESIGN.md` solo si hay patrón listado lateral que contradiga.
   - El preview HTML puede quedar como referencia de diseño (no es SSOT permanente post-archive, pero sí gate de este change).

5. **TDD / pruebas**
   - **Decisión:** Sin lógica de negocio nueva → no tests unitarios Rust nuevos. Si hay tests frontend de selectores/clases, ajustarlos. Verificación principal = desktop-ui + comparación con preview + build frontend.

## Risks / Trade-offs

| Riesgo | Mitigación |
|--------|------------|
| Selectores `.profile-item` / `.folder-row` afectan otra UI | Cualificar bajo `.connection-tree`; grep de usos antes de cambiar |
| Regresión visual en empty state / rename inline | Checklist UI incluye empty «Sin conexiones», rename carpeta/conexión, selected |
| Preview diverge durante el apply | Re-leer preview al implementar; gate explícito en tasks |
| Over-refactor de markup | Diff mínimo; CSS first |

## Migration Plan

1. Feature branch `feature/correccion-arbol-conexiones` (ya creada en propose).
2. Apply: CSS/markup → verificar preview → docs → reports mandatorios.
3. Rollback: revertir commit(s) de estilos; sin migración de datos.

## Open Questions

- Ninguna bloqueante. Tweaks menores del preview siguen permitidos antes/durante apply; el alcance permanece «igualar el preview denso», no reabrir footer/snippets.

### Corrección de Ruta (Fix) — Restaurar cajitas + edit + verificación exhaustiva

**Motivo:** El apply aplanó `.profile-item` (over-scope respecto a lo que el usuario acepta): se quitaron las «cajitas» (fondo, borde, radius, padding) de los items de conexión. El usuario reporta que, al quitar esas cajitas, se rompió **Editar** (menú contextual → modal → guardar). Además, no se aceptan falsos positivos: un `npm run build` verde **no** basta para marcar UI/edit como done.

**Estado del working tree:** Puede haber restauración CSS parcial no autorizada en `app/src/styles.css`. Antes de editar: **reconciliar con los estilos de tarjeta (cajitas) de `main`**; verificar el estado actual del archivo (diff vs `main` / HEAD) y solo aplicar lo que falte para paridad de cajitas bajo `.connection-tree .profile-item`, sin reintroducir aplanado ni mezclar con Petdex/snippets.

**Estrategia:**

1. **Restaurar CSS cajitas** bajo `.connection-tree .profile-item` a paridad con `main` (fondo, borde, radius, padding, gap de hijos). El indent + línea guía del árbol pueden quedarse **si no aplanan** los items (no quitar chrome de tarjeta).
2. **Diagnosticar y reparar Editar** (context menu → `openProfileModal` → guardar) si está roto por CSS/markup de este change; evidenciar la causa en un report (repro + root cause), no asumir.
3. **Verificación exhaustiva** del «punto» conexiones/árbol (no solo build): matriz obligatoria de escenarios con pass/fail y evidencia (qué se clicó, esperado vs actual). Prohibido inventar tests verdes o marcar done sin report.
4. **Prohibido** marcar tasks de UI/edit como `[x]` solo con `npm run build`. Build + tests unitarios existentes del área (si existen) se ejecutan y se pega output real en el report; si no hay tests UI, decirlo con honestidad y exigir matriz manual con evidencia.

**HARD:**
- Una cosa por task; no mezclar Petdex ni snippets en el fix de cajitas/edit (solo smoke de no-regresión snippets/footer).
- CSS scoped bajo `.connection-tree` (y descendientes necesarios).
- Tasks de verificación no se cierran sin report en `reports/` con evidencia.
