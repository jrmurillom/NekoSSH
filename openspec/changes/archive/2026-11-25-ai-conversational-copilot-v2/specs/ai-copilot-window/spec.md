## MODIFIED Requirements

### Requirement: Colector Opt-In de Terminal y Centinela Silencioso de Lectura (PtyContextTap)
El backend SHALL proveer `PtyContextTap` para capturar contexto limpio de consola exclusivamente cuando `Escucha` esté en `ON`, incluyendo sembrado inmediato del buffer visible actual de `xterm.js` al encender la escucha, corredor unificado de centinela POSIX silencioso inmune al eco TTY (`%s_%s:%d__`) y purga inmediata a `0 bytes` al pasar a `OFF`.

#### Scenario: Sembrado instantáneo de pantalla visible al activar Escucha ON
- **WHEN** el operador activa `Escucha ON` sobre una terminal conectada que ya muestra líneas o errores previos en `xterm.js`
- **THEN** la ventana `"main"` SHALL sincronizar las últimas líneas visibles limpias del buffer activo hacia `PtyTapRegistry` (sanitizadas por `SecretRedactor` al enviarse al modelo), permitiendo consultar de inmediato sobre lo que ya está en pantalla sin repetir comandos

#### Scenario: Cero captura en modo privacidad y purga al apagar escucha
- **WHEN** `Escucha` está en `OFF` (valor inicial) o pasa de `ON` a `OFF`
- **THEN** `PtyContextTap` SHALL retener `0 bytes` del flujo `ssh-stdout` y purgar de inmediato cualquier línea sembrada o previamente almacenada para esa sesión

#### Scenario: Limpieza ANSI, retorno de carro, pantallas alternas y presupuesto Head+Tail
- **WHEN** `Escucha` está en `ON` y el servidor emite códigos CSI/OSC ANSI, barras de progreso con `\r`, pantallas alternas (`\x1b[?1049h`...`\x1b[?1049l`) o más líneas que `context_max_lines`
- **THEN** `PtyContextTap` SHALL descartar bloques de pantalla alterna, resolver `\r` sobre la línea en curso, limpiar secuencias de escape y recortar deterministamente mediante estrategia `Head + Tail`

#### Scenario: Centinela POSIX inmune a eco TTY en mismo paquete TCP y commit en backend
- **WHEN** se inyecta un comando supervisado tras clic humano (o auto-lectura `🟢` en `Modo Flexible`) y el servidor devuelve tanto el eco TTY como la salida del comando en uno o varios paquetes TCP
- **THEN** el wrapper SHALL separar el prefijo con `%s` (`printf '\n%s_%s:%d__\n' '__NEKO_END' '<nonce>' "$__neko_rc"`), ocultar la línea del centinela y su eco antes de `xterm.js`, actualizar atómicamente `HarnessRegistry` en el backend aunque `"neko-ai"` esté cerrada y emitir `ai-pty-command-finished` con el `exit_code`, la salida limpia y el propósito completado

---

### Requirement: Barra Rápida Operativa y Flujo Completo de Fases 1 y 2
La ventana `"neko-ai"` SHALL presentar controles directos para Shell + Pin, `Escucha ON/OFF` armonizado con los tokens del tema, Modo (`Estricto` / `Flexible`), `<select>` de Modelo y botón de **Limpiar conversación**, junto con un indicador de las **4 Fases Operativas** (`1. Diagnóstico · 2. Propuesta · 3. Ejecución · 4. Verificación`).

#### Scenario: Auto-ejecución de lecturas en Modo Flexible vs autorización en Modo Estricto
- **WHEN** en Fase 1 el orquestador retorna `ProposeStep` de lectura pura (`ReadOnly`)
- **THEN** en modo `Flexible` (`auto_approved_read_only == true`) SHALL inyectarse y recolectarse automáticamente hasta alcanzar el diagnóstico/plan, mientras que en modo `Estricto` SHALL requerir clic explícito en **Autorizar Lectura** antes de inyectar en el PTY

#### Scenario: Debate de ruta y aprobación humana obligatoria del Plan en ambos modos
- **WHEN** el agente presenta un plan estructurado en Fase 2 (`PlanAndDebate`)
- **THEN** la interfaz SHALL mostrar las tareas con su severidad auditada, `backup_strategy` y `rollback_cmd`, permitir enviar cuestionamientos para recalcular el plan sin tocar el servidor y exigir siempre clic explícito en **Aprobar Plan** tanto en modo `Estricto` como en modo `Flexible`

#### Scenario: Indicador vivo de 4 fases y limpieza voluntaria de chat
- **WHEN** el agente avanza entre diagnóstico, propuesta, ejecución progresiva o verificación post-cambio
- **THEN** el indicador de fases SHALL reflejar la etapa activa sin bloquear preguntas libres del operador, y el botón **Limpiar conversación** SHALL reiniciar el historial únicamente cuando el usuario lo solicite

---

### Requirement: Vista Interactiva de Fase 3, Freno de Error y Rollback en Ventana neko-ai
La ventana `"neko-ai"` SHALL ofrecer una experiencia visual de nivel comercial centrada y legible, con renderizador Markdown técnico propio seguro contra XSS (encabezados, listas, negritas, tablas y bloques de código con botón **Copiar**), bloques de salida de consola **colapsables** con insignia `exit=0` / `exit!=0`, chips rápidos de inicio, compositor `<textarea>` multilínea (`Enter` envía, `Shift+Enter` nueva línea, `autocomplete="off"`) y tarjetas HITL explicativas libres de *neon glow*, cubriendo además los flujos de `GuidedExecution`, `HaltedOnError` y `Completed`.

#### Scenario: Renderizado de checklist vivo, editor inline y control Ctrl+C en GuidedExecution
- **WHEN** la sesión objetivo se encuentra en `GuidedExecution`
- **THEN** `"neko-ai"` SHALL mostrar el progreso `[i/N]`, el checklist de tareas (indicando pendientes, `[✓]` completadas y `[↺]` revertidas con acción de reversión bajo confirmación), el campo inline editable del comando actual con re-auditoría de riesgo y el botón **Interrumpir (Ctrl+C)** mientras el centinela esté en vuelo

#### Scenario: Tarjeta de Freno de Emergencia (HaltedOnError) y Resumen Completed
- **WHEN** un paso falla pasando a `HaltedOnError` o el último paso `[N/N]` concluye en `Completed`
- **THEN** en `HaltedOnError` `"neko-ai"` SHALL resaltar el `exit_code` y salida del error ofreciendo **Pedir corrección a IA**, **Editar y Reintentar** o **Ejecutar Rollback**, mientras que en `Completed` SHALL presentar el cierre exitoso de las `N` tareas con botón para reiniciar o abrir una nueva consulta

#### Scenario: Renderizado rico de Markdown técnico, tablas y salidas colapsables
- **WHEN** el copiloto entrega un diagnóstico con tablas/listas/código o finaliza un comando con decenas de líneas de consola
- **THEN** `"neko-ai"` SHALL escapar primero cualquier HTML hostil, renderizar las tablas, listas y bloques copiables, y agrupar la salida de consola en un acordeón limpio mostrando su código de salida

#### Scenario: Compositor multilínea y chips rápidos de diagnóstico inicial
- **WHEN** el operador abre una conversación nueva o redacta instrucciones de varias líneas
- **THEN** `"neko-ai"` SHALL mostrar atajos rápidos de 1 clic (*Radiografía del servidor*, *Servicios y logs con error*, *Explicar pantalla actual*) y permitir enviar con `Enter` o insertar salto de línea con `Shift+Enter`
