## ADDED Requirements

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

## MODIFIED Requirements

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
