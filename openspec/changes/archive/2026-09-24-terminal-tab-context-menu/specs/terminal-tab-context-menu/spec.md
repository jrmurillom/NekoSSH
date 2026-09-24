# Specification: `terminal-tab-context-menu`

## ADDED Requirements

### Requirement: VS Code Style Terminal Tab Context Menu with Disabled Boundary Items
Every terminal tab (`.term-tab` in `#terminal-tabs-list`) SHALL open a custom B3 context menu (`showContextMenu`) on right-click (`contextmenu` event) displaying the standard tab management actions in fixed order: `"Cerrar pestaña"` (`close`), `"Cerrar otras pestañas"` (`close-others`), `"Cerrar pestañas a la izquierda"` (`close-left`), `"Cerrar pestañas a la derecha"` (`close-right`), and `"Cerrar todas las pestañas"` (`close-all`). Directional or multi-tab options that have zero applicable target tabs SHALL remain visible in the menu with `disabled: true` (`.is-disabled`, `aria-disabled="true"`) and SHALL NOT be clickable.

#### Scenario: Right-clicking the leftmost tab disables "Cerrar pestañas a la izquierda"
- **GIVEN** three open terminal tabs ordered `[tab-1, tab-2, tab-3]`
- **WHEN** the user right-clicks `tab-1` (index `0`)
- **THEN** the context menu renders `"Cerrar pestañas a la izquierda"` with `disabled: true` (`.is-disabled`)
- **AND** `"Cerrar pestañas a la derecha"`, `"Cerrar otras pestañas"`, `"Cerrar pestaña"`, and `"Cerrar todas las pestañas"` are enabled (`disabled: false`).

#### Scenario: Right-clicking the rightmost tab disables "Cerrar pestañas a la derecha"
- **GIVEN** three open terminal tabs ordered `[tab-1, tab-2, tab-3]`
- **WHEN** the user right-clicks `tab-3` (index `2`)
- **THEN** the context menu renders `"Cerrar pestañas a la derecha"` with `disabled: true` (`.is-disabled`)
- **AND** `"Cerrar pestañas a la izquierda"`, `"Cerrar otras pestañas"`, `"Cerrar pestaña"`, and `"Cerrar todas las pestañas"` are enabled (`disabled: false`).

#### Scenario: Right-clicking the only open tab disables directional and "other" actions
- **GIVEN** a single open terminal tab `[tab-1]`
- **WHEN** the user right-clicks `tab-1`
- **THEN** `"Cerrar otras pestañas"`, `"Cerrar pestañas a la izquierda"`, and `"Cerrar pestañas a la derecha"` are rendered with `disabled: true`
- **AND** only `"Cerrar pestaña"` and `"Cerrar todas las pestañas"` remain enabled.

---

### Requirement: Consolidated Confirmation Dialog for Bulk Tab Closure with Active SSH Sessions
When the user selects `"Cerrar pestañas a la izquierda"`, `"Cerrar pestañas a la derecha"`, `"Cerrar otras pestañas"`, or `"Cerrar todas las pestañas"`, the application SHALL inspect only the subset of tabs targeted for closure. If one or more tabs in that subset have an active SSH connection (`isConnected`), the application SHALL display a single consolidated `confirmDialog` indicating the number of connected sessions and tabs to be closed before disconnecting any session. If the user cancels the dialog, zero tabs SHALL be closed.

#### Scenario: Closing tabs to the right with active SSH sessions prompts once and closes sequentially
- **GIVEN** four open terminal tabs `[tab-1, tab-2, tab-3, tab-4]` where `tab-3` and `tab-4` have active SSH connections
- **WHEN** the user right-clicks `tab-2` and selects `"Cerrar pestañas a la derecha"`
- **THEN** a single `confirmDialog` is displayed warning that `2` active SSH sessions to the right will be disconnected
- **AND** when the user confirms, `tab-3` and `tab-4` are closed sequentially without prompting again per tab, leaving `[tab-1, tab-2]` intact.

#### Scenario: Cancelling the consolidated confirmation leaves all tabs untouched
- **GIVEN** four open terminal tabs `[tab-1, tab-2, tab-3, tab-4]` where `tab-1` is connected
- **WHEN** the user right-clicks `tab-3`, selects `"Cerrar pestañas a la izquierda"`, and clicks `"Cancelar"` in the confirmation dialog
- **THEN** neither `tab-1` nor `tab-2` is closed and all SSH sessions remain active.

---

### Requirement: Deterministic Active Tab Focus Resolution After Directional or Bulk Closure
When a directional or `"Cerrar otras pestañas"` closure removes the currently active terminal tab (`currentActiveTerminalId`), the application SHALL automatically switch focus (`switchActiveTerminal`) to the clicked tab (`targetTerminalId`) on which the context menu was invoked. If the previously active tab was not among the closed tabs, it SHALL remain active.

#### Scenario: Active tab was to the right of the clicked tab and gets closed
- **GIVEN** open tabs `[tab-1, tab-2, tab-3]` where `tab-3` is currently active (`currentActiveTerminalId === "tab-3"`)
- **WHEN** the user right-clicks `tab-2` and executes `"Cerrar pestañas a la derecha"`
- **THEN** `tab-3` is closed and `tab-2` automatically becomes the active terminal tab (`currentActiveTerminalId === "tab-2"`).

#### Scenario: Active tab was to the left of the clicked tab and is preserved when closing to the right
- **GIVEN** open tabs `[tab-1, tab-2, tab-3]` where `tab-1` is currently active (`currentActiveTerminalId === "tab-1"`)
- **WHEN** the user right-clicks `tab-2` and executes `"Cerrar pestañas a la derecha"`
- **THEN** `tab-3` is closed and `tab-1` remains the active terminal tab (`currentActiveTerminalId === "tab-1"`).
