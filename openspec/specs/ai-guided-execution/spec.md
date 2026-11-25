# ai-guided-execution Specification

## Purpose
Ejecución guiada paso a paso de Fase 3 (`GuidedExecution`) sobre el PTY real con autorización humana obligatoria en todos los pasos `[1..N]`, wrapper POSIX inmune a eco TTY y comentarios, edición inline con re-auditoría de riesgo, interrupción `Ctrl+C`, freno de emergencia automático (`HaltedOnError`), rollback quirúrgico LIFO con bandera `is_rolled_back` y cierre determinista en `Completed`.
## Requirements
### Requirement: Ejecución Guiada Paso a Paso con Autorización Humana Universal
El copiloto SHALL operar bajo ejecución progresiva de **un único comando propuesto por turno** (y conservar la autorización humana paso a paso en `GuidedExecution`), alimentándose del `exit_code` y salida real capturada antes de formular el siguiente paso. En `Modo Flexible`, únicamente los comandos `🟢` verificados como lectura inocua (en Fase 1 de diagnóstico o Fase 4 de verificación) SHALL ejecutarse automáticamente (hasta el tope de seguridad por ráfaga); en `Modo Estricto`, o ante **cualquier** comando mutante o con alerta de auditoría (`🟡 / 🟠 / 🔴`) en ambos modos, el sistema SHALL pausar y presentar una tarjeta explicativa solicitando autorización explícita del operador.

#### Scenario: Bloqueo de auto-ejecución en todos los pasos de Fase 3
- **WHEN** un plan es aprobado y el arnés entra en `GuidedExecution` en `Modo Estricto` o en `Modo Flexible` (incluso si la tarea actual tiene `risk_level = read_only`)
- **THEN** el sistema SHALL presentar la tarjeta del paso `[i/N]` en espera y NO inyectará ningún comando en el PTY hasta que el operador accione explícitamente **Autorizar y Ejecutar**

#### Scenario: Inyección con centinela POSIX inmune a eco TTY y bloqueo de colisión
- **WHEN** el operador autoriza el paso `[i/N]` en un `terminal_id` conectado sin centinela previo en vuelo
- **THEN** el backend SHALL armar un centinela tipado e inyectar el comando envuelto con `;__neko_rc=$?;printf '\n%s_%s:%d__\n' '__NEKO_END' '<nonce>' "$__neko_rc"`, rechazando cualquier segundo disparo concurrente hasta que el centinela activo concluya o sea interrumpido

#### Scenario: Tarjeta explicativa clara con motivo, alerta de seguridad y acciones rápidas
- **WHEN** una propuesta de comando requiere confirmación humana (por estar en `Modo Estricto` o por tener severidad `🟡/🟠/🔴` o `audit_warning`)
- **THEN** `"neko-ai"` SHALL mostrar la pregunta natural *"¿Autorizas ejecutar este comando para `<propósito>`?"*, el semáforo auditado, el banner `audit_warning` (si aplica), el respaldo/rollback declarado y los botones **Autorizar y Ejecutar**, **Editar**, **Copiar** y **Omitir**

#### Scenario: Soberanía de la autorización humana al hacer clic
- **WHEN** el operador revisa (y opcionalmente edita inline) el comando en la tarjeta y pulsa **Autorizar y Ejecutar**
- **THEN** el backend SHALL inyectar el comando envuelto en el centinela POSIX sin rechazarlo por listas blancas rígidas, habilitando **Interrumpir (Ctrl+C)** mientras está en vuelo y devolviendo su salida real al copiloto al concluir

---

### Requirement: Edición Inline del Comando con Re-Auditoría de Riesgo
Antes de ejecutar el paso actual (en `GuidedExecution`) o antes de reintentarlo/revertirlo (en `HaltedOnError`), el sistema SHALL permitir al operador editar inline el comando propuesto, recalculando deterministamente su nivel de riesgo mediante `CommandRiskGuard`.

#### Scenario: Elevación de riesgo en vivo al editar comando inline
- **WHEN** el operador modifica el texto del comando del paso actual introduciendo operadores o verbos de mayor severidad (por ejemplo agregando una redirección `>`, `systemctl restart` o `| bash`)
- **THEN** la interfaz SHALL actualizar la insignia visual de severidad y el backend SHALL re-auditar y persistir el comando junto con su `risk_level` recalculado en `tasks[current_step_index]` antes de enviarlo al PTY

---

### Requirement: Interrupción en Vivo (Ctrl+C) y Desarme Determinista de Centinela
Mientras exista un comando del arnés en ejecución esperando su centinela en el PTY, el sistema SHALL proveer un control directo de interrupción (`Ctrl+C`) desde `"neko-ai"` que aborte el comando sin dejar la interfaz ni la FSM bloqueadas esperando un marcador cancelado por `SIGINT`.

#### Scenario: Envío de SIGINT y freno inmediato a HaltedOnError
- **WHEN** el operador pulsa **Interrumpir (Ctrl+C)** mientras corre un paso de Fase 3
- **THEN** el backend SHALL escribir `\x03` en el PTY correspondiente, limpiar inmediatamente `active_sentinel` en `PtyTapRegistry` y transicionar la FSM a `HaltedOnError` con `exit_code = 130` conservando el índice y el resto del plan

---

### Requirement: Freno de Emergencia (HaltedOnError), Diagnóstico y Rollback Quirúrgico por Paso
Cuando cualquier comando ejecutado devuelva `exit_code != 0` (o sea interrumpido con `Ctrl+C`), el sistema SHALL detener toda progresión automática, resaltar el fallo con su salida real, conservar accesible con **1 clic** el `rollback_cmd` asociado al cambio previo (editable antes de confirmar) y permitir que el copiloto diagnostique el error real para proponer la corrección puntual.

#### Scenario: Freno automático y congelamiento de pasos restantes
- **WHEN** el centinela de `PlanStep` reporta `exit_code != 0` en el paso `k`
- **THEN** el hilo lector del backend SHALL transicionar atómicamente `HarnessRegistry` a `HaltedOnError` registrando `failed_command`, `exit_code` y `captured_output` sanitizado, manteniendo intactos los pasos `k+1..N` sin ejecutarlos

#### Scenario: Asistencia de IA y reanudación sobre el paso fallido
- **WHEN** en `HaltedOnError` el operador consulta al agente por el fallo o edita el comando y confirma **Reintentar Paso**
- **THEN** el sistema SHALL invocar `resume_from_error(fixed_command)` re-auditando el riesgo del comando corregido y devolviendo la FSM a `GuidedExecution` en el mismo `step_index` lista para autorización humana

#### Scenario: Ejecución controlada de rollback del paso fallido y de pasos previos en orden inverso
- **WHEN** el operador solicita revertir el paso fallido `k` o un paso previo completado `j < k` (`is_done == true` y `is_rolled_back == false`)
- **THEN** `"neko-ai"` SHALL mostrar el `rollback_cmd` correspondiente (permitiendo ajuste inline y priorizando el último paso completado aún no revertido en orden LIFO), inyectarlo con centinela `StepRollback { step_index }` tras confirmación humana y, si devuelve `exit_code == 0`, marcar `tasks[idx].is_rolled_back = true` (`[↺ Revertido]`) impidiendo ejecuciones duplicadas accidentales

#### Scenario: Freno inmediato ante exit_code distinto de cero y rescate con rollback_cmd
- **WHEN** un comando devuelve `exit_code != 0` durante una intervención
- **THEN** el sistema SHALL exponer la alerta de fallo junto con el botón **Preparar Rollback** (si existe `rollback_cmd` registrado en el paso o turno reciente) y enviar/permitir analizar el output de error real para corregir antes de continuar

### Requirement: Cierre Determinista en Completed al Finalizar [N/N]
Cuando el último paso `[N/N]` del plan concluya con `exit_code == 0`, el sistema SHALL cerrar la misión directamente en `Completed` sin realizar llamadas automáticas adicionales al proveedor de IA.

#### Scenario: Finalización instantánea sin consumo extra de tokens
- **WHEN** el centinela del paso `N` (última tarea del plan) reporta `exit_code == 0`
- **THEN** el backend SHALL marcar `tasks[N-1].is_done = true`, transicionar `AgentHarnessFsm` a `Completed { objective, completed_tasks_count: N }` sin invocar `AiProviderDriver::execute_turn` y presentar en `"neko-ai"` el resumen completo de tareas ejecutadas con opción voluntaria de consultar o iniciar nueva misión

