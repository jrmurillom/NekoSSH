## Context

La aplicación NekoSSH cuenta con un control de imagen de fondo y regulador de opacidad. El requerimiento establece que la imagen de fondo debe aplicarse y visualizarse únicamente dentro de la tarjeta de la terminal (`.terminal-panel`), dejando el resto de la aplicación intacto.

## Goals / Non-Goals

**Goals:**
- Cargar y renderizar la imagen de fondo exclusivamente dentro del contenedor `.terminal-panel`.
- Vincular el slider `#config-bg-opacity` para controlar la capa de contraste de la imagen dentro de la terminal.
- Preservar la nitidez del texto de xterm.js utilizando `allowTransparency: true` y `background: transparent`.

**Non-Goals:**
- Modificar estilos de la barra lateral, cabeceras o contenedores globales.

## Decisions

### 1. Aplicación Directa de Imagen en `.terminal-panel`
- En `app/src/styles.css`:
  ```css
  .terminal-panel {
    background-size: cover;
    background-position: center;
    position: absolute;
  }
  .terminal-panel::before {
    content: "";
    position: absolute;
    inset: 0;
    background: rgba(8, 4, 9, var(--terminal-overlay-opacity, 0.70));
    z-index: 1;
  }
  .terminal-canvas-container {
    position: relative;
    z-index: 2;
    background: transparent !important;
  }
  ```

### 2. Regulador de Opacidad
- En `app/src/main.ts`, al cambiar `#config-bg-opacity`, actualizar `--terminal-overlay-opacity` en `.terminal-panel` de forma que a mayor opacidad seleccionada (1.0), menor sea la tinte oscura (0.20), revelando la imagen.

## Risks / Trade-offs

- Ninguno. La imagen queda aislada al recuadro de la consola.
