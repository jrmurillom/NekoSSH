**Surface types:** desktop-ui

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear y cambiar a la rama `feature/refactor-deuda-tecnica` antes de modificar el código de la app.
- [x] 0.2 Verificar que la rama activa en Git sea `feature/refactor-deuda-tecnica`.

## 1. Corrección de Hallazgos Críticos en Rust (RUST-CRIT-1 & RUST-CRIT-2)

- [x] 1.1 En [app/src-tauri/src/lib.rs](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/lib.rs), corregir la advertencia de visibilidad pública en `SshConnections` declarando `pub struct LiveSsh` (**RUST-CRIT-2**).
- [x] 1.2 En [app/src-tauri/src/external_edit.rs](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/external_edit.rs), [app/src-tauri/src/lib.rs](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/lib.rs) y [app/src-tauri/src/edit_session.rs](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/edit_session.rs), implementar acceso seguro a Mutexes evitando `.lock().unwrap()` expuesto a lock poisoning (**RUST-CRIT-1**).

## 2. Modularización de Monolitos (ARCH-HIGH-1 & ARCH-HIGH-2)

- [x] 2.1 En [app/src/modules/connection-tree-helper.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/modules/connection-tree-helper.ts), implementar la lógica pura de agrupación y ordenamiento de carpetas y perfiles para reducir el tamaño de `main.ts` (**ARCH-HIGH-1**).
- [x] 2.2 Crear [app/src/modules/connection-tree-helper.test.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/modules/connection-tree-helper.test.ts) con pruebas unitarias en Vitest para validación de jerarquía, orden por sort_order y carpetas huérfanas.
- [x] 2.3 En [app/src/modules/sftp-path-helper.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/modules/sftp-path-helper.ts), implementar helper puro de navegación SFTP para desacoplar `main.ts` (**ARCH-HIGH-1**).
- [x] 2.4 Crear [app/src/modules/sftp-path-helper.test.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/modules/sftp-path-helper.test.ts) con pruebas unitarias para navegación de padre (`..`), sanitización de barras y rutas con espacios.

## 3. Integración, Pruebas y Verificación Final (Rust & Vite)

- [x] 3.1 Integrar los nuevos submódulos en [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts).
- [x] 3.2 Ejecutar suite de pruebas de Rust con `cargo test` en `app/src-tauri`.
- [x] 3.3 Ejecutar suite de pruebas de Vitest con `npm run test` en `app`.
- [x] 3.4 Compilar el frontend (`npm run build` en `app`).
- [x] 3.5 Compilar y verificar el backend Rust (`cargo check` en `app/src-tauri`).
- [x] 3.6 Generar el reporte de validación final en `openspec/changes/refactor-deuda-tecnica/reports/2026-08-01-step-build-verification.md`.

## 4. Corrección de Focus y Parpadeo de Terminal (Fix)

- [x] 4.1 En [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), remover los event listeners manuales de `click` y `mousedown` que competían con el foco nativo de `xterm.js`.
- [x] 4.2 En [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css), remover la animación `@keyframes cyber-cursor-blink` forzada que invertía el parpadeo del cursor.
- [x] 4.3 Ejecutar la app en vivo con `tauri dev` y realizar una prueba real de conexión SSH hacia el host `161.97.142.252` registrado en la base de datos local para verificar el foco interactivo completo.
