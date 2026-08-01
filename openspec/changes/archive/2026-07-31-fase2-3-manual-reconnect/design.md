## Context

NekoSSH (Fase 2) mantiene una Session SSH por pestaña (`terminal_id`): PTY + SFTP, keepalive activo, cierre limpio. Al morir la sesión remota, el frontend ya escribe un aviso amarillo y pone el status en "Desconectado", pero no ofrece reconexión desde la misma pestaña. El usuario eligió el modelo Terminus/Moba: **manual reconnect** (no auto). Este change es la Fase **2.3** de higiene antes de Monaco.

## Goals / Non-Goals

**Goals:**
- Aviso claro en el PTY: sesión muerta + `Ctrl+R` para reconectar.
- `Ctrl+R` reconecta con el mismo perfil solo si la pestaña está desconectada.
- Indicador (dot + texto) con estados coherentes: connecting / connected / disconnected / error.
- Conservar en `ActiveTerminal` lo necesario para reabrir (`profile` o id + credenciales ya usadas).

**Non-Goals:**
- Auto-reconnect / reintentos en background.
- Restaurar cwd, historial de shell, o buffers del editor.
- Túneles runtime, known_hosts, cambios de keepalive.
- Fase 3 Monaco.

## Decisions

### 1. Reconnect = nuevo `start_ssh_session` sobre el mismo `terminal_id`
- **Decisión**: Tras disconnect, el backend ya no tiene la Session. Ctrl+R vuelve a invocar el flujo de conexión existente con el mismo `terminal_id` y credenciales del perfil guardado en la pestaña. Si el mapa aún tiene entrada stale, `close_ssh_session` idempotente antes de abrir.
- **Alternativa**: nuevo `terminal_id` + pestaña nueva — se descarta (rompe el modelo “misma pestaña”).

### 2. Snapshot de perfil en la pestaña
- **Decisión**: Al crear la pestaña, guardar referencia al perfil (objeto o campos mínimos: host, port, user, auth, keepalive) en `ActiveTerminal` para no depender de que el usuario lo tenga seleccionado en el sidebar.
- **Alternativa**: solo `profileId` + re-fetch DB — también válido; preferir id + re-load desde `currentProfiles` o invoke get si hace falta frescura de password.

### 3. Ctrl+R solo si desconectado
- **Decisión**: El keybinding se registra en xterm `attachCustomKeyEventHandler` (o equivalente): si `isConnected`, no interceptar Ctrl+R (dejar pasar al remoto). Si `!isConnected`, prevenir default y disparar reconnect.
- **Alternativa**: botón en la status bar — complementar opcional; Ctrl+R es el contrato mínimo.

### 4. Copy y estados visuales
- **Decisión**: Mensaje PTY en español, estilo Cyber-Sakura (ámbar/error ya usados). Status text: "Conectando...", "Conectado", "Desconectado — Ctrl+R", "Error de Conexión". Dot: clases `connecting` / `connected` / `disconnected` / `error`.
- Explorador SFTP: al disconnect, vaciar como hoy; al reconnect exitoso, re-habilitar cuando `ssh-connected` (comportamiento actual).

## Risks / Trade-offs

- **[Risk]** Ctrl+R choca con atajos del shell remoto → *Mitigación*: solo interceptar cuando `!isConnected`.
- **[Risk]** Password cambiada en DB mientras la pestaña vive → *Mitigación*: al reconnect, preferir re-leer perfil por id si existe.
- **[Risk]** Doble Ctrl+R rápido abre dos sesiones → *Mitigación*: flag `isReconnecting` / ignorar mientras connecting.
- **[Trade-off]** No restaurar cwd: usuario hace `cd` de nuevo — aceptado explícitamente.

## Migration Plan

1. Solo frontend (+ posible cleanup idempotente backend).
2. Sin migración SQL.
3. Rollback: revert commits de UI/keybinding.

## Open Questions

- ¿Botón “Reconectar” junto al status además de Ctrl+R? **Nice-to-have**; no bloquea 2.3.
- ¿Mismo `terminal_id` obligatorio? **Sí** (decisión 1).
