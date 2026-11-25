## MODIFIED Requirements

### Requirement: Cancelación Cooperativa, Timeout y Ejecución Sin Bloqueo de Mutex
El orquestador y `AiProviderDriver` SHALL permitir abortar cualquier turno activo de inmediato y NUNCA retener el `Mutex` global de `HarnessRegistry` mientras espera la respuesta del proveedor, garantizando la terminación completa del árbol de procesos hijo (incluyendo subprocesos lanzados mediante `.cmd`/`.bat` en Windows) y el retorno inmediato sin bloqueo sobre descriptores de salida.

#### Scenario: Consulta concurrente y cancelación inmediata de turno en vuelo
- **WHEN** un turno de IA se encuentra en ejecución y el cliente consulta el estado de la sesión o invoca `cancel_ai_turn_cmd` (o vence `--print-timeout` / timeout interno)
- **THEN** `HarnessRegistry` SHALL responder sin bloqueo de mutex, señalizar el `TurnCancellationToken`, terminar recursivamente el árbol de procesos del driver (`taskkill /F /T` en Windows y terminación del proceso en sistemas Unix), retornar `Cancelled` / `Timeout` de inmediato sin colgar hilos en `join()` sobre pipes abiertos y descartar cualquier escritura tardía mediante control de época/generación

#### Scenario: Limpieza global al cerrar la aplicación
- **WHEN** la aplicación recibe evento de salida (`RunEvent::ExitRequested` / `RunEvent::Exit`)
- **THEN** `AiTurnCancelRegistry::cancel_all` SHALL abortar todos los turnos activos evitando procesos huérfanos
