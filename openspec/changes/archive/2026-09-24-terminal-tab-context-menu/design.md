# Technical Design: `terminal-tab-context-menu`

## 1. Architecture Overview

El diseño divide la solución en tres capas limpias y desacopladas:

```
┌───────────────────────────────────────────────────────────────────────────┐
│ 1. UI Overlay Layer (`app/src/overlays.ts` & `app/src/styles.css`)        │
│    - Extiende `ContextMenuItem` con `disabled?: boolean`                  │
│    - Renderiza `.chrome-context-item.is-disabled` (`btn.disabled = true`) │
└─────────────────────────────────────┬─────────────────────────────────────┘
                                      │
                                      ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ 2. Pure Domain Helper (`app/src/modules/terminal-tab-menu-helper.ts`)     │
│    - `buildTabContextMenuState(orderedIds, targetId)`                     │
│    - `resolveTabsToClose(orderedIds, targetId, action)`                   │
│    - `buildBulkTabCloseConfirm(action, totalToClose, connectedCount)`     │
│    - `resolveNextActiveTabAfterBulkClose(activeId, targetId, closedIds)`  │
└─────────────────────────────────────┬─────────────────────────────────────┘
                                      │
                                      ▼
┌───────────────────────────────────────────────────────────────────────────┐
│ 3. Tab Bar Controller (`app/src/main.ts`)                                 │
│    - Listener `contextmenu` en `.term-tab` dentro de `openTerminalTab`    │
│    - Ejecuta `closeTerminalTabsSubset(targetTerminalId, action)`          │
│    - Confirmación consolidada única (`confirmDialog`) + cierre secuencial │
│      con `closeTerminalSession(id, true)` y ajuste final de foco          │
└───────────────────────────────────────────────────────────────────────────┘
```

## 2. Detailed Component Design

### 2.1 Extensión de `showContextMenu` (`app/src/overlays.ts` & `app/src/styles.css`)
- En `ContextMenuItem`:
  ```ts
  export type ContextMenuItem = {
    id: string;
    label: string;
    icon?: IconNode;
    danger?: boolean;
    disabled?: boolean;
    separatorBefore?: boolean;
  };
  ```
- En `showContextMenu`:
  - Cuando `item.disabled === true`:
    - Se asigna `btn.disabled = true`, `btn.setAttribute("aria-disabled", "true")` y se añade la clase `is-disabled`.
    - El manejador `click` ignora cualquier activación si `item.disabled` es verdadero (no cierra el menú ni resuelve la promesa con `item.id`).
- En `app/src/styles.css`:
  - `.chrome-context-item.is-disabled`: `opacity: 0.42; cursor: not-allowed; pointer-events: none; color: var(--color-text-secondary); background: transparent;`.

### 2.2 Módulo Puro `app/src/modules/terminal-tab-menu-helper.ts`
- **`TabContextMenuAction`**: `"close" | "close-others" | "close-left" | "close-right" | "close-all"`
- **`buildTabContextMenuState(orderedIds: string[], targetId: string)`**:
  - Calcula `index = orderedIds.indexOf(targetId)`.
  - `hasLeft = index > 0`
  - `hasRight = index >= 0 && index < orderedIds.length - 1`
  - `hasOthers = orderedIds.length > 1 && index >= 0`
  - Devuelve los descriptores listos para `showContextMenu`:
    1. `{ id: "close", label: "Cerrar pestaña", disabled: index < 0 }`
    2. `{ id: "close-others", label: "Cerrar otras pestañas", disabled: !hasOthers }`
    3. `{ id: "close-left", label: "Cerrar pestañas a la izquierda", disabled: !hasLeft, separatorBefore: true }`
    4. `{ id: "close-right", label: "Cerrar pestañas a la derecha", disabled: !hasRight }`
    5. `{ id: "close-all", label: "Cerrar todas las pestañas", danger: true, disabled: orderedIds.length === 0, separatorBefore: true }`
- **`resolveTabsToClose(orderedIds: string[], targetId: string, action: TabContextMenuAction): string[]`**:
  - `"close"` $\rightarrow$ `[targetId]`
  - `"close-others"` $\rightarrow$ `orderedIds.filter((id) => id !== targetId)`
  - `"close-left"` $\rightarrow$ `orderedIds.slice(0, index)`
  - `"close-right"` $\rightarrow$ `orderedIds.slice(index + 1)`
  - `"close-all"` $\rightarrow$ `[...orderedIds]`
- **`buildBulkTabCloseConfirm(action: Exclude<TabContextMenuAction, "close">, totalToClose: number, connectedCount: number)`**:
  - Genera el título, mensaje en español y `confirmLabel` precisos según la dirección (`a la izquierda`, `a la derecha`, `otras`, `todas`).
- **`resolveNextActiveTabAfterBulkClose(currentActiveId: string | null, targetId: string, closedIds: string[]): string | null`**:
  - Si `closedIds` incluye a `targetId` (caso `"close-all"`), retorna `null`.
  - Si `currentActiveId` estaba dentro de `closedIds` (o era `null`), retorna `targetId` (la pestaña donde el usuario hizo clic derecho).
  - En caso contrario, preserva `currentActiveId`.

### 2.3 Integración en `app/src/main.ts`
- En `openTerminalTab`, se registra `tabEl.addEventListener("contextmenu", ...)`:
  - Invoca `showContextMenu(ev.clientX, ev.clientY, items)`.
  - Si `action === "close"`, delega directamente en `closeTerminalSession(terminalId)`.
  - Si `action === "close-all"`, delega en `closeAllTerminals()`.
  - Si `action` es `"close-left"`, `"close-right"` o `"close-others"`:
    1. Obtiene `idsToClose = resolveTabsToClose(orderedIds, terminalId, action)`.
    2. Cuenta cuántas de `idsToClose` tienen al menos un `pane.isConnected`.
    3. Si `connectedCount > 0`, muestra un único `confirmDialog` consolidado con `buildBulkTabCloseConfirm(...)`. Si el usuario cancela, aborta sin tocar ninguna pestaña.
    4. Calcula `desiredActiveId = resolveNextActiveTabAfterBulkClose(currentActiveTerminalId, terminalId, idsToClose)`.
    5. Ejecuta secuencialmente `await closeTerminalSession(id, true)` para cada `id` en `idsToClose`.
    6. Si al finalizar `desiredActiveId` sigue en `activeTerminals` y `currentActiveTerminalId !== desiredActiveId`, llama a `switchActiveTerminal(desiredActiveId)` para garantizar que la pestaña clicada conserve el foco cuando la activa previa fue cerrada.
