# Proposal: `terminal-tab-context-menu`

## Summary

Implementar un menú contextual de escritorio estilo VS Code en las pestañas de terminal SSH (`.term-tab` dentro de `#terminal-tabs-list`) utilizando el componente nativo B3 (`showContextMenu` en `overlays.ts`), permitiendo gestionar el cierre individual o masivo direccional de pestañas con confirmación consolidada de sesiones activas y preservación determinista del foco.

## Motivation

Actualmente, las pestañas de servidor en NekoSSH solo pueden cerrarse de dos maneras:
1. Haciendo clic individualmente en el botón `✕` de cada pestaña (`.term-tab-close`).
2. Haciendo clic en el botón global `"Cerrar todas"` (`#btn-close-all-terminals`).

Cuando el usuario trabaja con múltiples servidores abiertos simultáneamente (por ejemplo, 6–10 sesiones entre entornos de desarrollo, staging y producción), carece de las acciones estándar de productividad de un entorno tipo VS Code al hacer clic derecho sobre una pestaña:
- **Cerrar pestaña** (`close`)
- **Cerrar otras pestañas** (`close-others`)
- **Cerrar pestañas a la izquierda** (`close-left`)
- **Cerrar pestañas a la derecha** (`close-right`)
- **Cerrar todas las pestañas** (`close-all`)

## Scope

### In Scope
1. **Soporte de ítems deshabilitados (`disabled?: boolean`) en `showContextMenu` (`app/src/overlays.ts` y `app/src/styles.css`):**
   - Extender `ContextMenuItem` con `disabled?: boolean`.
   - Renderizar `btn.disabled = true`, `aria-disabled="true"` y la clase `.is-disabled` para que las opciones sin pestañas aplicables (p. ej., *"Cerrar pestañas a la izquierda"* en la primera pestaña o *"Cerrar pestañas a la derecha"* en la última pestaña) permanezcan visibles en su posición fija de memoria muscular pero atenuadas y no accionables (**Opción A**).
2. **Módulo puro y testeable de resolución de pestañas (`app/src/modules/terminal-tab-menu-helper.ts`):**
   - Funciones puras para calcular, dado el listado ordenado de `terminalIds` y el `targetTerminalId`:
     - Los subconjuntos exactos de pestañas a cerrar para cada acción (`close`, `close-others`, `close-left`, `close-right`, `close-all`).
     - El estado `disabled` de cada opción del menú contextual según la posición de la pestaña objetivo (`index === 0`, `index === length - 1`, `length <= 1`).
     - El mensaje y título del `confirmDialog` consolidado cuando existen sesiones SSH activas dentro del subconjunto a cerrar.
     - La resolución de qué pestaña debe quedar activa tras el cierre (si la pestaña activa anterior pertenecía al subconjunto eliminado, el foco pasa automáticamente a `targetTerminalId`).
3. **Integración en `app/src/main.ts`:**
   - Añadir listener `contextmenu` en cada `.term-tab` creado en `openTerminalTab`.
   - Ejecutar el cierre secuencial reutilizando `closeTerminalSession(id, true)` tras una **única confirmación consolidada** (`confirmDialog`) si hay $\ge 1$ sesión SSH conectada dentro del grupo afectado.
4. **Suite de pruebas unitarias y E2E DOM/UX (`app/src/modules/terminal-tab-menu-helper.test.ts`):**
   - Cobertura exhaustiva de resolución de rangos izquierda/derecha/otras/todas, estados `disabled` en extremos y pestaña única, confirmación consolidada, cancelación del diálogo y transición de foco a la pestaña clicada.

### Out of Scope
- Reordenamiento de pestañas mediante drag-and-drop.
- Fijado (*pinning*) de pestañas.
