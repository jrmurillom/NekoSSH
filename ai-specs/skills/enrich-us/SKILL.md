---
name: enrich-us
description: Enrich ZenTao tasks or stories with PO-level acceptance criteria and scope before OpenSpec propose. Technical planning stays in chat and propose, not in ZenTao.
author: LIDR.co
version: 4.1.0
---
# enrich-us Skill (ZenTao — task in the box)

Use **before** `/opsx:propose` when work starts from ZenTao.

**Role:** Turn a vague task into a **planning-ready ticket** (QUÉ = done).  
**Not:** A technical spec — that belongs in `docs/` + OpenSpec `propose`.

**Default workflow:** ~80% **Task**; ~20% **Story**. Assume **task** unless user says story.

## Two outputs (critical)

| Destino | Contenido | Idioma |
|---------|-----------|--------|
| **ZenTao** (persistir) | Original + enriquecimiento **solo planeación** | Español latino |
| **Chat** (no persistir en ZenTao) | Notas para `propose` | Español latino (+ identificadores técnicos en inglés) |

**Never write routes, files, git branches, OpenSpec commands, stack, or test tooling into ZenTao.**

## Prerequisites

- ZenTao REST API v1 (tasks) + optional MCP `@tytt` (stories/context).
- Read `docs/` internally to **inform** acceptance criteria — do not copy technical specs into ZenTao.

## Arguments

`$ARGUMENTS` — **id obligatorio** y flags opcionales:
- Task (default): id numérico o `task-<id>` (ej. `6`, `task-6`)
- Story: `story-<id>` o el usuario dice explícitamente story
- Flag opcional: `--pm-mode` (indica que la ejecución viene desde el bot/panel para un PM).

**No** resolver por nombre, keywords, sprint ni idea en texto. Si falta el id, pedir solo el id y detener.

## Step 1 — Resolve work item type

| Signal | Type |
|--------|------|
| `story-` prefix or user says story | Story |
| Numeric id, `task-` prefix | **Task** (default) |

## Step 2 — Fetch from ZenTao

**Tasks:** REST API — `POST .../tokens`, luego **solo** `GET .../tasks/{id}` vía `node scripts/zentao-fetch-task.mjs <taskId>`.  
**Sin fallback:** no listar ejecuciones, no buscar por nombre, no reintentar con otras fuentes.  
**Si no existe:** detener y mostrar el mensaje del script (o equivalente):

> No se ha encontrado la task {id}. Asegúrate de que el id exista en ZenTao y de que las credenciales en ~/.cursor/mcp.json sean correctas.

**Stories:** MCP `zentao_get_story` por id. Si no existe, detener con mensaje equivalente para la story.  
Credentials from `~/.cursor/mcp.json` — never log passwords.

When reading task `desc`, extract only the **Definición original** block if enrich-us ran before (strip `data-section="enrich-us"`). Never overwrite original text.

## Step 3 — Evaluate (planning lens)

Ask: *Can PO and dev agree on "done" without opening the codebase?*

| Check | ZenTao if missing |
|-------|-------------------|
| Objetivo claro en lenguaje negocio | Sí |
| Dentro / fuera de alcance | Sí |
| Criterios de aceptación verificables (usuario/PO) | Sí |
| Preguntas abiertas o supuestos | Sí, si aplica |
| Rutas, archivos, tests, stack | **No — solo chat → propose** |

Use `docs/` privately to **detect gaps** (e.g. original says "nombre" but model uses `title`) — reflect gaps as **business criteria** or **preguntas abiertas**, not as HTTP routes.

## Step 4 — Produce output

### A) ZenTao — Enriquecimiento (persist)

Spanish. **Only these sections:**

```markdown
### Objetivo
Una o dos frases — valor para el usuario.

### Dentro del alcance
- …

### Fuera del alcance
- …

### Criterios de aceptación
1. Dado / Cuando / Entonces — o bullets verificables sin jerga técnica.
2. …

### Supuestos y preguntas abiertas (si aplica)
- …

```

**Forbidden in ZenTao:** `GET /…`, file paths, `feature/task-…`, `npm test`, SQLite, EJS, `/opsx:…`, branch/commit conventions.

**Acceptable:** domain words the PO uses ("post-it", "nombre", "descripción", "tablero").

### B) Chat — Notas para propose (no persistir en ZenTao)

**Solo si NO se especificó el flag `--pm-mode`:**
En **español latino**. Referencias técnicas con nombres en inglés (`title`, rutas, archivos).
Si se especificó `--pm-mode`, omite esta sección por completo del chat.

### C) Chat — Definición original

Show verbatim original for user confirmation.

## Step 5 — Write back to ZenTao (required)

El script `zentao-enrich-push.mjs` envuelve automáticamente el enriquecimiento con los marcadores robustos `[ENRICH-US-START]` y `[ENRICH-US-END]`. Todo lo que esté fuera de estos marcadores se considera la Definición original intacta.

Two blocks in HTML:

1. Texto original íntegro (fuera de los marcadores).
2. Bloque delimitado por los marcadores — **section A only** (planning).

```bash
node scripts/zentao-enrich-push.mjs <taskId> \
  --enhanced-file /tmp/enhanced-planning.html \
  --original-file /tmp/original.html

# Re-run (keeps Definición original from ZenTao):
node scripts/zentao-enrich-push.mjs <taskId> \
  --enhanced-file /tmp/enhanced-planning.html \
  --use-task-original
```

Template for planning HTML: `scripts/templates/zentao-planning-enhanced.html`.

## Step 6 — Hand off to OpenSpec

**Solo si NO se especificó el flag `--pm-mode`:**
Tell the user:

```
/opsx:propose task-<id>-<short-name>
```

`propose` reads `docs/` + enriched **business intent** from ZenTao (via context you used). Technical plan is created there — not in ZenTao.

**Si se especificó el flag `--pm-mode`:**
Omitir por completo este paso y no imprimir ningún comando de desarrollo ni de consola en la respuesta del chat.

## Re-run rules

| Change | Action |
|--------|--------|
| PO edits original in ZenTao | Re-run enrich-us; keep new original verbatim |
| Business scope / acceptance change | Re-run enrich-us |
| Technical detail (routes, files) | Edit in `propose` / `docs/` — **not** ZenTao |

## MCP reference (`@tytt`)

Products, projects, executions, stories, bugs — **not tasks** (use REST).
