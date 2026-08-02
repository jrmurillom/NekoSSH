**Surface types:** desktop-ui

## 0. Setup: Feature Branch (MANDATORY - FIRST STEP)

- [x] 0.1 Verificar rama `feature/correccion-arbol-conexiones` (creada en propose); crearla/cambiar si hace falta
- [x] 0.2 Confirmar working tree listo para cambios en `app/` (sin tocar footer/snippets)

## 1. Inventario y alcance CSS

- [x] 1.1 Releer SSOT visual `docs/design/preview-connection-tree-dense.html` (panel «Después») y anotar tokens de densidad (padding, min-height, gap, tipografía, guía)
- [x] 1.2 Grep usos de `.profile-item`, `.folder-row`, `.connection-tree` en `app/` para asegurar overrides scoped
- [x] 1.3 Listar selectores a tocar solo bajo `.connection-tree` (evitar CSS global / modal snippets)

## 2. Restyle árbol denso (CSS + markup mínimo)

- [x] 2.1 Ajustar `.connection-tree` / bloques de carpeta: gap denso, filas de carpeta compactas (hover sutil, sin tarjeta)
- [x] 2.2 Reforzar jerarquía: indent + `border-left` guía en `.folder-children` alineado al preview
- [x] 2.3 Restyle filas de conexión (`.profile-item` bajo `.connection-tree`): reposo sin chrome de tarjeta; hover/selected sakura; tipografía nombre + endpoint cyan
- [x] 2.4 Verificar empty «Sin conexiones», rename inline y botón `+` / copy siguen legibles tras el densificado
- [x] 2.5 Markup mínimo en `main.ts` / HTML solo si el preview lo exige; sin cambios de lógica de negocio

## 3. Gate visual vs preview (antes de cerrar UI)

- [x] 3.1 Comparar árbol runtime con `preview-connection-tree-dense.html` (densidad, indent, guía, tipografía)
- [x] 3.2 Smoke: expand/collapse, crear conexión desde `+`, vacío, copiar host, doble clic conectar, menú contextual — comportamiento intacto
- [x] 3.3 Smoke negativo: abrir modal Snippets / footer — sin regresiones visuales por CSS

## 4. Review and Update Existing Unit Tests (MANDATORY)

- [x] 4.1 Revisar tests TS/Rust afectados por clases/markup del árbol; ajustar solo si fallan
- [x] 4.2 Documentar N/A de tests nuevos si no hay lógica nueva (solo presentación)

## 5. Run Unit Tests and Verify State (MANDATORY)

- [x] 5.1 Ejecutar suite relevante (`npm run build` y/ o tests del área tocada)
- [x] 5.2 Persistencia: N/A — este change no muta SQLite; documentarlo en el report
- [x] 5.3 Report: `openspec/changes/correccion-arbol-conexiones/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md`

## 6. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)

- [x] 6.1 Arrancar app (o harness visual disponible) y validar árbol denso vs preview + escenarios de aceptación de specs
- [x] 6.2 Evidencia (notas/capturas) de densidad, jerarquía, empty, copy, doble clic, menús; y que snippets/footer no regresaron
- [x] 6.3 Report: `openspec/changes/correccion-arbol-conexiones/reports/YYYY-MM-DD-step-desktop-ui-verification.md`

## 7. Update Technical Documentation (MANDATORY)

- [x] 7.1 Actualizar `docs/design/ui-layout-contract.md`: lenguaje «tarjeta de conexión» → fila densa / densidad de lista en el árbol
- [x] 7.2 Revisar `DESIGN.md` solo si un patrón listado lateral contradice el resultado; ajustar o dejar nota mínima
- [x] 7.3 Confirmar que el preview sigue referenciado como gate de este change (no inventar segunda SSOT visual)

## 8. Fix cajitas + edit + pruebas exhaustivas

> Pivot post-apply: restaurar cajitas (over-scope del densificado en 2.3), diagnosticar/reparar Editar si se rompió al quitarlas, y verificar TODO el sistema de conexiones sin falsos positivos. Las tasks `[x]` de §§0–7 se conservan; el apply de esta sección debe **reconciliar CSS con `main` (card styles)** y **verificar estado actual de `styles.css` antes de editar** (puede haber restore parcial en working tree).

- [x] 8.1 Inventario CSS: diff de `.connection-tree .profile-item` (y empty dashed si aplica) vs `main`; anotar qué falta para paridad de cajitas sin aplanar items
- [x] 8.2 Restaurar CSS cajitas bajo `.connection-tree .profile-item` a paridad con `main` (fondo, borde, radius, padding, gap hijos); indent + guía pueden permanecer si no aplanan; solo `styles.css` / empty dashed si aplica — sin Petdex/snippets
- [x] 8.3 Diagnosticar Editar: repro context menu → Editar → `openProfileModal` con datos correctos → guardar; documentar causa (CSS/markup/otro) en report — sin asumir
- [x] 8.4 Fix mínimo de Editar si 8.3 confirma rotura por este change (CSS/markup scoped); si no está roto, documentar evidencia de que funciona
- [x] 8.5 Matriz exhaustiva del sistema conexiones/árbol — ejecutar TODOS los escenarios y registrar pass/fail + evidencia (clic / esperado / actual) en report; **prohibido** marcar done solo con build verde o inventar tests:
  - Expand/collapse carpeta
  - Cajitas visibles (fondo/borde/radius) en items
  - Indent + línea guía padre/hijo
  - Nueva conexión / nueva carpeta
  - Context menu: Editar abre modal con datos correctos
  - Guardar edit persiste y se refleja en lista
  - Renombrar; Eliminar (o skip delete si destructivo — documentar)
  - Doble clic conecta (o documentar comportamiento real)
  - Copiar user@host
  - Empty «Sin conexiones» dashed
  - Snippets/footer no regresionados (smoke)
- [x] 8.6 Ejecutar `npm run build` y pegar output real en el report
- [x] 8.7 Ejecutar tests unitarios existentes que toquen profiles/folders (si existen); pegar output real. Si no hay tests UI del área, decirlo con honestidad — **no inventar verdes**
- [x] 8.8 Report obligatorio en `openspec/changes/correccion-arbol-conexiones/reports/` (cajitas + edit + matriz + build/tests) con evidencia; **ninguna task 8.x se marca `[x]` sin este report**
- [x] 8.9 Si docs permanentes (§7) quedaron contradiciendo cajitas restauradas, ajustar `ui-layout-contract.md` / nota mínima en `DESIGN.md` para no mentir sobre «sin chrome de tarjeta» en items
