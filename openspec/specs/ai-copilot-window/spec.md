# ai-copilot-window Specification

## Purpose
Ventana nativa independiente `"neko-ai"` multi-monitor, colector opt-in `PtyContextTap` con centinela silencioso de lectura POSIX, catálogo multi-shell con *Pin Shell*, sincronización de temas y vistas de Reconocimiento (Fase 1), Plan/Debate (Fase 2) y Ajustes del Agente.
## Requirements
### Requirement: Ventana Nativa Independiente Multi-Monitor y Ciclo de Vida (neko-ai)
El sistema SHALL abrir y gestionar el asistente agéntico en una ventana nativa independiente del sistema operativo (`WebviewWindow` con etiqueta `"neko-ai"`), compilada como segunda entrada en Vite (`ai-copilot.html`) y gobernada por una *capability* aislada (`capabilities/ai-copilot.json`).

#### Scenario: Instancia única y enfoque al invocar repetidamente
- **WHEN** el usuario acciona el botón de IA o presiona `Ctrl+Shift+A` estando la ventana `"neko-ai"` ya abierta o minimizada en otro monitor
- **THEN** el sistema SHALL desminimizar y enfocar la ventana existente sin crear instancias duplicadas

#### Scenario: Fijación superior (Always on Top)
- **WHEN** el usuario alterna el control *Siempre visible* en la ventana `"neko-ai"`
- **THEN** el backend SHALL aplicar `set_always_on_top` sobre `"neko-ai"` reflejando su estado activo en la interfaz

#### Scenario: Cierre en cascada al destruir ventana principal y cierre aislado de neko-ai
- **WHEN** el usuario cierra la ventana principal `"main"`
- **THEN** el backend SHALL cancelar cualquier turno activo de IA, cerrar automáticamente `"neko-ai"` y finalizar las sesiones SSH; por el contrario, cuando se destruye únicamente la ventana `"neko-ai"`, el backend SHALL mantener intactas `"main"` y las sesiones SSH, cancelar cualquier turno `agy` activo, desactivar `Escucha` (`OFF`) purgando buffers en `PtyTapRegistry`, emitir `"ai-shells-updated"` y conservar la memoria de sesión del arnés y del feed visual en memoria RAM para recargarlos si el usuario reabre `"neko-ai"`

---

### Requirement: Punto de Entrada, Integridad Visual Cyber-Sakura y Catálogo Multi-Shell con Pin
La ventana `"main"` y la ventana `"neko-ai"` SHALL mantener paridad estricta con `DESIGN.md`, `ui-icons` y `base-standards.md`, sincronizando en tiempo real los 8 temas conceptuales y el catálogo de shells activos (`ShellPane` padres e hijos).

#### Scenario: Disparador accesible con Lucide outline y atajo Ctrl+Shift+A
- **WHEN** se renderiza `.footer-actions` en la ventana principal
- **THEN** SHALL mostrarse el botón disparador de IA con icono Lucide de contorno (`currentColor`, sin emojis), `title`/`aria-label` en español latino y soporte global de `Ctrl+Shift+A` listado en el popover de atajos

#### Scenario: Sincronización reactiva de tema y cumplimiento de estándares frontend
- **WHEN** el usuario cambia de tema en `"main"` o interactúa con controles en `"neko-ai"`
- **THEN** `"neko-ai"` SHALL actualizar `data-theme` en vivo, aplicar acentos planos sin *neon glow*, scrollbars Cyber-Sakura, `autocomplete="off"` en todos los campos de entrada y escapado seguro anti-XSS sobre cualquier texto del modelo o terminal

#### Scenario: Seguimiento de foco y Fijación de Shell (Pin)
- **WHEN** el usuario cambia de pestaña o celda en `"main"` teniendo desactivado *Pin Shell* en `"neko-ai"`
- **THEN** `"neko-ai"` SHALL apuntar automáticamente al `focusedTerminalId` actual; si *Pin Shell* está activo, `"neko-ai"` SHALL permanecer anclada al `terminal_id` fijado hasta que el usuario lo desbloquee o ese shell se cierre

---

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
La ventana `"neko-ai"` SHALL proveer controles directos y visibles en todo momento para Shell + Pin (en la cabecera compacta) y para `Escucha ON/OFF`, Modo (`Estricto` / `Flexible`) y `<select>` dinámico de Modelo (integrados en el dock compositor inferior), gobernando el flujo interactivo progresivo de Diagnóstico, Propuesta, Ejecución y Verificación.

#### Scenario: Auto-ejecución de lecturas en Modo Flexible vs autorización en Modo Estricto
- **WHEN** en Fase 1 el orquestador retorna `ProposeStep` de lectura pura (`ReadOnly`)
- **THEN** en modo `Flexible` (`auto_approved_read_only == true`) SHALL inyectarse y recolectarse automáticamente hasta alcanzar el diagnóstico/plan, mientras que en modo `Estricto` SHALL requerir clic explícito en **Autorizar Lectura** antes de inyectar en el PTY

#### Scenario: Debate de ruta y aprobación humana obligatoria del Plan en ambos modos
- **WHEN** el agente presenta un plan estructurado en Fase 2 (`PlanAndDebate`)
- **THEN** la interfaz SHALL mostrar las tareas con su severidad auditada, `backup_strategy` y `rollback_cmd`, permitir enviar cuestionamientos para recalcular el plan sin tocar el servidor y exigir siempre clic explícito en **Aprobar Plan** tanto en modo `Estricto` como en modo `Flexible`

#### Scenario: Indicador vivo de 4 fases y limpieza voluntaria de chat
- **WHEN** el agente avanza entre diagnóstico, propuesta, ejecución progresiva o verificación post-cambio
- **THEN** la píldora de etapa en la respuesta del asistente SHALL reflejar la etapa activa sin bloquear preguntas libres del operador, y el botón **Limpiar conversación** en cabecera SHALL reiniciar el historial y las tarjetas pendientes únicamente cuando el usuario lo solicite

### Requirement: Panel de Ajustes del Agente y Salud del Proveedor
`"neko-ai"` SHALL incluir una vista de ajustes que muestre el estado de `probe_ai_driver_health_cmd` y permita editar y restaurar las directivas de Capa 2 y opciones del proveedor en SQLite.

#### Scenario: Edición y restauración de directivas y parámetros
- **WHEN** el usuario guarda cambios en rol del operador, reglas de infraestructura, ruta opcional del binario o `context_max_lines`, o pulsa **Restaurar predeterminados**
- **THEN** los valores SHALL persistirse mediante los comandos IPC de preferencias y reflejarse inmediatamente en la barra rápida y siguientes turnos

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

### Requirement: Arquitectura Visual Limpia Estilo Copilot Pro en neko-ai
La ventana `"neko-ai"` SHALL reducir su chrome fijo a una sola barra superior compacta, un lienzo conversacional limpio sin bordes redundantes en las respuestas del asistente y un dock compositor inferior unificado, manteniendo fidelidad total con `DESIGN.md` (sin *neon glow*) y `base-standards.md`.

#### Scenario: Cabecera única y ausencia de barras apiladas redundantes
- **WHEN** se abre o renderiza la vista conversacional de `"neko-ai"`
- **THEN** la cabecera superior SHALL concentrar en una única franja (`~44px`) la marca `NekoAI`, el selector de shell objetivo con icono *Pin Shell* y los botones de *Limpiar conversación*, *Siempre visible* y *Ajustes*, sin renderizar una segunda toolbar superior ni una barra stepper separada de 4 pasos

#### Scenario: Lectura respirable y foco exclusivo en tarjetas de decisión
- **WHEN** se renderizan mensajes en el feed de `"neko-ai"` (incluyendo tablas Markdown, bloques de código, respuestas directas o estado vacío inicial)
- **THEN** las respuestas del asistente (`assistant`) SHALL mostrarse sin caja ni borde envolvente pesado mostrando su etapa operativa como etiqueta sutil en el encabezado, el estado vacío SHALL consolidarse en un único bloque hero centrado con accesos rápidos, y únicamente las burbujas del operador, errores, acordeones de consola y tarjetas de autorización/rescate/rollback SHALL presentar contorno delimitado

#### Scenario: Dock compositor unificado con controles de contexto y modelo
- **WHEN** el usuario interactúa con la zona inferior de entrada de `"neko-ai"`
- **THEN** el área de texto multilínea (`autocomplete="off"`) y la fila de controles operativos (`Escucha ON/OFF`, `Modo Estricto/Flexible`, selector dinámico de modelo, refresco de catálogo y botón `Enviar`) SHALL integrarse dentro de un mismo contenedor dock inferior que se adapte limpiamente tanto a `500px–520px` como a pantalla completa sin desbordamiento horizontal

### Requirement: Eco TTY Resiliente a ANSI, Detección de Espera Interactiva y Estado HITL por Sesión
El colector `PtyContextTap` y la ventana `"neko-ai"` SHALL limpiar el eco del wrapper centinela incluso cuando el shell remoto intercale secuencias ANSI de resaltado (p. ej. `zsh-syntax-highlighting`), advertir al operador cuando un comando supervisado quede esperando entrada interactiva o encuentre un shell no-POSIX (`fish`), y conservar por `terminal_id` toda tarjeta pendiente, fallo reciente y rollback preparado entre cambios de pestaña o reapertura de ventana.

#### Scenario: Supresión limpia de eco en zsh con resaltado ANSI intercalado
- **WHEN** el shell remoto devuelve el eco de `;__neko_rc=$?;printf '\n%s_%s:%d__\n' '__NEKO_END' '<nonce>' "$__neko_rc"` intercalado con códigos de color ANSI CSI (`\x1b[32m...`)
- **THEN** `strip_echo_plumbing` SHALL ignorar secuencias ANSI durante la detección de fronteras y ocultar íntegro el sufijo de eco antes de enviarlo a `xterm.js`

#### Scenario: Aviso en vivo cuando el comando espera entrada interactiva o encuentra shell fish
- **WHEN** mientras hay un centinela activo llegan patrones de espera interactiva (`[sudo] password`, `password:`, `passphrase`, `[y/N]`) o errores de sintaxis `fish`, o transcurre el umbral de espera en tarjeta activa
- **THEN** el backend SHALL emitir `ai-pty-waiting-input` una vez por centinela y `"neko-ai"` SHALL mostrar un aviso claro indicando que la terminal espera respuesta del operador (o pulsación de `Ctrl+C`)

#### Scenario: Preservación de tarjeta pendiente y rollback por terminal_id
- **WHEN** el operador tiene una tarjeta pendiente de autorización o un contexto de fallo/rollback preparado en el shell `A`, cambia momentáneamente al shell `B` (o cierra y reabre `"neko-ai"` en la misma ejecución de NekoSSH) y regresa al shell `A`
- **THEN** `"neko-ai"` SHALL restaurar exactamente la tarjeta pendiente, comando editado en borrador, contexto de error y rollback del shell `A` sin perder estado

---

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

---

### Requirement: Aceleradores de Usabilidad Comercial y Productividad en neko-ai y Main Terminal
La ventana principal `"main"` y la ventana independiente `"neko-ai"` SHALL proporcionar flujos de trabajo de alta velocidad sin comprometer la privacidad opt-in (`Escucha OFF`) ni la legibilidad minimalista del HUD.

#### Scenario: Preguntar a NekoAI sobre selección de terminal mediante Ctrl+Shift+E incluso con Escucha OFF
- **WHEN** el operador selecciona texto en una terminal conectada de `"main"` y presiona `Ctrl+Shift+E` (teniendo `Escucha` en `OFF` u `ON`)
- **THEN** el sistema SHALL abrir o enfocar `"neko-ai"` y precargar el texto seleccionado dentro de `#copilot-prompt-input` como bloque citado listo para editar o enviar, sin activar la escucha continua ni sufrir pérdida por carrera de inicialización de ventana

#### Scenario: Botón Insertar en terminal sin salto de línea en tarjeta de comando
- **WHEN** existe una tarjeta de comando pendiente en `"neko-ai"` y el operador hace clic en **Insertar en terminal**
- **THEN** `"neko-ai"` SHALL invocar `insert_ai_command_to_pty_cmd`, enviando el texto limpio al PTY activo de la sesión objetivo **sin** añadir `\n`, `\r` ni wrapper centinela para que el operador pueda completarlo o revisarlo antes de presionar `Enter`

#### Scenario: Vista previa de streaming limpia sin exponer JSON crudo
- **WHEN** el proveedor `agy` emite deltas incrementales del objeto `--json-schema` (`{"kind":"propose_step","message":"...`) durante `"Analizando con agy…"`
- **THEN** `#copilot-progress-stream` SHALL extraer y mostrar únicamente el texto legible del campo de respuesta/razonamiento (o `"Preparando respuesta técnica…"` mientras llegan metadatos JSON previos), evitando imprimir llaves JSON crudas en pantalla

#### Scenario: Indicador vivo de líneas capturadas, auto-ajuste de compositor e historial con flechas
- **WHEN** `Escucha` está en `ON` sobre el shell objetivo, el operador pega texto extenso en `#copilot-prompt-input`, o presiona `ArrowUp` / `ArrowDown` con el compositor vacío o navegando historial previo
- **THEN** la micro-etiqueta del chip SHALL mostrar `Escucha ON · N líneas`, el `<textarea>` SHALL auto-ajustar su altura suavemente hasta `140px`, y las teclas `ArrowUp` / `ArrowDown` SHALL recuperar prompts previos enviados en la sesión conservando cualquier borrador en curso


