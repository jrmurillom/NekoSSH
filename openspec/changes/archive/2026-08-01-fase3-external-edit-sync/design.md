## Context

NekoSSH ya tiene Fase 1–2 en producto: perfiles SQLite, PTY SSH (`ssh2`, una Session por `terminal_id`), canal SFTP multiplexado (`sftp_list_dir`), explorador lazy en el sidebar, chrome A1/B3 (`overlays.ts`). `docs/project_scope.md` aún describe Fase 3 como Monaco + re-subida silenciosa con `Ctrl+S`.

Tras explore, el equipo pivota: esta entrega implementa **edición remota estilo FileZilla** (temp + editor externo + confirm antes de subir). Monaco queda **fuera de alcance / diferido (Fase 3b)**. No se modifica el change `chrome-confirms-context-menus` (completo); solo se reutiliza el patrón A1.

Constraints: código de app solo en `app/`; TDD; español latino en docs/OpenSpec; identificadores en inglés; sesión SFTP no debe tumbar el PTY (`set_blocking` agresivo prohibido en el modelo actual).

## Goals / Non-Goals

**Goals:**
- Flujo completo: doble clic / “Editar” → download temp → abrir editor externo → watch → A1 “¿Subir al servidor?” → upload/replace → cleanup razonable.
- Preferencia de usuario: ruta del editor externo (+ fallback a asociación OS).
- Políticas claras: desconexión mid-edit, archivos grandes/binarios, cancel en dialog dirty.
- Actualizar alcance de producto (Fase 3 = editor externo + sync; Monaco = 3b).

**Non-Goals:**
- Monaco / editor integrado en el workspace.
- Auto-upload sin confirmación.
- Diff visual, merge, multi-file batch edit, drag-drop masivo.
- Túneles runtime, host-key UI, auto-reconnect.
- Segundo login SSH para transferencias.

## Decisions

### 1. Pivot de alcance Fase 3 (producto)
- **Decisión**: Este change **redefine** la Fase 3 entregable como “edición remota vía editor externo + sync con confirm A1”. Monaco se documenta como **Fase 3b (futuro / diferido)**, no como dependencia de este change.
- **Alternativa**: Mantener Monaco como Fase 3 y llamar esto “Fase 2.5” — se descarta: confunde el roadmap y el usuario ya eligió el pivot.
- **Docs**: actualizar `docs/project_scope.md` y la fila Fase 3 en `ui-layout-contract.md` (workspace ya no promete pestaña Monaco en esta fase; la edición ocurre fuera del proceso).

### 2. Operaciones SFTP de archivo sobre la Session existente
- **Decisión**: Commands Tauri `sftp_download_file` / `sftp_upload_file` (nombres orientativos) ligados a `terminal_id` + path remoto, usando el **mismo** handle SFTP multiplexado que el listado. Lectura/escritura por chunks no bloqueantes / con yields, sin `set_blocking(true)` que corrompa el PTY.
- **Alternativa**: Segundo TCP login solo para transfer — se descarta (historial Fase 2: MaxSessions / tumba PTY).
- **Contrato**: Fallo de transfer informa error; PTY sigue usable cuando el fallo es solo de archivo.

### 3. Directorio temporal y naming
- **Decisión**: Temp bajo el app data dir de Tauri, p. ej. `…/edit-sessions/<edit_id>/` con el **basename** remoto (o basename + hash corto si hay colisión). Cada sesión de edición tiene un `edit_id` UUID y metadatos en memoria (y opcionalmente un mapa Rust): `terminal_id`, `remote_path`, `local_path`, `mtime`/`content_hash` baseline, estado (`watching` | `confirm_pending` | `uploading` | `closed`).
- **Alternativa**: Un solo archivo plano en `%TEMP%` del OS sin aislamiento — peor cleanup y colisiones.

### 4. Apertura del editor externo
- **Decisión**: Preferencia persistida `preferred_external_editor` (ruta ejecutable; string vacío = sin preferencia). Si hay ruta válida → spawn/open con ese ejecutable + path local. Si no → `open` / shell-execute con asociación del SO (plugin Tauri `opener` o equivalente del runtime).
- **Alternativa**: Forzar siempre un editor hardcodeado — se descarta.
- **UI**: Campo en el patrón existente **Settings / appearance** (modal o vista dedicada del layout contract): label en español (“Editor externo preferido”), input de path + botón examinar si el runtime lo permite.

### 5. Persistencia de preferencias
- **Decisión**: Tabla SQLite simple `app_preferences(key TEXT PRIMARY KEY, value TEXT)` (migración nueva) o fila única; lectura/escritura vía commands o plugin SQL ya usado. Arranque carga la preferencia al frontend.
- **Alternativa**: Solo `localStorage` — menos alineado con el resto de persistencia SQLite del producto; aceptable como fallback corto, pero preferir SQLite para SSOT de datos de app.

### 6. File watcher y debounce
- **Decisión**: Watcher nativo (crate tipo `notify`, o API Tauri) sobre el archivo local; **debounce** (~500–1000 ms) tras el último evento write/rename, luego comparar mtime/size/hash contra baseline. Si hay cambio real → emitir evento al frontend `edit-session-changed` → mostrar A1.
- **Alternativa**: Polling periódico — más simple pero menos responsive; usable como fallback si notify falla en algún OS.
- **Importante**: Muchos editores escriben vía temp+rename; el watcher MUST tolerar replace/rename al mismo path.

### 7. Confirm A1 (no silent upload)
- **Decisión**: Reusar `confirmDialog` de `overlays.ts` (patrón A1). Copy sugerido:
  - Título: “Subir cambios”
  - Cuerpo: “¿Subir al servidor?”
  - Detalle: **solo basename** del path remoto por defecto; control colapsable “ver ruta completa” que expone el path completo en textarea readonly (wrap/seleccionable) — ver Corrección de Ruta (Fix) de UX dialog
  - Primaria: “Subir” / Cancelar: “Cancelar”
- **Cancel / Escape**: no sube; la sesión de edición **sigue en watching** (el usuario puede seguir editando y volver a disparar el confirm en el próximo debounce). No hay segundo dialog “¿Descartar dirty?” en este MVP salvo que el usuario cierre explícitamente la sesión de edición (ver decisión 9).
- **Durante confirm pendiente**: no apilar múltiples A1 por el mismo `edit_id`; coalescer eventos.

### 8. Upload y baseline
- **Decisión**: Tras confirm → `sftp_upload_file` replace remoto → actualizar baseline local (mtime/hash) → seguir watching hasta cleanup. Error de upload → alert A1 / mensaje de estado; el archivo local dirty no se borra.
- **Overwrite remoto**: replace in-place del path remoto existente (FileZilla vibe); no versionado remoto en este change.

### 9. Cleanup de temporales
- **Decisión**:
  - Al cerrar sesión de edición explícita (futuro ítem UI opcional “dejar de vigilar”) o al cerrar la terminal/`close_ssh_session`: stop watcher, borrar el dir `edit-sessions/<edit_id>/` best-effort.
  - Al salir de la app: cleanup de huérfanos bajo `edit-sessions/` best-effort.
  - **No** borrar el temp mientras el dialog de subida está abierto o el upload está en curso.
- **Alternativa**: Dejar basura en TEMP del OS hasta reboot — se descarta.

### 10. Desconexión mid-edit
- **Decisión**: Si la Session/`terminal_id` se desconecta o cierra mientras hay edit sessions:
  1. Stop watchers ligados a ese `terminal_id`.
  2. Si hay dialog A1 de subida abierto → cerrarlo sin subir; opcional alert: “La sesión se desconectó; no se pudo subir.”
  3. Conservar el archivo local temporal **una vez** (no borrar de inmediato) para que el usuario no pierda trabajo; mostrar aviso no bloqueante. Cleanup diferido en próximo arranque o tras TTL corto documentado (p. ej. 24 h) — ver Open Questions.
  4. Reconectar (Ctrl+R manual existente) **no** reatacha automáticamente la edit session en este change (non-goal de auto-reconnect de edición).

### 11. Archivos grandes y binarios (default sensato)
- **Decisión (MVP)**:
  - **Límite de tamaño**: rechazo amable si el remoto supera **10 MiB** (configurable en código como constante; no UI de límite en este change). Mensaje: el archivo es demasiado grande para edición externa en esta versión.
  - **Binarios**: no hay detector MIME perfecto; heurística ligera (NUL en sample inicial y/o extensión de denylist opcional). Si se clasifica como binario → A1 de aviso “El archivo parece binario. ¿Abrir de todos modos?” antes de download; si cancela, no abre.
  - **No** bloquear por extensión de texto común (`.rs`, `.ts`, `.md`, `.conf`, sin extensión, etc.).
- **Alternativa**: Solo texto UTF-8 estricto — demasiado agresivo para configs con latin-1.

### 12. Entrada UX en el explorador
- **Decisión**:
  - **Doble clic** en nodo **archivo** → inicia edit session.
  - **Doble clic** en carpeta → comportamiento actual (abrir/navegar); sin cambio.
  - Menú contextual B3 sobre archivo: ítem **“Editar”** (icono pencil) además de acciones existentes.
- **Alternativa**: Solo menú, sin doble clic — peor ergonomía FileZilla.

### 13. Concurrencia
- **Decisión**: Permitir varias edit sessions en paralelo (archivos distintos). Mismo `remote_path` + `terminal_id`: reusar la session existente (focus/reabrir editor) en lugar de duplicar temp.

## Risks / Trade-offs

- **[Risk]** Watcher dispara en guardados parciales / antivirus → confirms espurios.
  - *Mitigación*: debounce + comparar hash/size; coalescer A1.
- **[Risk]** Transfer SFTP bloquea o degrada PTY.
  - *Mitigación*: mismo modelo no bloqueante que `sftp_list_dir`; chunks + yields; tests de teclado PTY durante download/upload.
- **[Risk]** Editor externo no libera file lock / escribe raro en Windows.
  - *Mitigación*: tolerar rename; reintentar lectura; documentar editores conocidos.
- **[Risk]** Usuario edita offline tras disconnect y espera sync mágico.
  - *Mitigación*: mensaje claro; no auto-reattach; temp conservado temporalmente.
- **[Risk]** 10 MiB arbitrario frustra logs grandes.
  - *Mitigación*: constante documentada; Open Question para subir límite o streaming edit más adelante.
- **[Trade-off]** Sin Monaco = sin pestaña editor en workspace; el “área editor” del layout contract se reformula para esta fase.

## Migration Plan

1. Migración SQLite `app_preferences` (idempotente).
2. Commands + watcher + frontend flow detrás de feature en la rama `feature/fase3-external-edit-sync`.
3. Actualizar `project_scope.md` + layout contract en el step de docs.
4. Rollback: revert de rama; temps huérfanos se limpian en arranque si el código de cleanup ya shippeó, o borrado manual de `edit-sessions/`.

No hay migración de datos de usuario crítica más allá de preferencia nueva (default vacío).

## Open Questions

1. **TTL de temps tras disconnect**: ¿24 h al arranque vs cleanup inmediato tras aviso? Propuesta default: conservar hasta cierre de app + sweep de huérfanos >24 h al startup.
2. **Límite 10 MiB**: ¿subir a 25–50 MiB tras feedback, o hacer preferencia avanzada?
3. **Denylist binaria**: ¿lista mínima de extensiones (`.png`, `.zip`, …) además del sample NUL, o solo heurística?
4. **Botón “Dejar de vigilar”** en UI: ¿MVP o solo cleanup por disconnect/app exit?
5. **Examinar archivo** para elegir editor: ¿dialog nativo Tauri en este change o path pegado a mano primero?

### Corrección de Ruta (Fix)

**MANDATORIO — no mutar el SSH de pruebas compartido**

Durante implementación, tests automatizados y verificación por agente de este change **MUST NOT** alterar ningún archivo en el host SSH de pruebas del equipo (ni upload/replace, ni delete, ni overwrite de configs/datos reales del lab).

- **Producto (uso real del usuario final)**: el flujo FileZilla **sigue subiendo** cuando el usuario confirma “¿Subir al servidor?”. Esta corrección **no** elimina upload del producto.
- **Dev / test / agent**: la evidencia se obtiene con **mocks**, **fixtures locales** o un **fake SFTP in-process** — nunca apuntando writes al lab host compartido.
- **Desktop UI / commands con SSH live**: postura por defecto = **cero escrituras remotas**. Si en el futuro se necesita un smoke live, solo dentro de un **sandbox remoto desechable** vacío/dedicado que el usuario cree y documente explícitamente; sin esa provisión, la verificación live se limita a **solo lectura / download a temp local** o se omite y se reporta N/A con mocks.
- **Targets de upload en verificación**: solo paths bajo sandbox explícito de sesión de prueba **o** mocks; **nunca** writes agent-driven a paths arbitrarios del servidor de pruebas compartido.
- **Decisión (sin Open Question)**: verificación de este change = mock/local only, salvo que el usuario entregue después una carpeta remota disposable documentada en tasks/reports.

### Corrección de Ruta (Fix)

**UX dialog A1 “Subir cambios” — overflow del path remoto**

**Problema**: El confirm de subida pasaba el path remoto completo (o truncado con ellipsis) como `impact` inline. Paths largos (p. ej. `/root/documentos/LLM_comparativa_junio_2026.md`) desbordan el panel A1 de ~320px.

**Estrategia**:
1. Por defecto mostrar **solo el filename** (basename del `remote_path`).
2. Control colapsable **“ver ruta completa”** / **“ocultar ruta”**.
3. Al expandir, mostrar el path completo en un **textarea** readonly (wrap + seleccionable) dentro del dialog, sin overflow horizontal.
4. Extensión mínima de `confirmDialog`: opciones opcionales `detailFilename` + `detailFullPath` (otros confirms siguen usando `impact` string). Mantener look glass A1.
