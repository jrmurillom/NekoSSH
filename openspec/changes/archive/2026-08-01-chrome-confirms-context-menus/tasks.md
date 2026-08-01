**Surface types:** desktop-ui, desktop-commands



## 0. Setup: Feature Branch (MANDATORY - FIRST STEP)



- [x] 0.1 Crear y cambiar a la rama `feature/chrome-confirms-context-menus`

- [x] 0.2 Verificar rama actual y working tree listo para cambios en `app/`



## 1. Helpers de overlays (SSOT A1 / B3)



- [x] 1.1 Implementar dialog glass centrado (`confirmDialog` / aviso) según `DESIGN.md` § Confirmaciones

- [x] 1.2 Implementar menú contextual B3 (Lucide + hover sakura, Escape/click-fuera)

- [x] 1.3 Estilos CSS compartidos; sin glow; danger row separada



## 2. Migrar confirms/alerts nativos



- [x] 2.1 Reemplazar `confirm` de eliminar conexión / carpeta / cerrar todas por dialog A1

- [x] 2.2 Reemplazar `alert` de errores de producto por dialog/aviso A1 de un botón



## 3. Árbol: carpetas



- [x] 3.1 Quitar basurero de la fila; mantener `+`

- [x] 3.2 Menú contextual carpeta: Renombrar (inline), Eliminar (confirm A1)

- [x] 3.3 Quitar rename por doble clic; iniciar inline solo desde el menú



## 4. Árbol: conexiones



- [x] 4.1 Quitar lápiz y basurero; conservar copy `user@host`

- [x] 4.2 Menú contextual: Editar (modal), Renombrar (inline), Eliminar (confirm A1); doble clic sigue conectando

- [x] 4.3 Implementar rename inline de nombre de conexión (Enter/Escape/blur)



## 5. Review and Update Existing Unit Tests (MANDATORY)



- [x] 5.1 Ajustar tests solo si hay lógica Rust nueva; si es frontend, documentar N/A en report



## 6. Run Unit Tests and Verify State (MANDATORY)



- [x] 6.1 Ejecutar `npm run build` (+ `cargo test` si aplica)

- [x] 6.2 Report `openspec/changes/chrome-confirms-context-menus/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md`



## 7. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)



- [x] 7.1 Verificar que CRUD folders/profiles sigue usando los mismos commands (AGENT MUST EXECUTE)

- [x] 7.2 Report `openspec/changes/chrome-confirms-context-menus/reports/YYYY-MM-DD-step-desktop-commands-verification.md`



## 8. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)



- [x] 8.1 Validar dialog A1, menús B3, fila carpeta/conexión (build + inspección o runtime) (AGENT MUST EXECUTE)

- [x] 8.2 Report `openspec/changes/chrome-confirms-context-menus/reports/YYYY-MM-DD-step-desktop-ui-verification.md`



## 9. Update Technical Documentation (MANDATORY)



- [x] 9.1 Ajustar `ui-layout-contract.md` si el árbol cambió respecto al texto actual (basurero/lápiz)

- [x] 9.2 Verificar `DESIGN.md` § overlays sigue siendo la SSOT (sin contradicciones)



## 10. Fix: hover B3 = rosa exacto de “Nueva conexión”



- [x] 10.1 Inspeccionar CSS del botón “Nueva conexión” (`#btn-new-profile` / `.btn-primary`) y documentar token/clase de hover (rosa real)

- [x] 10.2 Alinear `.chrome-context-item:hover` (y `:focus-visible`) a ese mismo token/color; actualizar `DESIGN.md` § Menús contextuales, `preview-overlays.html` (B3) y `ui-layout-contract.md` si contradicen

- [x] 10.3 Verificar visualmente / `npm run build`

- [x] 10.4 Report corto `openspec/changes/chrome-confirms-context-menus/reports/YYYY-MM-DD-step-desktop-ui-verification.md` (nota de fix hover)

