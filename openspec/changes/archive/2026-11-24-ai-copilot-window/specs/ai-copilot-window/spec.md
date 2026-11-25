## ADDED Requirements

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
- **THEN** el backend SHALL cancelar cualquier turno activo de IA, cerrar automáticamente `"neko-ai"` y finalizar las sesiones SSH; por el contrario, cerrar únicamente `"neko-ai"` SHALL mantener intactas `"main"` y las sesiones SSH activas desactivando la escucha activa

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
El backend SHALL proveer `PtyContextTap` para capturar contexto limpio de consola exclusivamente cuando el operador habilite la escucha en un `terminal_id`, incluyendo un corredor de lecturas de Fase 1 con centinela POSIX silencioso.

#### Scenario: Cero captura en modo privacidad y purga al apagar escucha
- **WHEN** `Escucha` está en `OFF` (valor inicial) o pasa de `ON` a `OFF`
- **THEN** `PtyContextTap` SHALL retener `0 bytes` del flujo `ssh-stdout` y purgar de inmediato cualquier línea previamente almacenada para esa sesión

#### Scenario: Limpieza ANSI, retorno de carro, pantallas alternas y presupuesto Head+Tail
- **WHEN** `Escucha` está en `ON` y el servidor emite códigos CSI/OSC ANSI, barras de progreso con `\r`, pantallas alternas (`\x1b[?1049h`...`\x1b[?1049l`) o más líneas que `context_max_lines`
- **THEN** `PtyContextTap` SHALL descartar bloques de pantalla alterna, resolver `\r` sobre la línea en curso, limpiar secuencias de escape y recortar deterministamente mediante estrategia `Head + Tail`

#### Scenario: Ejecución de lectura con centinela POSIX oculto y retorno automático
- **WHEN** se ejecuta una lectura verificada por `ReadOnlyEnforcer` en Fase 1 (automáticamente en modo `Flexible` o tras clic en **Autorizar** en modo `Estricto`)
- **THEN** el backend SHALL anexar `; printf '\n__NEKO_END_%s:%d__\n' '<nonce>' "$?"`, filtrar dicho marcador antes de emitir `ssh-stdout` hacia `xterm.js` y, al detectarlo, entregar automáticamente la salida limpia y el `exit_code` para continuar el siguiente turno del arnés

---

### Requirement: Barra Rápida Operativa y Flujo Completo de Fases 1 y 2
La ventana `"neko-ai"` SHALL presentar en su barra superior controles directos para Shell + Pin, `Escucha ON/OFF`, Modo (`Estricto` / `Flexible`) y `<select>` dinámico de Modelo, gobernando el flujo interactivo de Reconocimiento (Fase 1) y Plan/Debate (Fase 2).

#### Scenario: Auto-ejecución de lecturas en Modo Flexible vs autorización en Modo Estricto
- **WHEN** en Fase 1 el orquestador retorna `ProposeStep` de lectura pura (`ReadOnly`)
- **THEN** en modo `Flexible` (`auto_approved_read_only == true`) SHALL inyectarse y recolectarse automáticamente hasta alcanzar el diagnóstico/plan, mientras que en modo `Estricto` SHALL requerir clic explícito en **Autorizar Lectura** antes de inyectar en el PTY

#### Scenario: Debate de ruta y aprobación humana obligatoria del Plan en ambos modos
- **WHEN** el agente presenta un plan estructurado en Fase 2 (`PlanAndDebate`)
- **THEN** la interfaz SHALL mostrar las tareas con su severidad auditada, `backup_strategy` y `rollback_cmd`, permitir enviar cuestionamientos para recalcular el plan sin tocar el servidor y exigir siempre clic explícito en **Aprobar Plan** tanto en modo `Estricto` como en modo `Flexible`

---

### Requirement: Panel de Ajustes del Agente y Salud del Proveedor
`"neko-ai"` SHALL incluir una vista de ajustes que muestre el estado de `probe_ai_driver_health_cmd` y permita editar y restaurar las directivas de Capa 2 y opciones del proveedor en SQLite.

#### Scenario: Edición y restauración de directivas y parámetros
- **WHEN** el usuario guarda cambios en rol del operador, reglas de infraestructura, ruta opcional del binario o `context_max_lines`, o pulsa **Restaurar predeterminados**
- **THEN** los valores SHALL persistirse mediante los comandos IPC de preferencias y reflejarse inmediatamente en la barra rápida y siguientes turnos
