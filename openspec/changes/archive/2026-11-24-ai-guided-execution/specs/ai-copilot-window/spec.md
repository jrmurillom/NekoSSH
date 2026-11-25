## MODIFIED Requirements

### Requirement: Colector Opt-In de Terminal y Centinela Silencioso de Lectura (PtyContextTap)
El backend SHALL proveer `PtyContextTap` para capturar contexto limpio de consola exclusivamente cuando el operador habilite la escucha en un `terminal_id`, incluyendo un corredor unificado de centinela POSIX silencioso inmune al eco TTY para lecturas de Fase 1 (`ReadOnlyRecon`), pasos del plan de Fase 3 (`PlanStep`) y reversiones (`StepRollback`), sincronizando atómicamente `HarnessRegistry` en backend al completarse el marcador.

#### Scenario: Cero captura en modo privacidad y purga al apagar escucha
- **WHEN** `Escucha` está en `OFF` (valor inicial) o pasa de `ON` a `OFF`
- **THEN** `PtyContextTap` SHALL retener `0 bytes` del flujo `ssh-stdout` y purgar de inmediato cualquier línea previamente almacenada para esa sesión

#### Scenario: Limpieza ANSI, retorno de carro, pantallas alternas y presupuesto Head+Tail
- **WHEN** `Escucha` está en `ON` y el servidor emite códigos CSI/OSC ANSI, barras de progreso con `\r`, pantallas alternas (`\x1b[?1049h`...`\x1b[?1049l`) o más líneas que `context_max_lines`
- **THEN** `PtyContextTap` SHALL descartar bloques de pantalla alterna, resolver `\r` sobre la línea en curso, limpiar secuencias de escape y recortar deterministamente mediante estrategia `Head + Tail`

#### Scenario: Centinela POSIX inmune a eco TTY en mismo paquete TCP y commit en backend
- **WHEN** se inyecta un comando supervisado en Fase 1 o Fase 3 y el servidor devuelve tanto el eco TTY como la salida del comando en uno o varios paquetes TCP
- **THEN** el wrapper SHALL separar el prefijo con `%s` (`printf '\n%s_%s:%d__\n' '__NEKO_END' '<nonce>' "$__neko_rc"`), ocultar la línea del centinela antes de `xterm.js`, actualizar atómicamente `HarnessRegistry` en el backend aunque `"neko-ai"` esté cerrada y emitir `ai-pty-command-finished` con el `exit_code`, la salida limpia y el propósito completado

## ADDED Requirements

### Requirement: Vista Interactiva de Fase 3, Freno de Error y Rollback en Ventana neko-ai
La ventana `"neko-ai"` SHALL renderizar el flujo completo de Fase 3 (`GuidedExecution`, `HaltedOnError` y `Completed`) con estética Cyber-Sakura libre de *neon glow*, iconos Lucide outline (`currentColor`), escapado anti-XSS y `autocomplete="off"` en todo control editable.

#### Scenario: Renderizado de checklist vivo, editor inline y control Ctrl+C en GuidedExecution
- **WHEN** la sesión objetivo se encuentra en `GuidedExecution`
- **THEN** `"neko-ai"` SHALL mostrar el progreso `[i/N]`, el checklist de tareas (indicando pendientes, `[✓]` completadas y `[↺]` revertidas con acción de reversión bajo confirmación), el campo inline editable del comando actual con re-auditoría de riesgo y el botón **Interrumpir (Ctrl+C)** mientras el centinela esté en vuelo

#### Scenario: Tarjeta de Freno de Emergencia (HaltedOnError) y Resumen Completed
- **WHEN** un paso falla pasando a `HaltedOnError` o el último paso `[N/N]` concluye en `Completed`
- **THEN** en `HaltedOnError` `"neko-ai"` SHALL resaltar el `exit_code` y salida del error ofreciendo **Pedir corrección a IA**, **Editar y Reintentar** o **Ejecutar Rollback**, mientras que en `Completed` SHALL presentar el cierre exitoso de las `N` tareas con botón para reiniciar o abrir una nueva consulta
