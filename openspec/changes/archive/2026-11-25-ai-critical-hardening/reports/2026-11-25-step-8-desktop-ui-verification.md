# Verificación Step 8 — Desktop UI (`ai-critical-hardening`)

- **Fecha:** 2026-11-25
- **Cambio OpenSpec:** `ai-critical-hardening`

## Evidencia verificada

- **Build servido:** `app/dist/ai-copilot.html` tras el último ajuste TypeScript (`vite build` en `3.24s`).
- **Inspección Edge Headless (`--dump-dom`):**
  - `DOM_LEN=12488`
  - `HAS_HEADER=True`
  - `HAS_DOCK=True`
  - `HAS_HERO=True`
  - `HAS_SVG=True`
- **Aislamiento de tarjetas por `terminal_id` (Vitest):** `copilot-state.test.ts` verifica que `getOrCreateSessionUiState` mantiene `pendingAction`, `lastFailedContext`, `lastRollback` y `rollbackDraftOpen` independientes entre `term-1` y `term-2`.
