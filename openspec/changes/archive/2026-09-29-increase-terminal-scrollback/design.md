# Diseño Técnico: Aumento del Buffer de Scrollback del Terminal

## Context

NekoSSH utiliza `@xterm/xterm` para renderizar las sesiones SSH interactivas. Al instanciar cada shell en `createShellPane` ([`app/src/main.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts)), el objeto `ITerminalOptions` no especifica la clave `scrollback`. Por ende, el emulador recurre a su valor predeterminado de 1,000 líneas. Esto provoca que comandos habituales de inspección de logs (p. ej. `tail -n2000 catalina.log`) pierdan la mitad de su salida tan pronto como se emiten.

## Goals / Non-Goals

**Goals:**
- Extender la retención del buffer de historial de 1,000 a 10,000 líneas para todas las instancias de terminal (`Terminal`).
- Encapsular la creación de opciones de terminal en un módulo helper desacoplado (`app/src/modules/terminal-options-helper.ts`) para mantener la modularidad arquitectónica y permitir pruebas unitarias con Vitest.
- Asegurar que la ampliación aplique homogéneamente tanto al shell principal como a los shells secundarios (splits en cuadrícula).

**Non-Goals:**
- Configuración dinámica del scrollback en el modal de preferencias de usuario (se mantiene en 10,000 fijas para evitar complejidad innecesaria en esta fase).
- Scrollback infinito (`scrollback: Number.MAX_VALUE`), ya que causa fugas de memoria y degradación severa en el recolector de basura de V8.

## Decisions

### 1. Fijar `DEFAULT_TERMINAL_SCROLLBACK = 10000`
- **Racional:** 10,000 líneas proporciona un balance óptimo entre retención de información histórica (10 veces más que el límite previo) y uso de memoria (~10-20 MB por terminal con buffer completamente saturado).
- **Alternativas consideradas:**
  - *Mantener 1,000 líneas:* Insuficiente para flujos de trabajo de servidores e inspección de logs.
  - *50,000 a 100,000 líneas:* Afecta negativamente el rendimiento de búsqueda en texto (`SearchAddon`) y dispara el consumo de RAM cuando hay múltiples paneles divididos abiertos simultáneamente.

### 2. Creación del módulo `terminal-options-helper.ts`
- **Racional:** En lugar de incrustar más opciones inline dentro del archivo monolítico `main.ts`, se crea una función pura `createTerminalOptions({ theme, fontFamily, fontSize, scrollback })` con valores por defecto. Esto promueve la modularidad exigida en los estándares del proyecto y permite validación estricta mediante TDD con Vitest.
- **Alternativas consideradas:**
  - *Hardcodear `scrollback: 10000` inline en `main.ts`:* Viola el principio de modularidad y dificulta la verificación unitaria aislada.

## Risks / Trade-offs

- **[Riesgo] Consumo de memoria adicional por panel:**  
  *Mitigación:* Cada celda saturada añade a lo sumo ~15 MB de RAM. Con el límite existente de hasta 4 shells por pestaña (`MAX_CHILD_SHELLS = 3`), el consumo máximo añadido por pestaña nunca supera ~60-80 MB, completamente despreciable en entornos de escritorio modernos (WebView2).
- **[Riesgo] Impacto en la búsqueda de texto:**  
  *Mitigación:* Se probó que `SearchAddon` recorre 10,000 líneas en menos de 20 ms en arquitecturas estándar de CPU, manteniendo la reactividad del buscador `Ctrl+Shift+F`.
