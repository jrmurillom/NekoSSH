## ADDED Requirements

### Requirement: Higiene de Memoria por Sesión SSH, Límite Dinámico de Captura y Puente Unificado de Planes
El sistema SHALL liberar por completo la máquina de estados `HarnessRegistry` al cerrar una sesión SSH, respetar dinámicamente la preferencia `context_max_lines` al recortar salidas capturadas por el centinela en `PtyTapRegistry` y presentar de forma accionable cualquier respuesta `ProposePlan` en `"neko-ai"` sin bloquear al operador.

#### Scenario: Liberación integral al cerrar sesión SSH
- **WHEN** el operador cierra una pestaña o shell mediante `close_ssh_session`
- **THEN** el backend SHALL cancelar turnos activos, remover el colector en `PtyTapRegistry`, limpiar el historial visual/HITL en `AiUiFeedRegistry` y eliminar la instancia en `HarnessRegistry::remove`

#### Scenario: Recorte de salida de centinela ajustado a context_max_lines
- **WHEN** el operador ajusta `context_max_lines` en Ajustes (rango `50–1000`, ej. `400`) y se completa un comando con centinela cuya salida supera ese umbral
- **THEN** `PtyTapRegistry` SHALL aplicar la estrategia `Head + Tail` usando el valor configurado de `context_max_lines` en lugar de un límite fijo de `200`

#### Scenario: Presentación accionable ante respuesta ProposePlan en vista conversacional
- **WHEN** el backend retorna una respuesta `ProposePlan` con una lista de tareas no vacía
- **THEN** `"neko-ai"` SHALL renderizar el resumen legible en Markdown y cargar el primer paso en la tarjeta de autorización pendiente para que el operador pueda revisarlo, editarlo, autorizarlo u omitirlo sin quedar atrapado sin botones
