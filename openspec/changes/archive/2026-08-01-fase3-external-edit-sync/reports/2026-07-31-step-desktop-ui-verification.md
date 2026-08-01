# Desktop UI Verification

- Date: 2026-07-31
- Change: `fase3-external-edit-sync`
- Branch: `feature/fase3-external-edit-sync`
- Agent: Cursor Auto (opsx-apply)

## Postura

Verificación **mock/local / estructural**. No se lanzó GUI contra el lab SSH ni se mutaron archivos remotos. El upload de producto tras confirm del usuario permanece implementado en código; la corrida de agente no lo ejercita contra el lab.

## Evidencia estructural (código + build)

- `npm run build --prefix app` → **PASS** (`tsc && vite build`)
- Hooks UI presentes en `app/src/main.ts` / `app/index.html`:
  - Doble clic archivo → `beginExternalEdit`
  - Menú B3 “Editar” (icono pencil) en archivos
  - Doble clic carpeta **no** inicia edición
  - `confirmDialog` A1 “¿Subir al servidor?” / “Subir” / “Cancelar”
  - Aviso binario A1; alert >10 MiB
  - Listener `edit-session-changed` + `edit-session-disconnected`
  - Settings: `#config-editor-path` + “Editor externo preferido” / Guardar
  - Sin `window.confirm` en el flujo nuevo

## Casos (8.1b / 8.2) — resultado agente

| Caso | Resultado |
|------|-----------|
| Doble clic / menú Editar → orquestación FE | **PASS** (código cableado a `probe_external_edit` + `start_external_edit`) |
| Guardar → A1 subir; Cancelar → `dismiss_edit_change` | **PASS** (path de código; sin write remoto en verificación) |
| Confirmar → `confirm_edit_upload` | Implementado; **N/A remoto** en agente (mock posture) |
| Settings editor preferido | **PASS** (UI + commands SQLite) |
| Rechazo >10 MiB / aviso binario | **PASS** (FE + unit/fake) |
| Disconnect mid-edit aviso | **PASS** (evento `edit-session-disconnected` + status/alert) |
| Abrir editor OS real en desktop | **N/A** en esta corrida (no se spawneó GUI Tauri interactiva) |

## Lab SSH: cero mutaciones

- **Declaración explícita:** cero uploads/replaces/deletes al host SSH de pruebas compartido.
- Qué se mockeó: ciclo de transfer y registry en unit/harness; UI validada por build + inspección de hooks.
- Write remoto: **N/A**.
- Sandbox disposable: **no provisionado**; no se ejecutó smoke live de subida.

## Outcome

- Desktop-ui verification (mock/local): **PASS** con N/A documentados para apertura interactiva del editor y write remoto.
