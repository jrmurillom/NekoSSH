## Context

NekoSSH entrega (o está entregando en el change hermano `fase3-external-edit-sync`) edición remota FileZilla-style: temp local, editor externo, watcher, confirm A1 «¿Subir al servidor?» y upload SFTP normal sobre la Session existente. Ese upload falla con frecuencia en paths con permisos restringidos (owned por root / sin write del usuario SSH).

Este design cubre **solo** el reintento opcional con sudo tras ese fallo. No redefine el ciclo de edición ni el confirm de subida inicial. Chrome A1 (`overlays.ts`) se reutiliza. Verificación de agente: **cero writes** al host SSH de pruebas compartido (mocks / fake SFTP / harness local), igual que fase3.

Constraints: código de app solo en `app/`; TDD; español latino en docs/OpenSpec/UI; identificadores en inglés; no tumbar el PTY con `set_blocking` agresivo; sin UI de password.

## Goals / Non-Goals

**Goals:**
- Tras fallo del upload SFTP post-confirm, ofrecer A1 «Subir con sudo».
- Un único path elevado mínimo y predecible (temp remoto + `sudo cp` no interactivo, o equivalente igualmente simple).
- Fallar y parar si sudo pide password, no hay TTY usable, o el comando falla; alert claro; temp local dirty conservado.
- Tests/verificación del camino elevado sin mutar el lab SSH.

**Non-Goals:**
- Sudo como camino por defecto / always-on.
- Prompt de password (UI o agent que “adivine” credenciales).
- `sudo -S` leyendo password desde stdin de la app.
- Monaco, rewrite de fase3, multi-file, diff/merge.
- Configurar `NOPASSWD` en el servidor remoto desde la app.
- Smoke live que escriba en el lab compartido.

## Decisions

### 1. Sibling delta sobre fase3 (no rewrite)
- **Decisión**: Change nuevo `external-edit-sudo-retry` que depende del código/comportamiento de `fase3-external-edit-sync`. Apply asume que el flujo confirm → `sftp_upload_file` ya existe.
- **Alternativa**: Meter sudo dentro del mismo change fase3 — se descarta: fase3 ya está complete; el usuario pidió sibling.

### 2. Orden de intentos (siempre normal primero)
- **Decisión**:
  1. Usuario confirma «¿Subir al servidor?» → upload SFTP normal (fase3).
  2. Éxito → baseline + seguir watching (sin mencionar sudo).
  3. Fallo → clasificar; si es candidato a elevación (permisos / permission denied / write error / EACCES-equivalente en mensaje ssh2) → A1 «Subir con sudo».
  4. Otros fallos (desconexión, path inexistente, sesión cerrada) → alert de error **sin** ofrecer sudo.
- **Alternativa**: Ofrecer sudo siempre al confirmar — se descarta (non-goal always-on).

### 3. Mecánica elevada: temp remoto + `sudo cp` (mínima)
- **Decisión**: Path elevado único y simple:
  1. Subir el contenido local vía SFTP a un path temporal remoto writable por el usuario (p. ej. `/tmp/nekossh-edit-<edit_id>-<basename>` o bajo home).
  2. Ejecutar en la Session SSH (canal exec, no PTY interactivo) un comando no interactivo del estilo:
     `sudo -n cp -- <temp_remoto> <remote_path_destino>`
     (quoting seguro de paths; sin shell interpolation de input de usuario sin escape).
  3. Best-effort borrar el temp remoto tras éxito o fallo del cp.
  4. Si éxito → actualizar baseline local como en upload normal.
- **`sudo -n`**: falla inmediato si se requeriría password (no prompt).
- **Alternativa A**: `sudo tee` con stdin del archivo — más frágil con binarios/grandes; se descarta para MVP.
- **Alternativa B**: segundo login root — fuera de alcance.
- **Alternativa C**: `pkexec` / Polkit — no aplica en servidor remoto típico.

### 4. Canal de ejecución
- **Decisión**: Usar un canal `exec` de corta duración sobre la **misma** Session SSH del `terminal_id` (como el SFTP multiplexado), sin `set_blocking(true)` prolongado que tumbe el PTY. Timeout corto (p. ej. 15–30 s) al exec.
- **Alternativa**: Inyectar el comando en el PTY del usuario — se descarta (contamina la terminal visible y es frágil).

### 5. UX de dialogs
- **Decisión**:
  - Primer fallo candidato: A1 título tipo «Error al subir» / cuerpo que explique permisos + primaria **«Subir con sudo»** + cancelar.
  - Cancel / Escape: no intenta sudo; temp dirty queda; watching puede continuar.
  - Fallo del path elevado: alert A1 (o mensaje de estado) con razón usable (p. ej. «sudo requiere contraseña o no está disponible», «comando sudo falló»); **no** reabrir automáticamente el dialog de sudo; temp dirty conservado.
  - Durante el intento elevado: no apilar dialogs; deshabilitar doble click de reintento en el mismo flujo.
- Copy en español latino; tokens/overlays existentes.

### 6. API de commands (orientativa)
- **Decisión**: Extender el flujo de edit-session, p. ej.:
  - Frontend detecta error de upload y, si el backend marca `elevatable: true` (o código `permission_denied`), muestra el A1 sudo.
  - Command dedicado `edit_session_upload_with_sudo(edit_id)` **o** flag `elevated: true` en el upload existente — preferir command/flag explícito para tests claros.
- Respuesta de error estructurada: `{ kind: "permission_denied" | "sudo_password_required" | "sudo_failed" | "disconnected" | …, message }`.

### 7. Seguridad y quoting
- **Decisión**: Paths remotos y temp se pasan como argumentos escapados (sin `sh -c` con string cruda del usuario). Rechazar paths con NUL. No loguear contenido del archivo. No persistir password.
- El usuario ya tiene Session autenticada; sudo `-n` solo aprovecha NOPASSWD si el host lo permite.

### 8. Lab safety (verificación)
- **Decisión**: Tests unitarios del clasificador de errores + del builder del comando sudo + fake SFTP/exec in-process. Desktop-commands/UI verification **MUST NOT** escribir al lab SSH compartido. Documentar N/A para writes remotos live. Igual espíritu que el requirement de fase3.

## Risks / Trade-offs

- **[Risk]** Hosts sin `sudo -n` / sin NOPASSWD → el reintento siempre falla.
  - *Mitigación*: mensaje claro; non-goal de password UI; el usuario puede arreglar sudoers fuera de la app.
- **[Risk]** `sudo cp` no preserva ownership/mode como espera el admin.
  - *Mitigación*: documentar que el destino conserva metadata según `cp` del host; Open Question si hace falta `install -o/-g` después.
- **[Risk]** Exec remoto tumba o degrada PTY.
  - *Mitigación*: canal exec corto; mismos cuidados anti-blocking que SFTP; test de aislamiento con mock.
- **[Risk]** Clasificar mal errores → ofrecer sudo cuando no ayuda (disco lleno, etc.).
  - *Mitigación*: allowlist estrecha de permission/write denied; resto → error simple.
- **[Trade-off]** Simplicidad de `sudo -n cp` vs. soporte password: se elige fallar limpio.

## Migration Plan

1. Implementar sobre rama `feature/external-edit-sudo-retry` con fase3 ya en el árbol (merge/rebase según convención del equipo).
2. Sin migración SQLite nueva.
3. Rollback: revert del change; el upload normal de fase3 sigue igual.

## Open Questions

1. ¿Nombre final del command: `edit_session_upload_with_sudo` vs. flag en upload existente? (Default de apply: command dedicado si simplifica tests.)
2. ¿Temp remoto bajo `/tmp` o bajo `$HOME/.cache/nekossh/`? (Default: `/tmp` con prefijo `nekossh-edit-`.)
3. ¿Preservar mode con `cp -p`? (Default MVP: `cp` sin `-p`; ajustar solo si un caso real lo exige.)
