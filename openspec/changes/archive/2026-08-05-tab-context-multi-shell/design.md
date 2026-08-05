## Context

Hoy en NekoSSH cada pestaña es un `terminalId` con un xterm y una Session SSH (`LiveSsh`: PTY + SFTP en la misma Session). El explorador y la edición externa se ligan a ese `terminalId`. El fondo/opacidad y el glow sakura viven en `.terminal-panel`, fusionado visualmente con la pestaña activa.

El producto necesita varios shells del mismo servidor visibles a la vez, sin romper el contrato “pestaña ↔ SFTP” ni el look de panel unificado.

## Goals / Non-Goals

**Goals:**

- Pestaña = contexto con shell **padre** (ancla SFTP) + hasta **3 hijos** (mismo perfil, logins independientes).
- Cerrar pestaña cierra padre + hijos; cerrar hijo solo cierra ese shell.
- Padre estático en UI (sin × de celda); si el padre muere, el contexto queda desconectado.
- Layout progresivo 1 → 2 → T(3) → 2×2(4) dentro de un solo `.terminal-panel`.
- Fondo/opacidad/glow sin cambios de contrato: una superficie por pestaña; celdas transparentes.
- Teclado e I/O al shell con foco; SFTP siempre al padre del contexto activo.

**Non-Goals:**

- Multi-canal PTY sobre una sola Session (enfoque A).
- Grid entre pestañas de distintos servidores (mosaico cross-tab).
- Más de 4 celdas, splits recursivos tipo tmux, o redimensionar divisores arrastrables (v1: proporciones fijas CSS grid).
- Cambiar auth, perfiles o esquema SQLite.
- Publicar/macOS/Linux release.

## Decisions

### D1 — Modelo B: N Sessions, padre ancla SFTP

Cada shell (padre o hijo) es un `terminal_id` y una entrada en `SshConnections` (login completo). El contexto guarda `parentTerminalId` + `childTerminalIds[]` (máx. 3). SFTP, `cd` desde explorador y edición externa usan solo el padre.

**Alternativa rechazada:** multi-canal en una Session — más elegante en SSH, pero exige rearmar `LiveSsh` y el pump PTY; mayor riesgo.

### D2 — Ciclo de vida

| Acción | Efecto |
|---|---|
| Abrir pestaña / conectar perfil | Crea contexto + padre; layout 1 celda |
| Nuevo shell | Nuevo `terminal_id`, mismo perfil; se añade al grid |
| Cerrar hijo (× en celda) | `close_ssh_session(childId)`; reflow del grid |
| Cerrar pestaña | Confirmar si hay conexiones vivas; cierra padre + todos los hijos |
| Padre desconectado (EOF/error) | Contexto desconectado; UI ofrece reconectar contexto (Ctrl+R en padre); hijos se cierran al reconectar o al cerrar tab (v1: al caer el padre, marcar contexto offline y cerrar hijos para evitar shells huérfanos sin SFTP coherente) |

### D3 — IDs y frontend

```
TabContext {
  tabId: string              // id de pestaña UI
  profile: ConnectionProfile
  parentTerminalId: string
  childTerminalIds: string[] // max 3
  focusedTerminalId: string  // padre o hijo
  panelEl, tabEl
  // cwd/explorer cache siguen en el contexto (padre)
}
ShellPane {
  terminalId: string
  role: 'parent' | 'child'
  term, fitAddon, cellEl
  isConnected, ...
}
```

Backend sin comandos nuevos: `start_ssh_session` / `write_ssh_input` / `resize_ssh_pty` / `close_ssh_session` por `terminal_id`. El frontend agrupa al cerrar tab.

### D4 — Layout CSS

- `.terminal-panel` conserva background-image, `--terminal-overlay-opacity`, border-radius, glow.
- Interior: `.term-grid` con clases de densidad `cells-1|2|3|4`.
  - 1: una celda
  - 2: dos columnas
  - 3: fila superior 2 cols (padre | hijo1); fila inferior 1 col span (hijo2) — forma T
  - 4: `grid-template: 1fr 1fr / 1fr 1fr` — padre arriba-izq
- `.term-cell`: sin fondo propio; borde hairline; `.focused` con borde sakura.
- Padre: sin botón cerrar celda; badge mínimo de ancla.
- Resize: `FitAddon` por celda al cambiar layout o ventana.

### D5 — Foco y entrada

- Click en celda → `focusedTerminalId`; `term.focus()`.
- `onData` / write IPC usan el focused id.
- Eventos `ssh-stdout` / `ssh-closed` se enrutan por `terminal_id` al pane correcto (igual que hoy, con mapa pane).

### D6 — “Abrir en Terminal” desde SFTP

Siempre envía `cd` al **padre** (ancla del contexto), no al hijo enfocado. Evita sorpresas con el explorador.

## Risks / Trade-offs

- [N logins al mismo host] → Mitigación: tope 4; mismo perfil; documentar en UI.
- [Padre cae y deja hijos vivos] → Mitigación v1: al `ssh-closed` del padre, cerrar hijos y marcar contexto desconectado.
- [Fit/xterm en grids pequeños] → Mitigación: fit al entrar/salir de celda y al resize; padding del panel se reparte (revisar 24px: puede vivir en el panel o reducirse por celda sin recortar texto).
- [Complejidad en `main.ts`] → Mitigación: introducir `TabContext` + helpers de grid antes de tocar SFTP; tests/manual checklist por densidad.
- [Edición externa ligada a terminalId] → Mitigación: sesiones de edit siguen ligadas al padre; al cerrar tab se limpian todas las del contexto.

## Migration Plan

- Sin migración de datos.
- Comportamiento actual = contexto con 0 hijos (layout 1 celda).
- Rollback: revertir el change; backend sigue compatible 1:1.

## Open Questions

- Atajo de teclado para ciclar foco entre celdas (candidato: Ctrl+Tab dentro del panel) — se puede dejar para iteración corta si no entra en el primer apply.
- ¿El shell padre muestra título “Principal” o solo el badge? Preferencia: badge mínimo, sin ruido.
