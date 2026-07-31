# ZenTao Integration

> **Onboarding dev:** paso a paso en [faq/mi-primer-tarea.md](./faq/mi-primer-tarea.md) (clone, credenciales, comandos).
This project uses **ZenTao** (禅道) with **@tytt/zentao-mcp** for MCP and REST API v1 for **tasks**.

## Workflow: task in the box

~**80% tasks**, ~**20% stories**. Development usually starts from an assigned **task**, not a story.

```
1. Task assigned in ZenTao (execution / sprint)
2. enrich-us 456              ← task id (default)
3. /opsx:propose task-456-...  ← OpenSpec plan
4. /opsx:apply
5. commit task-456
6. /opsx:archive
```

For a **story** (20%): `enrich-us story-12`

## SSOT layers (sources of truth)

ZenTao and `docs/` are **linked, not duplicated**. Each layer owns a different kind of truth.

| Capa | Ubicación | Rol | Idioma |
|-------|----------|------|--------|
| **Producto (técnico)** | `docs/` | SSOT permanente: stack, modelo, rutas | Español latino (+ identificadores en inglés) |
| **Diseño visual** | `docs/design/DESIGN.md` | SSOT de estilo: colores, tipografía, componentes visuales | Español latino (+ tokens técnicos) |
| **Layout de UI** | `docs/design/ui-layout-contract.md` | SSOT estructural: shell, zonas, patrones de página | Español latino |
| **Iteración (planeación)** | Task ZenTao | SSOT de *esta task*: intención, alcance, criterios | Español latino |
| **Plan de implementación** | `openspec/changes/` | Cómo entregar *este change* | Español latino |

### How they connect

```
ZenTao (WHAT = done for this task)
    ↓ enrich-us reads docs/ to align criteria (no copy-paste of specs)
    ↓ /opsx:propose reads docs/ for technical plan
docs/ (HOW the product works — permanent)
    ↓ /opsx:apply implements
openspec/changes/ (plan for this iteration)
```

### Conflict resolution

| Situation | Wins |
|-----------|------|
| Task scope vs permanent product rules | **`docs/`** + `openspec/config.yaml` for technical limits |
| Acceptance criteria vs route names | **ZenTao** for user-visible behaviour; **`docs/api-spec.yml`** for HTTP |
| Look & feel vs layout structure | **`docs/design/DESIGN.md`** for visual tokens; **`ui-layout-contract.md`** for page structure |
| Task asks for something not in `docs/` | Update **`docs/`** if it becomes product truth; otherwise mark **out of scope** in ZenTao |

**Rule:** ZenTao owns **what PO agreed** for the task. `docs/` owns **how the codebase implements the product**. Do not put routes, stack, or file paths in ZenTao — that creates a second technical SSOT and drift.

## Cursor MCP setup

Edit `%USERPROFILE%\.cursor\mcp.json`:

```json
{
  "mcpServers": {
    "zentao": {
      "command": "npx",
      "args": ["-y", "@tytt/zentao-mcp"],
      "env": {
        "ZENTAO_URL": "http://localhost/zentao/www",
        "ZENTAO_ACCOUNT": "your_username",
        "ZENTAO_PASSWORD": "your_password",
        "ZENTAO_SKIP_SSL": "false"
      }
    }
  }
}
```

**Important:** `ZENTAO_URL` is the site root (`.../zentao/www`), **not** `/api.php/v1`.

Restart Cursor after saving.

### Verify

```
zentao_get_products
enrich-us 456
```

## MCP vs REST API

| Data | How |
|------|-----|
| Products, projects, executions | MCP `@tytt` |
| Stories, bugs, test cases | MCP `@tytt` |
| **Tasks (80%)** | **REST API** (see below) |

### Task API (used by `enrich-us`)

```
POST http://localhost/zentao/www/api.php/v1/tokens
Body: {"account":"...","password":"..."}

GET http://localhost/zentao/www/api.php/v1/tasks/{taskId}
Header: Token: {token}

PUT http://localhost/zentao/www/api.php/v1/tasks/{taskId}
Header: Token: {token}
Body: {"desc": "... enhanced description ..."}
```

Helpers in repo:
- `scripts/zentao-fetch-task.mjs` — read task **by id only** (no execution/name fallback; clear not-found message)
- `scripts/zentao-enrich-push.mjs` — write planning to ZenTao (preferred)
- `scripts/zentao-extract-original.mjs` — extract Definición original on re-run
- `scripts/templates/zentao-planning-enhanced.html` — PO-only HTML template

**Lookup rule:** `enrich-us` requires a numeric task id (or `story-<id>`). If `GET /tasks/{id}` fails, stop — do not list executions or search by name.

### enrich-us — ZenTao vs propose

| Layer | Where | Content |
|-------|--------|---------|
| Planning | **ZenTao** (persisted) | Original + objective, scope in/out, acceptance criteria (Spanish, no tech) |
| Technical | **Chat + `/opsx:propose`** | Routes, files, tests, design — from `docs/` |

- **Repo** (`docs/`, OpenSpec, commits): **español latino** — ver `docs/base-standards.md`. Identificadores de código en inglés.
- Task `desc`: **Definición original** (verbatim) + **Enriquecimiento** (planning only; re-run replaces enrich block).

## ZenTao ↔ git traceability

| Artifact | Task (default) | Story (20%) |
|----------|----------------|-------------|
| Branch | `feature/task-<id>-<name>` | `feature/story-<id>-<name>` |
| Commit | `task-<id>: feat(scope): summary` | `story-<id>: ...` |
| PR title | `[task-<id>] summary` | `[story-<id>] summary` |

## Story vs Task vs Bug

| Type | ZenTao | When |
|------|--------|------|
| **Task (任务)** | Execution work item | **Default** — `enrich-us <taskId>` |
| **Story (需求)** | Product requirement | Epics / refinement — `enrich-us story-<id>` |
| **Bug** | Defect | MCP bug tools; separate change |

## Security

- Never commit passwords to git.
- Keep credentials in user-level `mcp.json` only.

## Requirements

- ZenTao **12.x+** with REST API v1 enabled
- Local example URL: `http://localhost/zentao/www`
