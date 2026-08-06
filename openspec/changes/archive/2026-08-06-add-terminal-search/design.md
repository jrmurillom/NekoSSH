# Design - Terminal Search

## UI/UX
- Buscador flotante en la esquina superior derecha de la terminal activa.
- Animación suave de entrada al presionar `Ctrl+F`.
- Borde con efecto de neón rosa, input oscuro con tipografía monospace.
- Botones Lucide de flechas anterior/siguiente y cierre.

## Flujo de Datos
1. Evento de teclado `Ctrl+F` → mostrar buscador y hacer `.focus()`.
2. Input event → llamar a `searchAddon.findNext(query, options)`.
3. Navegación → `findNext` / `findPrevious`.
4. Escape / Cierre → ocultar panel y hacer `term.focus()`.

### Corrección de Ruta (Fix)
- **Visualización de Coincidencias Inactivas:** La búsqueda nativa de Xterm.js selecciona el texto encontrado usando el fondo de selección. Sin embargo, al enfocar el input de búsqueda flotante, la terminal pierde el foco del teclado y difumina/oculta la selección activa.
- **Solución Nativa:** Añadir la propiedad `selectionInactiveBackground` con el mismo color o valor translúcido correspondiente en la definición de cada tema de terminal (`THEME_TERMINAL_COLORS` en `main.ts`). Esto preserva visualmente el resaltado del texto seleccionado de fondo sin parches en las llamadas de búsqueda.

