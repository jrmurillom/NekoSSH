## Context

Fase 2 de NekoSSH: añadir SFTP y explorador remoto sincronizado con la terminal, sobre el esqueleto Fase 1 (perfiles SQLite, PTY SSH con `ssh2`, xterm.js, shell Cyber-Sakura sin neon glow). El código vive solo en `app/`.

## Goals / Non-Goals

**Goals:**
- Segundo canal/sesión SSH dedicado a SFTP por terminal/sesión activa.
- Árbol de directorios remotos en el panel Archivos del sidebar.
- Sync: `cd` en terminal → explorador actualiza ruta; “Abrir en Terminal” → `cd` en la terminal activa.
- Actualizar `docs/design/ui-layout-contract.md` para reflejar Fase 2.

**Non-Goals:**
- Editor Monaco / descarga-edición-reupload (Fase 3).
- Upload/download masivo, drag-and-drop de archivos, permisos avanzados.
- Snippets, mascotas, multi-root SFTP sin sesión de terminal.

## Decisions

### 1. Segundo canal SFTP con `ssh2` (mismo crate que Fase 1)
- **Decisión**: Al iniciar (o al conectar) una sesión de terminal, abrir una segunda `Session`/`Channel` SFTP (o reutilizar autenticación y abrir `sftp()` en sesión paralela) asociada al mismo `terminal_id`.
- **Alternativa**: Multiplexar SFTP en el mismo canal PTY — se descarta porque bloquea la terminal.
- **Implementación**: Estado Rust `SftpSessions` (mapa `terminal_id` → handle SFTP); al `close_ssh_session` cerrar también SFTP.

### 2. Listado remoto vía commands Tauri
- **Decisión**: Commands `sftp_list_dir`, `sftp_stat` (si hace falta) invocados desde el frontend; el árbol se construye en TypeScript (expansión lazy por carpeta).
- **Alternativa**: Empujar todo el árbol al conectar — se descarta por latencia en homes grandes.

### 3. Detección de `cd` para sync Explorer ← Terminal
- **Decisión**: Tras salida del PTY (o periódicamente), consultar cwd remoto con un comando no intrusivo (`pwd`) vía canal de control o heurística sobre el prompt; preferir un comando Tauri `get_remote_cwd` que ejecute `pwd` en un canal auxiliar de la sesión SFTP/SSH sin ensuciar el PTY visible cuando sea posible. Si no es viable sin PTY, documentar fallback: escuchar líneas de salida que indiquen cambio de directorio y confirmar con `pwd` enviado de forma controlada.
- **Estrategia preferida**: canal exec separado `pwd` en la misma sesión SSH (no el PTY interactivo) ligado al `terminal_id`.

### 4. Abrir en Terminal (Explorer → Terminal)
- **Decisión**: Frontend invoca `write_ssh_input` (o command dedicado `ssh_cd`) con `cd <path>\n` hacia el PTY de la terminal activa, luego refresca el explorador.

### 5. UI Archivos
- **Decisión**: Habilitar tab Archivos (hoy disabled); mostrar árbol solo cuando hay sesión SSH activa; empty state si no hay conexión. Estilos con tokens planos de `DESIGN.md` (sin glow).

## Risks / Trade-offs

- **[Risk]** Detección de cwd frágil con prompts custom.
  - *Mitigación*: Preferir `pwd` por exec channel; no depender solo de parseo ANSI del PTY.
- **[Risk]** Doble autenticación / límites de MaxSessions en el servidor.
  - *Mitigación*: Reutilizar credenciales; una sesión SSH adicional por terminal; documentar fallo si el server rechaza.
- **[Risk]** Paths con espacios/unicode.
  - *Mitigación*: Escapar/quotear paths en `cd` y listados; tests con nombres edge.

### Corrección de Ruta (Fix)

**Directiva:** Código de aplicación solo en `app/`. Docs y OpenSpec en la raíz del workspace.

**Motivo del pivot (2026-07-30):**
1. El explorador **no sigue** el `cd` del shell interactivo: `exec("pwd")` en un canal auxiliar **no comparte** el cwd del PTY.
2. Un segundo login SSH para SFTP (`SftpSessions`) puede tumbar la sesión PTY (p. ej. MaxSessions / un solo login).
3. Falta UX de navegación: ruta editable + icono **Ir**, y refresco como **icono** (no solo texto).
4. Las verificaciones previas no dieron **garantía real** de sync tras `cd` en terminal.

**Nueva estrategia:**

| Tema | Antes (inválido / insuficiente) | Ahora |
|------|----------------------------------|--------|
| SSH | 2º TCP login SFTP | **Una** `Session` por terminal; PTY + SFTP/exec como canales multiplexados |
| Cwd interactivo | `exec pwd` auxiliar | Cwd del **mismo shell PTY**: sonda con marcadores (escribir `pwd` acotado al PTY, leer/filtrar marcadores sin ensuciar xterm), o equivalente documentado; **prohibido** usar exec-cwd como fuente del sync Explorer←Terminal |
| Sync | Poll de exec-cwd | Tras `cd` detectado / sonda post-comando / “Abrir en Terminal” / Ir: refrescar árbol con cwd **del shell** |
| Path UI | Label solo lectura | Input editable de ruta + icono **Ir** (navega explorador vía SFTP a esa path) + icono **Actualizar** (relista cwd actual) |
| Garantías | Reports sin smoke real de sync | Tests unitarios de parsing/marcadores + verificación desktop-commands/UI **ejecutada por el agente** con evidencia de: (a) `cd` en terminal → explorador cambia; (b) Ir a path → árbol lista esa ruta; (c) teclado PTY sigue vivo tras SFTP |

**Zombies a limpiar / refactorizar:**
- Estado `SftpSessions` / segundo `authenticate_session` si aún queda rastro.
- Poll/intervalo basado en `get_remote_cwd` vía exec como “sigue el cd”.
- Cualquier claim en reports de sync cwd que no se haya reproducido tras este fix.

**Non-goal de este fix:** sync perfecto con prompts exóticos sin shell usable; si la sonda PTY falla, degradar con mensaje claro y mantener Ir / Abrir en Terminal.

### Corrección de Ruta (Fix)

**Pivot (2026-07-30, v2 — modelo sólido acordado):**

Descartar sonda PTY con marcadores (`pwd` inyectado). Modelo final:

1. **Una** Session SSH por terminal; SFTP = canal multiplexado (nunca 2º login).
2. **Explorador** = navegador SFTP independiente: path editable + icono **Ir** + icono **Actualizar** + “Abrir en Terminal”.
3. **Sync Terminal → Explorador** = parsear **OSC 7** (`\033]7;file://…/path\007`) en la salida del PTY. Opcional al conectar: one-liner para habilitar OSC 7 en bash/zsh si el shell no lo emite.
4. Si no hay OSC 7 → no fingir sync; el usuario usa Ir / Abrir en Terminal.
5. **Prohibido** como fuente de sync: `exec pwd` aislado y sondas PTY con marcadores.

**Zombies:** resto de `SftpSessions`/2º login; poll `get_remote_cwd` exec-as-sync; código/docs de “sonda PTY con marcadores” del fix v1.

### Corrección de Ruta (Fix)

**Pivot (2026-07-30, v3 — árbol navegable):**

**Problema reportado:** el árbol de Archivos está mal construido: no se navega, no colapsa/expande de forma usable, no es funcional.

**Causas probables (a verificar en apply):**
1. Cada OSC 7 / `PROMPT_COMMAND` hace `loadExplorerAt(path, true)` y **destruye** el estado expandido → el árbol “no se descolapsa” / se resetea.
2. El click solo hace toggle expand; no hay “entrar” a carpeta (actualizar path bar + listar como raíz del panel) ni subir al padre.
3. Layout CSS del panel/árbol puede impedir scroll o altura útil.
4. Hook OSC 7 visible en el PTY (ruido UX; aparte del árbol).

**Estrategia:**
- Árbol lazy **estable**: expand/collapse no se pierde por OSC 7 si el path no cambió; si el path cambió, relistar la nueva raíz.
- Navegación: click en chevron = expand/collapse; click/doble-click en carpeta = **abrir** (path bar + listado de esa ruta); control **Subir** (padre).
- Empty/error/loading visibles; dirs vacíos colapsables.
- Verificación UI obligatoria: expand → hijos; collapse; abrir carpeta; subir; Ir; sin reset al recibir OSC 7 del mismo path.
- Fuera de alcance Fase 2 (documentar, no implementar aquí): upload/download, editar archivo, drag-drop, multi-root.

**Qué más falta del explorador (inventario):**
| Ítem | ¿Fase 2? |
|------|----------|
| Expand/collapse + lazy load fiable | **Sí — este fix** |
| Entrar a carpeta / subir padre | **Sí — este fix** |
| Path Ir + Actualizar | Ya (arreglar si rompe árbol) |
| Abrir en Terminal | Ya |
| Sync OSC 7 sin romper árbol | **Sí — este fix** |
| Ocultar hook OSC en PTY | Deseable en este fix |
| Iconos/file size/sort UI | Nice-to-have |
| Upload / download / Monaco | No (Fase 3+) |

### Corrección de Ruta (Fix)

**Pivot (2026-07-30, v4 — definitivo, sin seguir-cd):**

**Alcance acordado (solo esto):**
1. Terminal estable: Conexión SSH **A** = solo PTY. Prohibido SFTP/`set_blocking` sobre esa Session.
2. Árbol navegable: Conexión SSH **B** = solo SFTP, **viva mientras dure la terminal** (mismo perfil). Expand / abrir / subir / Ir / Actualizar / Abrir en Terminal.
3. Pruebas reales: unitarias + **smoke SSH** (tipear sin caída + listar/navegar árbol) antes de marcar tasks. Sin smoke → no está hecho.

**Fuera de alcance (explícito):**
- Seguir el `cd` automático / sync OSC 7 como requisito de producto.
- Inyectar `PROMPT_COMMAND` u hooks en el PTY.

**Zombies a limpiar:**
- SFTP multiplexado en la Session del PTY con `set_blocking`.
- Listeners/código de sync OSC 7 → explorador (o dejar inerte; no promesa de producto).
- Claims `[x]` / reports que afirmaron PASS sin smoke SSH real — rehacer verificación honesta.

**Si MaxSessions no permite conexión B:** explorador muestra error claro; PTY **no** se cae.

### Corrección de Ruta (Fix)

**Pivot (2026-07-31, v5 — ciclo de vida: cerrar conexiones con pestaña/app):**

**Problema:** Las sesiones SSH (PTY + SFTP en la misma Session / mapa `SshConnections`) pueden quedar vivas en el proceso o en el servidor cuando:
1. Se cierra una pestaña de terminal (cleanup incompleto: solo `channel.close`, sin `session.disconnect` / drop ordenado).
2. Se cierra la ventana / se sale de la app (no hay hook Tauri `ExitRequested` / `RunEvent::Exit` / `CloseRequested` que cierre **todas** las entradas de `SshConnections`).
3. “Cerrar todas” no garantiza await ordenado ni limpieza del explorador ligado.

**Estrategia:**
| Evento | Comportamiento requerido |
|--------|--------------------------|
| Cerrar pestaña de terminal | Frontend → `close_ssh_session(terminal_id)`; backend: quitar del mapa, cerrar canal PTY, `disconnect` de la Session, drop handles; UI limpia explorador si estaba bound a ese id |
| Cerrar todas las pestañas | Mismo cleanup por cada `terminal_id` (secuencial o paralelo controlado); sin sesiones huérfanas |
| Cerrar ventana / salir de la app | Hook nativo: iterar `SshConnections` y cerrar **todas** antes de terminar el proceso |
| Crash / kill del OS | Best-effort (TCP muere); no requisito de graceful en kill -9 |

**Zombies / gaps actuales a cubrir en apply:**
- `close_ssh_session` que solo hace `channel.close` sin `session.disconnect`.
- Ausencia de `RunEvent` / `on_window_event(CloseRequested)` / `ExitRequested` que vacíe el mapa.
- Posible race: hilo PTY reader vs close (debe tolerar remove del mapa sin panic).

**Non-goals de este fix:** reconexión automática; pool de conexiones; keepalive como sustituto de close.
