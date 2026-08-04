---
description: Principios y reglas núcleo del proyecto (SSOT de desarrollo)
alwaysApply: true
---

# Base Standards (SSOT de desarrollo)

Este documento es la **fuente de verdad** de principios, idioma, TDD y enlaces a estándares específicos. Antes de planificar o implementar, léelo completo.

---

## 1. Principios núcleo

1. **Tareas pequeñas, una a una** — pasos de ~2 h máx.; no saltar adelante.
2. **TDD** — prueba que falla → implementación mínima → refactor.
3. **Tipado estricto** — TypeScript y Rust sin atajos que borren tipos.
4. **Nombres claros** — identificadores descriptivos; sin abreviaturas opacas.
5. **Cambios incrementales** — diffs enfocados y revisables.
6. **Código de aplicación solo en `app/`** — raíz del repo para `docs/`, `openspec/`, agentes y scripts.

---

## 2. Idioma (regla bilingüe)

| Superficie | Idioma |
|------------|--------|
| Docs humanos, artefactos OpenSpec, commits, títulos/descripciones de tests | **Español latino** |
| Identificadores de código (variables, funciones, rutas, SQL, archivos fuente) | **Inglés** |
| UI visible al usuario final | **Español latino** |

No aplicar “English only” a documentación ni a artefactos OpenSpec. Esa regla heredada de Specboot **no aplica** aquí.

---

## 3. Capas SSOT (quién manda)

| Capa | Ubicación | Dueño de |
|------|-----------|----------|
| Producto técnico | `docs/` (este archivo + estándares enlazados) | Stack, alcance, reglas permanentes |
| Look & feel | `docs/design/DESIGN.md` | Tokens, tipografía, componentes visuales |
| Layout de UI | `docs/design/ui-layout-contract.md` | Shell, zonas, patrones de página |
| Iteración (negocio) | Task ZenTao | Intención, alcance, criterios de *esta* task |
| Plan de change | `openspec/changes/<name>/` | Cómo entregar *este* change |
| Specs principales | `openspec/specs/` | Comportamiento consolidado post-archive |

**Regla:** ZenTao no es SSOT técnico (sin rutas, stack ni paths). Conflictos técnicos → `docs/` + `openspec/config.yaml`.

Ciclo de trabajo: `docs/workflow-ciclo-end-to-end.md`.  
Pasos obligatorios en `tasks.md`: `docs/openspec-tasks-mandatory-steps.md`.

---

## 4. Estándares específicos

Leer solo los que apliquen al cambio:

| Área | Documento |
|------|-----------|
| Alcance y fases | `docs/project_scope.md` |
| Frontend | `docs/frontend-standards.md` |
| Diseño visual | `docs/design/DESIGN.md` |
| Layout UI | `docs/design/ui-layout-contract.md` |
| Documentación | `docs/documentation-standards.md` |
| ZenTao | `docs/zentao-integration.md` |

Si falta un estándar (p. ej. backend HTTP clásico), no inventar uno paralelo: documentar decisiones en el `design.md` del change y, si se vuelven permanentes, promoverlas a `docs/`.

---

## 5. OpenSpec y agentes

1. **Obligatoriedad de la CLI de OpenSpec**: Toda acción de propuesta, diseño, especificación, implementación, corrección y archivado de cambios MUST realizarse estrictamente a través de los comandos de la CLI de OpenSpec (`openspec new change`, `openspec status`, `openspec instructions`, `openspec validate`, `openspec archive`, etc.). Queda estrictamente prohibido que el agente actúe de manera independiente o realice modificaciones al código de la aplicación sin seguir el flujo de la CLI, registrar la planificación en los artefactos correspondientes y obtener autorización humana previa.
2. Crear artefactos con `/opsx:propose` (o comandos CLI `openspec` equivalentes).
3. Implementar con `/opsx:apply` siguiendo el checklist de `tasks.md` generado y TDD.
4. Si el plan falla o cambia de rumbo, usar `/opsx:fix` para mutar los artefactos de planificación únicamente (design, proposal, specs, tasks). Queda estrictamente prohibido realizar cambios en el código de la aplicación durante esta fase hasta que el plan modificado sea validado por la CLI de OpenSpec y aprobado explícitamente por el usuario.
5. Tras verificación: commit → `/opsx:archive` para consolidar las especificaciones principales.
6. Skills canónicos viven en `ai-specs/skills/` y se exponen por symlinks a `.claude` / `.cursor` / `.agents`.

---

## 6. Git y trazabilidad

- Rama: `feature/task-<id>-<nombre>` (o `feature/<change-name>` si no hay task).
- Commit: `task-<id>: <tipo>(alcance): resumen` cuando hay task ZenTao.
- No commits de secretos (`.env`, credenciales).

---

## 7. Cumplimiento para agentes

Antes de escribir código o artefactos:

1. Leer este archivo.
2. Leer los estándares específicos del área tocada.
3. Seguir TDD y el contrato de tasks (pasos mandatorios por tipo de superficie).
4. Actualizar docs permanentes cuando el change cambie verdad de producto (ver `documentation-standards.md`).
