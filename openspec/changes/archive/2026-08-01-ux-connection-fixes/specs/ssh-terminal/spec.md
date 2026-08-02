## MODIFIED Requirements

### Requirement: Cierre de conexiones con pestaña y aplicación
El sistema SHALL liberar de forma ordenada la Session SSH asociada a un `terminal_id` cuando el usuario cierra esa pestaña de terminal, y SHALL liberar todas las Sessions SSH activas cuando se cierra la ventana o se sale de la aplicación. Al intentar cerrar una pestaña con sesión viva (`isConnected === true`), el sistema MUST solicitar confirmación mediante un diálogo de confirmación antes de proceder a la desconexión. Al ejecutar "Cerrar Todo", si existen conexiones vivas, el sistema MUST solicitar confirmación global una sola vez antes de cerrar todas las sesiones.

#### Scenario: Confirmación al cerrar pestaña individual con sesión viva
- **WHEN** el usuario hace clic en el botón de cerrar (`x`) en una pestaña de terminal que tiene una conexión SSH activa (`isConnected === true`)
- **THEN** el sistema despliega el diálogo de confirmación glass. Si el usuario confirma, la sesión se libera y la pestaña se remueve; si cancela, la terminal permanece abierta

#### Scenario: Cerrar pestaña desconectada sin confirmación
- **WHEN** el usuario hace clic en el botón de cerrar (`x`) en una pestaña que ya está desconectada (`isConnected === false`)
- **THEN** la pestaña se cierra de inmediato sin solicitar confirmación

#### Scenario: Confirmación al cerrar todas las terminales
- **WHEN** el usuario activa la acción "Cerrar Todo" teniendo una o más terminales activas conectadas
- **THEN** el sistema presenta un único diálogo de confirmación global antes de cerrar todas las pestañas
