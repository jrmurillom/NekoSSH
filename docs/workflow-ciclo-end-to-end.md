---
description: Ciclo end-to-end ZenTao → OpenSpec → commit → archive (comandos opsx)
alwaysApply: false
---

# Ciclo de trabajo end-to-end

Flujo oficial del equipo. Comandos canónicos: **`/opsx:*`** (skills en `ai-specs/skills/`).  
No usar nombres heredados de Specboot (`/ff`, `/verify`) salvo que el skill exista con ese alias.

Guía corta para el primer día: `docs/faq/mi-primer-tarea.md`.

---

## Diagrama

```
ZenTao (task asignada)
        │
        ▼
 init-task / enrich-us     → intención + criterios (negocio)
        │
        ▼
 /opsx:propose             → proposal, design, specs delta, tasks
        │
        ▼
 /opsx:apply               → implementación task a task (TDD)
        │
 /opsx:fix (si hace falta) → replan sin borrar progreso
        │
        ▼
 commit (+ PR)             → skill commit / convención task-<id>
        │
        ▼
 /opsx:archive             → sync a openspec/specs + docs permanentes
```

---

## Pasos

### 0. (Opcional) Worktree aislado

Usar el skill `using-git-worktrees` si el trabajo debe aislarse del workspace principal.

### 1. Contexto de negocio

```text
init-task <task-id>
```

o

```text
enrich-us <task-id>
```

- ZenTao = qué debe quedar hecho (criterios, alcance).
- No pegar stack, rutas ni paths en ZenTao.
- Una sola carga de contexto por task salvo re-enrich explícito.

### 2. Plan técnico

```text
/opsx:propose task-<id>-<nombre-corto>
```

Equivale a generar todos los artefactos del change (antes se llamaba “ff” en plantillas viejas).

Revisar `design.md` y `tasks.md` antes de aplicar. Si el plan no convence → ajustar en chat o `/opsx:fix`.

### 3. Implementación

```text
/opsx:apply task-<id>-<nombre-corto>
```

- El agente sigue `tasks.md`, TDD y pasos mandatorios por **tipo de superficie** (`openspec-tasks-mandatory-steps.md`).
- No marcar tasks hechas sin ejecutar las verificaciones que correspondan.

### 4. Parada de emergencia

```text
/opsx:fix "qué falló y cómo replanificar"
```

Actualiza design/tasks sin tirar el progreso ya marcado.

### 5. Commit / PR

```text
commit task-<id>
```

Prefijos: ver `docs/zentao-integration.md` y `docs/base-standards.md`.

### 6. Archivar

```text
/opsx:archive task-<id>-<nombre-corto>
```

Consolida deltas en `openspec/specs/` y actualiza docs permanentes si el change cambió verdad de producto.

---

## Capas SSOT (recordatorio)

| Momento | Gana |
|---------|------|
| Criterios de aceptación de la task | ZenTao |
| Cómo se implementa el producto | `docs/` |
| Cómo se entrega este change | `openspec/changes/<name>/` |
| Look vs layout | `docs/design/DESIGN.md` vs `ui-layout-contract.md` |

Detalle: `docs/base-standards.md` §3 y `docs/zentao-integration.md`.

---

## Alias / nombres obsoletos

| Viejo (Specboot / docs externas) | Usar aquí |
|----------------------------------|-----------|
| `/ff` | `/opsx:propose` |
| `/apply` | `/opsx:apply` |
| `/verify` | verificación dentro de apply + reports en el change |
| `/archive` | `/opsx:archive` |
| `/continue` | `/opsx:apply` (continuar) o `/opsx:fix` |
