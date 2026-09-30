# Reporte de Verificación Paso 5: Pruebas Unitarias y Estado Local

- **Fecha:** 2026-09-29
- **Cambio:** `sftp-recursive-folder-upload`
- **Rama:** `feature/sftp-recursive-folder-upload`

---

## 1. Resumen Ejecutivo

Se ejecutó la suite completa de pruebas unitarias en Frontend (TypeScript/Vitest) y Backend (Rust/Cargo), validando el escaneo recursivo local, la creación idempotente de rutas remotas en SFTP y los helpers de formateo y filtrado del diálogo de confirmación.

- **Frontend Tests:** 109 pasados / 109 totales (14 archivos de test).
- **Backend Tests:** 75 pasados / 75 totales (0 fallas, 0 ignorados).
- **Regresiones detectadas:** 0.

---

## 2. Detalle de Pruebas Frontend (Vitest)

Comando ejecutado: `npm run test` en `app/`

```text
 ✓ src/modules/sftp-path-helper.test.ts (3 tests)
 ✓ src/modules/remote-history-helper.test.ts (4 tests)
 ✓ src/modules/connection-tree-helper.test.ts (2 tests)
 ✓ src/modules/brand-logo-helper.test.ts (5 tests)
 ✓ src/strip-trailing-paste.test.ts (6 tests)
 ✓ src/modules/shell-grid-helper.test.ts (8 tests)
 ✓ src/modules/explorer-name-filter.test.ts (11 tests)
 ✓ src/modules/transfer-progress-helper.test.ts (6 tests)
 ✓ src/modules/transfer-progress-e2e.test.ts (11 tests)
 ✓ src/bg-settings-helper.test.ts (14 tests)
 ✓ src/modules/terminal-tab-menu-helper.test.ts (5 tests)
 ✓ src/modules/explorer-drop-helper.test.ts (19 tests)
 ✓ src/modules/theme-wallpaper-helper.test.ts (8 tests)
 ✓ src/modules/terminal-options-helper.test.ts (7 tests)

 Test Files  14 passed (14)
      Tests  109 passed (109)
```

Nuevas pruebas añadidas en `src/modules/explorer-drop-helper.test.ts`:
1. `formatFileSize`: manejo de 0 B, valores negativos, KB, MB y GB.
2. `formatUploadPlanImpact`:
   - Carpeta individual: `"Carpeta 'techpeople' (3 carpetas, 125 archivos · 4.2 MB) → /var/www"`.
   - Archivo individual: `"index.html (2.0 KB) → /var/www"`.
   - Selección mixta: `"2 carpetas, 50 archivos (1.0 MB) → /home/neko"`.
   - Lote de archivos: `"5 archivos (5.0 KB) → /home/neko"`.
3. `filterPlanByExcludedRoots`: exclusión en cascada de carpetas y archivos anidados cuando el usuario omite la sustitución en una colisión.

---

## 3. Detalle de Pruebas Backend (Rust)

Comando ejecutado: `cargo test` en `app/src-tauri`

```text
running 75 tests
...
test upload_scan::tests::scan_single_file ... ok
test upload_scan::tests::scan_nested_directory_tree ... ok
test upload_scan::tests::scan_mixed_selection ... ok
test path_util::tests::parent_de_deep_path ... ok
test path_util::tests::parent_de_subdir ... ok
test path_util::tests::join_desde_raiz ... ok
test path_util::tests::join_subdir ... ok
test path_util::tests::shell_quote_simple ... ok
test path_util::tests::shell_quote_con_comilla ... ok
...
test result: ok. 75 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

Nuevas pruebas añadidas:
1. `scan_single_file`: escaneo de archivo individual y cálculo de metadatos.
2. `scan_nested_directory_tree`: escaneo recursivo con orden determinista, normalización de rutas a POSIX `/`, y preservación de jerarquía.
3. `scan_mixed_selection`: soporte para arrastre conjunto de carpetas y archivos independientes.
4. `parent_de_deep_path`: resolución de directorios padre para rutas anidadas profundas (`/var/www/techpeople/assets/css/style.css`).

---

## 4. Conclusión

El estado del código es consistente, no genera alertas ni errores de compilación (`tsc && vite build` y `cargo check` limpios al 100%), certificando la estabilidad para las pruebas de comandos e interfaz de escritorio.
