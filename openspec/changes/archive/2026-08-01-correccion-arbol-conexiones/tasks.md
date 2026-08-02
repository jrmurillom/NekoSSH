**Surface types:** desktop-ui

## 0. Setup

- [x] 0.1 Confirmar rama `feature/correccion-arbol-conexiones` (o crearla desde la base acordada) antes de tocar código de app

## 1. Header de zona Connections

- [x] 1.1 Reemplazar `.panel-actions--split` en `#panel-servers` por header de zona (label **Connections** + icon-button crear conexión + icon-button crear carpeta)
- [x] 1.2 Estilos del header (fila label izquierda / iconos derecha) con tokens existentes; sin CTA texto “Nueva conexión”
- [x] 1.3 Cablear iconos Lucide + listeners a los mismos flujos actuales de nueva conexión y `createNewFolder` (ids renombrados solo si hace falta)

## 2. Carpeta plana / cajita solo en hijos

- [x] 2.1 Quitar chrome de caja en `.folder-row` (borde/radius que forme rectángulo en idle/hover/activo); hover/activo solo tint plano si aplica
- [x] 2.2 Verificar que `.profile-item` conserva cajita (fondo/borde/radius) y contraste visual carpeta vs hijo
- [x] 2.3 Confirmar que `+` por carpeta, expand/collapse y menús contextuales siguen iguales

## 3. Tests y verificación

- [x] 3.1 Revisar/ajustar unit tests del área tocada (si existen bindings de ids/DOM); si no hay cobertura, documentar N/A en el report N+1
- [x] 3.2 Ejecutar unit tests del área + estado de datos (N/A DB si no hay mutación) y escribir `reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md`
- [x] 3.3 Validación desktop-ui (agente): header Connections + iconos; sin toolbar split; carpeta sin caja; hijos con cajita; crear conexión/carpeta; `+` por carpeta; report `reports/YYYY-MM-DD-step-desktop-ui-verification.md`

## 4. Documentación

- [x] 4.1 Actualizar `docs/design/ui-layout-contract.md` (§ árbol: header de zona + iconos; carpeta plana sin caja)
- [x] 4.2 Actualizar `DESIGN.md` solo si el header introduce un patrón de componente no documentado

## 5. Fix: label UI en español latino (opsx:fix)

- [x] 5.1 Cambiar el label visible del header de zona de `Connections` → **Conexiones** en `app/index.html` (clases CSS en inglés se mantienen)
- [x] 5.2 Actualizar copy en `docs/design/ui-layout-contract.md` y `DESIGN.md` (Connections → Conexiones)
- [x] 5.3 Re-verificar desktop-ui: label visible = Conexiones; no aparece “Connections”; report breve en `reports/` (addendum o nuevo)

## 6. Fix: Snippets con fill primario del tema (opsx:fix — solo color, captura = referencia)

- [x] 6.1 Aplicar a `.sidebar-footer .snippets-footer-btn` el fill/gradiente y texto de `.btn-primary` (tokens `--color-sakura-neon`); sin copiar layout ni copy de la captura
- [x] 6.2 Confirmar engrane sin fill primario; Snippets sigue abriendo el modal
- [x] 6.3 Actualizar `DESIGN.md` / `ui-layout-contract.md` si documentan el look del botón Snippets
- [x] 6.4 Re-verificar desktop-ui (Snippets = fill primario; header Conexiones intacto); evidenciar en report
