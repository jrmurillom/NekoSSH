# Desktop UI Verification Report

- Date: 2026-07-30
- Change: fase1
- Agent: Cursor Auto (opsx-apply)
- Surface: desktop-ui

## Environment
- `npx tauri dev` — Vite en `http://localhost:1420/` + `app.exe` en ejecución
- Verificación de shell también vía browser sobre el frontend Vite (IPC Tauri no disponible en browser)

## Scenarios

### 1. Shell Cyber-Sakura carga
- **Resultado:** PASS
- Sidebar (Servidores / Archivos), CTA “+ Nuevo Perfil”, empty state del workspace y controles de fondo visibles.
- Evidencia: screenshot browser + snapshot accessibility.

### 2. Token `--font-mono` resuelto en runtime
- **Resultado:** PASS
- `getComputedStyle(document.documentElement).getPropertyValue('--font-mono')` → `'Fira Code', 'JetBrains Mono', monospace`
- Métricas: anchos de cadenas `WWW…` e `iii…` iguales (`looksMonospace: true`) → familia monoespaciada efectiva.
- Código: `app/src/main.ts` usa ese valor al instanciar `Terminal` (no `var(--font-mono)` estático).
- Script: `node scripts/verify-font-mono.mjs` → PASS

### 3. Crear perfil de prueba
- **Resultado:** PASS (capa datos) / N/A interactivo en browser
- CRUD de perfiles verificado con unit tests SQLite (`cargo test`, 3/3).
- En browser puro, `invoke` Tauri no está disponible; el formulario UI está presente (modal Nuevo Perfil).

### 4. Terminal + ASCII art vía SSH live
- **Resultado:** NO EJECUTADO
- Requiere host SSH real y automatización del WebView nativo. No bloquea tipografía (cubierta en §2) ni CRUD (§3).

## Commands
- `npx tauri dev` (app + Vite)
- `node scripts/verify-font-mono.mjs`
- `npm run build` (tsc + vite) — PASS previo
- Browser CDP `Runtime.evaluate` (métricas mono)

## Outcome
- Step desktop-ui status: PASS (con limitación documentada en §4)
- Blocking issues: ninguno para tipografía/shell; SSH live pendiente de entorno con host
