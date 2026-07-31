---
description: Pasos obligatorios en tasks.md OpenSpec, parametrizados por tipo de superficie
alwaysApply: true
---

# OpenSpec Tasks: pasos mandatorios

Al crear o actualizar `tasks.md` en un change OpenSpec:

1. Leer `openspec/config.yaml`.
2. Declarar el **tipo de superficie** del change (ver §2).
3. Incluir solo los pasos mandatorios de esa superficie (más los globales).
4. El agente **ejecuta** las verificaciones; no las delega al usuario.

---

## 1. Globales (siempre)

### Step 0 — Feature branch (PRIMERO)

- Rama: `feature/task-<id>-<nombre>` o `feature/<change-name>`.
- Crear y cambiar a la rama antes de tocar código.

### Step N — Review / update unit tests (MANDATORY)

- Revisar y ajustar pruebas existentes afectadas.

### Step N+1 — Run unit tests + estado de datos (MANDATORY)

- Ejecutar suite/unitarios del área tocada.
- Si hay persistencia local (p. ej. SQLite): capturar baseline pre/post y restaurar si hubo mutación.
- Report: `openspec/changes/<change>/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md`
  (si no hay DB, el mismo report documenta “N/A — sin persistencia en este change”).

### Step Last — Update technical documentation (MANDATORY)

- Actualizar SSOT tocados (`docs/`, `DESIGN.md`, layout contract, etc.) según `documentation-standards.md`.

---

## 2. Tipos de superficie

Declarar al inicio de `tasks.md`:

```markdown
**Surface types:** desktop-ui | desktop-commands | http-api | web-ui
```

Puede haber más de uno; aplicar la unión de pasos.

| Tipo | Cuándo | Verificación mandatoria extra |
|------|--------|-------------------------------|
| **desktop-ui** | UI de app de escritorio (ventanas, layout, terminal visible) | Validación manual de UI **ejecutada por el agente** (runtime de la app o automatización disponible) + report |
| **desktop-commands** | Commands/events IPC del runtime de escritorio (sin HTTP) | Invocar commands/events de prueba (CLI del runtime, harness o script) + report; **no** exigir curl HTTP |
| **http-api** | Endpoints HTTP REST/JSON | curl (o cliente HTTP) + restore de datos + report |
| **web-ui** | UI en navegador | E2E con browser MCP / Playwright si aplica + report |

### Mapeo rápido

- Cliente escritorio con UI + IPC local → `desktop-ui` + `desktop-commands`.
- Solo backend HTTP → `http-api`.
- SPA en browser → `web-ui` (+ `http-api` si hay API).

---

## 3. Detalle por superficie

### A. http-api — Manual endpoint testing (MANDATORY — AGENT MUST EXECUTE)

1. Servidor arriba.
2. GET/POST/PUT/PATCH/DELETE con curl (o equivalente).
3. Casos de error (validación, 404, auth).
4. Restaurar estado de datos tras mutaciones.
5. Report en `openspec/changes/<change>/reports/`.
6. Marcar complete solo tras ejecución real.

### B. web-ui — E2E browser (MANDATORY if applicable — AGENT MUST EXECUTE)

1. Frontend (+ API si aplica) arriba.
2. Flujos con browser MCP / Playwright.
3. Errores de UI y persistencia.
4. Cleanup de datos.
5. Report en `reports/`.

### C. desktop-commands — IPC / commands (MANDATORY — AGENT MUST EXECUTE)

1. App o harness de test disponible.
2. Invocar commands/events cubiertos por el change (éxito + error).
3. Verificar efectos en persistencia local si aplica; restaurar.
4. Report: `YYYY-MM-DD-step-desktop-commands-verification.md`.
5. **No** sustituir esto por curl a URLs HTTP inventadas.

### D. desktop-ui — Validación de UI (MANDATORY — AGENT MUST EXECUTE)

1. Arrancar la app en el runtime de escritorio (o harness visual disponible).
2. Cubrir escenarios de aceptación visibles (shell, listas, terminal, preferencias tocadas).
3. Documentar pasos, resultado y capturas/notas en report:
   `YYYY-MM-DD-step-desktop-ui-verification.md`.
4. Si no hay automatización: el agente ejecuta el checklist manual y deja evidencia en el report; **no** pedir “pruébalo tú” como cierre de task.

---

## 4. Checklist antes de cerrar `tasks.md`

- [ ] Step 0 es el primero
- [ ] Surface types declarados
- [ ] Solo pasos de las superficies aplicables (+ globales)
- [ ] Steps numerados en orden
- [ ] Labels `(MANDATORY)` / `AGENT MUST EXECUTE` donde toca
- [ ] Paths de reports bajo `openspec/changes/<change>/reports/`
- [ ] Sin exigir curl HTTP en changes solo desktop
- [ ] Sin exigir Playwright browser en changes solo desktop sin UI web

---

## 5. Al aplicar (`/opsx:apply`)

1. Ejecutar verificaciones del surface type — nunca delegar al usuario.
2. Marcar `[x]` solo tras evidencia (comando + report).
3. Documentar comandos, resultados y restore.

---

## 6. Ejemplo (desktop-ui + desktop-commands)

```markdown
**Surface types:** desktop-ui, desktop-commands

## 0. Setup: Create Feature Branch (MANDATORY)
- [ ] 0.1 Crear rama `feature/task-123-profiles`
- [ ] 0.2 Verificar rama actual

## 1. … implementación TDD …

## 8. Review and Update Existing Unit Tests (MANDATORY)
## 9. Run Unit Tests and Verify Local DB (MANDATORY)
## 10. Desktop Commands Verification (MANDATORY - AGENT MUST EXECUTE)
## 11. Desktop UI Verification (MANDATORY - AGENT MUST EXECUTE)
## 12. Update Technical Documentation (MANDATORY)
```

## 7. Ejemplo (http-api + web-ui)

```markdown
**Surface types:** http-api, web-ui

## 0. Feature branch …
## … implementación …
## N. Unit tests + DB (MANDATORY)
## N+1. curl endpoints (MANDATORY - AGENT MUST EXECUTE)
## N+2. E2E browser (MANDATORY - AGENT MUST EXECUTE)
## N+3. Docs (MANDATORY)
```

---

## Failure

Crear `tasks.md` sin surface types o con pasos de API en un change solo desktop obliga a corregir a mano. Leer siempre este doc + `openspec/config.yaml` antes de escribir tasks.
