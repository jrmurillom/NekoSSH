## Context

El elemento `#files-status` se ubica en el flujo vertical entre `#files-toolbar` y `#files-tree`. El toggle `display: none/block` empuja el árbol causando CLS. Esta variante (Opción 1) resuelve el problema con un overlay flotante estilo HUD.

## Goals / Non-Goals

**Goals:**
- Convertir `#files-status` en un overlay posicionado absolutamente en la parte inferior del panel `#panel-files`.
- Aplicar estética HUD cyberpunk: glassmorphism (backdrop-filter blur), borde neón sakura, animación de glow sutil.
- Implementar auto-dismiss para mensajes de éxito/info (fade-out tras 3 segundos). Los errores persisten hasta la próxima acción.
- Cero CLS: el árbol nunca se mueve.

**Non-Goals:**
- No crear un sistema de notificaciones genérico ni una cola de toasts apilables.
- No modificar qué mensajes se emiten ni cuándo.

## Decisions

### Overlay flotante con position: absolute
- **Decisión**: El `#files-status` se posiciona con `position: absolute; bottom: 16px; left: 12px; right: 12px;` dentro de `#panel-files` (que ya tiene `position: relative`).
- **Alternativa descartada**: Barra de consola fija — no tiene el efecto visual premium y ocupa espacio permanente.

### Visibilidad controlada por clases CSS
- **Decisión**: En vez de `display: none/block`, usar una clase `.is-visible` que controla `opacity` y `pointer-events`. Esto permite transiciones suaves con CSS.
  ```css
  .files-status { opacity: 0; pointer-events: none; transition: opacity 0.3s ease; }
  .files-status.is-visible { opacity: 1; pointer-events: auto; }
  ```

### Auto-dismiss con setTimeout
- **Decisión**: `setExplorerStatus` limpia el timeout anterior y programa uno nuevo de 3000ms para mensajes no-error. Los mensajes de error (`isError = true`) no tienen auto-dismiss y persisten hasta que se invoque `setExplorerStatus("")` o un nuevo mensaje.

### Estilos CSS HUD
- **Decisión**: Aplicar los siguientes estilos:
  ```css
  .files-status {
    position: absolute;
    bottom: 16px;
    left: 12px;
    right: 12px;
    z-index: 10;
    padding: 8px 12px;
    background: rgba(26, 21, 35, 0.92);
    backdrop-filter: blur(6px);
    -webkit-backdrop-filter: blur(6px);
    border: 1px solid var(--color-sakura-neon);
    box-shadow: 0 0 10px rgba(255, 105, 180, 0.25);
    border-radius: var(--border-radius-sm);
    font-family: var(--font-family-mono);
    font-size: 0.75rem;
    color: var(--color-text-primary);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.3s ease;
  }
  ```

## Risks / Trade-offs

- **Riesgo**: El overlay puede tapar las últimas filas del árbol cuando hay pocos archivos.
  - **Mitigación**: `pointer-events: none` cuando está oculto; el overlay es compacto (una línea) y desaparece tras 3s para info.
- **Riesgo**: Los mensajes de error quedan flotando indefinidamente si el usuario no interactúa.
  - **Mitigación**: Cualquier acción posterior (navegar, expandir, refrescar) invoca `setExplorerStatus` y reemplaza o limpia el mensaje.
