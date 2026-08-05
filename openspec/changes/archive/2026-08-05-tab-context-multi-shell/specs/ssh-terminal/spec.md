## ADDED Requirements

### Requirement: Cierre y reconexión a nivel de contexto
Al cerrar una pestaña de terminal, el sistema SHALL liberar la Session del shell padre y las de todos los shells hijos del contexto. La reconexión con Ctrl+R en un contexto desconectado SHALL reestablecer el shell padre (mismo perfil y `terminal_id` de padre); los hijos previos MUST NOT restaurarse automáticamente tras la reconexión del padre.

#### Scenario: Cerrar pestaña libera todas las Sessions del contexto
- **WHEN** el usuario confirma el cierre de una pestaña con padre e hijos conectados
- **THEN** el backend cierra todos los `terminal_id` del contexto antes de remover la pestaña

#### Scenario: Reconectar contexto sin revivir hijos
- **WHEN** el contexto está desconectado tras caída del padre y el usuario pulsa Ctrl+R
- **THEN** el sistema reconecta solo el padre y el layout vuelve a una celda (sin recrear hijos automáticamente)

#### Scenario: Confirmación al cerrar pestaña con cualquier Session viva
- **WHEN** el usuario intenta cerrar la pestaña y el padre o algún hijo tiene `isConnected === true`
- **THEN** el sistema MUST mostrar el diálogo de confirmación glass antes de desconectar el contexto
