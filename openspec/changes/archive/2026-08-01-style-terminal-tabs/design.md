## Context

El layout del área principal de terminales se compone de la barra de pestañas (`.terminal-tabs-bar`) y el panel contenedor (`.terminal-panel`). Actualmente, se renderizan de forma rectangular tradicional y plana. Se requiere un rediseño de estilos en CSS para que la pestaña activa y el contenedor del panel se perciban como una sola pieza física fluida con esquinas redondeadas y un glow sutil integrado en su contorno.

## Goals / Non-Goals

**Goals:**
- Fusionar visualmente la pestaña activa y el contenedor de la terminal.
- Aplicar esquinas redondeadas amplias en el panel de la terminal y las pestañas.
- Incorporar un glow neón sakura en el contorno del bloque unificado usando los tokens del tema.
- Garantizar un padding de seguridad en el canvas de la terminal para evitar que las curvas recorten texto.
- Todo cambio debe realizarse exclusivamente a nivel de estilos (CSS) sin modificar la estructura HTML existente.

**Non-Goals:**
- Modificar el comportamiento de la terminal o el backend en Rust.
- Utilizar colores hardcodeados fuera de los tokens oficiales del tema Cyber-Sakura.

## Decisions

### 1. Fusión de la Pestaña Activa y Contenedor
* **Decisión:** Configurar la pestaña activa (`.term-tab.active`) con el mismo color de fondo del panel superior y usar superposición de bordes.
* **Detalle técnico:**
  * Fondo: `rgba(24, 12, 26, 0.85)` con `backdrop-filter: var(--glass-blur)` para un acabado opaco translúcido consistente.
  * Superposición: Aplicar un posicionamiento relativo con `margin-bottom: -1px` y un pseudo-elemento `.term-tab.active::after` absoluto (`bottom: -2px`) de `3px` de alto para solaparse y ocultar la línea de división superior del panel.
  * Redondeo superior: `12px` en las pestañas (`border-radius: var(--border-radius-md) var(--border-radius-md) 0 0`).

### 2. Estructura de Panel con Esquinas Redondeadas y Glow
* **Decisión:** Aplicar curvatura y resplandor al contenedor `.terminal-panel`.
* **Detalle técnico:**
  * Curvatura: `border-radius: var(--border-radius-lg, 24px)`.
  * Borde y Glow: `border: 1px solid rgba(255, 105, 180, 0.25)` y `box-shadow: 0 0 25px rgba(255, 105, 180, 0.12), var(--glass-shadow)`. Esto crea un resplandor sakura sutil y limpio que fluye por todo el contorno unificado.

### 3. Evitar texto recortado en xterm.js (Padding de Seguridad)
* **Decisión:** Incrementar el padding de `.terminal-canvas-container` a `24px`.
* **Justificación:** La esquina inferior del panel con `border-radius: 24px` recortaría los caracteres de la terminal de xterm si estos se renderizan muy pegados al borde. Un padding de `24px` mantiene todo el texto dentro de la zona rectangular central segura, mientras que `@xterm/addon-fit` ajusta de forma automática el canvas a este nuevo espacio útil.

## Risks / Trade-offs

* **[Riesgo]** El incremento del padding reduce ligeramente el número de columnas/filas útiles en la terminal para resoluciones de pantalla pequeñas.
  * *Mitigación:* Se considera un intercambio aceptable por la mejora estética premium. El espacio de visualización de `800px` o superior sigue siendo más que óptimo para el uso diario de comandos SSH.

### Corrección de Ruta v2 (Fix Real)
* **Problema Identificado:** El panel `.terminal-panel` se crea dinámicamente en TypeScript y se inserta directamente en `.main-display-area`. El panel usa `width: 100%; height: 100%` — esto hace que llene el contenedor sin importar el `padding` que se aplique a `.main-display-area`, ya que `width: 100%` no descuenta el padding del padre (ignora el box model del padre cuando el hijo declara `width: 100%` + `height: 100%`).
* **Solución Técnica Correcta:**
  * En lugar de poner `padding` en `.main-display-area`, el espacio perimetral se aplica directamente al `.terminal-panel` mediante un `margin` uniforme.
  * Aplicar `margin: 0 20px 20px 20px;` en `.terminal-panel` (sin margen superior para que la pestaña activa se una al borde superior del panel).
  * Ajustar `height: calc(100% - 20px)` para compensar el margen inferior y evitar que la barra de scroll aparezca innecesariamente.
  * Ajustar `width: calc(100% - 40px)` para compensar el margen lateral de 20px a cada lado.
  * Restaurar `overflow: hidden` en `.main-display-area` para evitar scroll externo.
  * Revertir el `padding` aplicado incorrectamente a `.main-display-area` y en `.terminal-tabs-bar`.

### Corrección de Ruta v3 (Fix Definitivo — Dos Bugs)
* **Bug 1 — Desbordamiento derecho:** `position: relative` + `width: calc(100% - 40px)` + `margin` producía desbordamiento. Solución: `position: absolute` con `top: 0; left: 20px; right: 20px; bottom: 20px`.
* **Bug 2 — Panel negro:** `padding: 24px` en `.terminal-canvas-container` rompía el viewport de xterm.js. Solución: `padding: 0`.

### Corrección de Ruta v4 — Integración rota en esquina superior-izquierda
* **Problema Identificado:** Con `position: absolute; top: 0; left: 20px` y `border-radius: 24px` aplicado a las cuatro esquinas del `.terminal-panel`, la esquina **superior-izquierda** tiene una curva de 24px. Esta curva crea un hueco visible entre el costado izquierdo de la pestaña activa y el borde del panel, rompiendo la fusión visual en el lado izquierdo.
* **Solución Técnica:** Cambiar `border-radius` del panel de un valor uniforme a valores asimétricos: `border-radius: 0 var(--border-radius-lg, 24px) var(--border-radius-lg, 24px) var(--border-radius-lg, 24px)`. La esquina **superior-izquierda = 0** (cuadrada, conecta flush con la pestaña activa), las demás tres esquinas mantienen el radio de `24px` para conservar el look redondeado premium.

