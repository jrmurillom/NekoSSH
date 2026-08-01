## Why

Hoy la lista lateral trata cada conexión SSH como un ítem plano (“perfil”). El producto necesita un nivel intermedio de **carpetas/grupos (profiles)** para organizar conexiones (p. ej. Production / Staging), con edición de nombre **inline**, creación de carpetas desde el icono de carpeta, y conexiones creadas **dentro** de una carpeta — incluyendo el CRUD de persistencia.

## What Changes

- Introducir entidad **carpeta/profile group** en un nivel por encima de las conexiones SSH existentes.
- UI tipo árbol (referencia adjunta): encabezado CONNECTIONS, `+`/carpeta para agregar carpeta, chevron expand/collapse, icono carpeta + nombre editable inline; conexiones anidadas bajo la carpeta (estado/latencia si ya existen o como placeholder de UI).
- Las **conexiones se crean sobre carpetas** (no como raíz plana sin carpeta, salvo migración/default documentada).
- Actualizar **CRUD** SQLite + commands Tauri: create/list/update/delete de carpetas; create/update/delete de conexiones con `folder_id` (o equivalente); cascade al borrar carpeta.
- Migración de datos: perfiles/conexiones actuales → carpeta por defecto o una carpeta “General” para no perder datos.
- Iconografía Lucide outline (ya en app) para carpeta / chevron / add.

## Capabilities

### New Capabilities
- `connection-folders`: Carpetas/grupos de conexiones en sidebar; expand/collapse; rename inline; create folder; CRUD de carpetas.

### Modified Capabilities
- `connection-profiles`: Los perfiles de conexión SSH pasan a vivir **dentro** de una carpeta; CRUD y listado jerárquico; el modal/flujo de “nueva conexión” requiere carpeta destino.

## Impact

- DB: nueva tabla (o rename semántico) + FK desde conexiones; migración SQL en `app/src-tauri/migrations/`.
- Backend Rust: models/commands CRUD carpetas + adaptar get/create/update/delete de conexiones.
- Frontend: `app/src/main.ts`, `index.html`, `styles.css` — árbol jerárquico, inline edit, acciones add folder / add connection.
- Docs: `ui-layout-contract.md`, `DESIGN.md` si hay patrones de árbol/inline edit; README.
- Specs main: `connection-profiles` + nuevo `connection-folders`.
