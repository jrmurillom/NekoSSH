# Desktop UI Verification

- Date: 2026-07-31
- Change: `external-edit-sudo-retry`
- Branch: `feature/external-edit-sudo-retry`
- Agent: Cursor Auto (opsx-apply)

## Postura

Verificación **mock/local / estructural**. No se lanzó GUI contra el lab SSH ni se mutaron archivos remotos. El path elevado de producto permanece en código; la corrida de agente lo ejercita solo vía harness mock.

## Evidencia (código + build)

- `npm run build --prefix app` → **PASS** (`tsc && vite build`)
- Flujo en `app/src/main.ts` (`handleEditSessionChanged`):
  1. Confirm A1 «¿Subir al servidor?» → `confirm_edit_upload` (upload normal primero; sin sudo).
  2. Si error con `elevatable: true` → segundo A1 «Subir con sudo» / Cancelar (`confirmDialog`, `danger: false`).
  3. Cancel / Escape → no invoca `edit_session_upload_with_sudo`; status indica temp conservado.
  4. Aceptar → un `invoke("edit_session_upload_with_sudo")`.
  5. Fallo elevado → `alertDialog` con mensaje (sin UI de password).
  6. Fallo no elevable → `alertDialog` sin oferta sudo.
- Reusa `overlays.ts` (chrome A1); sin `window.confirm` nativo en el flujo.

## Casos (7.1 / 7.2) — resultado agente

| Caso | Resultado |
|------|-----------|
| Fallo elevable → A1 «Subir con sudo» | **PASS** (código cableado a `elevatable`) |
| Cancel no eleva | **PASS** (return sin invoke sudo) |
| Aceptar dispara path elevado | **PASS** (`edit_session_upload_with_sudo`) |
| Error elevado → alert, sin password UI | **PASS** (solo `alertDialog`) |
| Upload normal exitoso no ofrece sudo | **PASS** (sudo solo en `catch` elevable) |
| Fallo no elevable no ofrece sudo | **PASS** (rama `alertDialog` sin segundo confirm) |
| GUI Tauri interactiva end-to-end | **N/A** en esta corrida |

## Lab SSH: cero mutaciones

- **Declaración explícita:** cero uploads/exec/delete al host SSH de pruebas compartido.
- Qué se mockeó / validó: build FE + inspección de hooks; path elevado en `smoke_elevated_upload_local`.
- Write remoto: **N/A**.
- Sandbox disposable: **no provisionado**; no se ejecutó smoke live de sudo.

## Outcome

- Desktop-ui verification (mock/local): **PASS** con N/A para GUI interactiva y writes remotos live.
