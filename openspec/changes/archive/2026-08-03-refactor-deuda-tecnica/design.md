## Context

Se requiere resolver el 100% de los hallazgos de severidad **Crítica** y **Alta** identificados en la auditoría técnica de NekoSSH.

## Goals / Non-Goals

**Goals:**
- Eliminar el riesgo de *lock poisoning* en Rust (`RUST-CRIT-1`).
- Corregir el warning de visibilidad de `SshConnections` (`RUST-CRIT-2`).
- Modularizar `main.ts` (`ARCH-HIGH-1`) y organizar `lib.rs` (`ARCH-HIGH-2`).
- Incorporar pruebas unitarias valiosas tanto en Rust como en TypeScript.

**Non-Goals:**
- Modificar esquemas de la base de datos de producción.
- Cambiar la paleta o tokens visuales del tema Cyber-Sakura.

## Decisions

### 1. Manejo Seguro de Locks en Rust (`RUST-CRIT-1`)
- En lugar de `edits.lock().unwrap()`, utilizar la llamada segura `match edits.lock()` o un helper `lock_safe()` que capture `PoisonError` y recupere el guard con `unwrap_or_else(|e| e.into_inner())`.

### 2. Corrección de Visibilidad (`RUST-CRIT-2`)
- En `lib.rs:35`, cambiar `pub(crate) struct LiveSsh` a `pub struct LiveSsh`.

### 3. Modularización Frontend (`ARCH-HIGH-1`)
- Extraer funciones puras de `main.ts` a `app/src/modules/connection-tree-helper.ts` y `app/src/modules/sftp-path-helper.ts`.

###  decision: Corrección Limpia de Foco y Cursor (Fix Pivot)
- **Eliminación de Parches Conflictivos**: Se remueven los manejadores manuales de mousedown/click en `main.ts` que competían y saboteaban el foco nativo de `xterm.js`.
- **Eliminación de Sobreescritura CSS del Cursor**: Se elimina la animación `@keyframes cyber-cursor-blink` forzada que rompía la lógica interna del cursor, causando que parpadeara al perder foco y se quedara estático al enfocarse.
- **Mantener Corrección de Dimensiones**: Se conserva la asignación de dimensiones en `.terminal-canvas-container` (`flex: 1; height: calc(100% - 32px); width: 100%; overflow: hidden;`) para asegurar que el área de la terminal sea cliqueable por el usuario.

## Risks / Trade-offs

- Ninguno. Se vuelve al comportamiento estándar de xterm.js.
