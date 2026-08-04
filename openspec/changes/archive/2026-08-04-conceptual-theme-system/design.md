## Context
Actualmente, los colores en NekoSSH están definidos de forma rígida en la pseudo-clase `:root` dentro de `styles.css` y las configuraciones de terminal en `main.ts` instancian directamente colores quemados en el objeto de configuración de `xterm.js`. No existe un mecanismo para cambiar la paleta de colores de la aplicación de forma dinámica. La interfaz cuenta con un `#prefs-popover` para configuraciones generales, pero carece de opciones de personalización visual integrales. El proyecto requiere un sistema temático conceptual que ofrezca 8 temas inspirados en anime y estética cyberpunk.

## Goals / Non-Goals
**Goals:**
- Implementar un sistema de temas dinámico que permita alternar entre 8 paletas de colores conceptuales sin requerir recargar la aplicación.
- Proveer una interfaz de selección de temas intuitiva en el `#prefs-popover` utilizando previsualizaciones en formato de esferas bi-color (split-color balls).
- Sincronizar automáticamente los colores de la terminal (`xterm.js`) con el tema seleccionado.
- Persistir la selección de tema del usuario utilizando `localStorage`.
- Extraer colores hardcodeados de `styles.css` y migrar todos los colores a variables CSS estandarizadas.

**Non-Goals:**
- Soportar temas definidos por el usuario (custom themes) en esta fase.
- Modificar la estructura de componentes HTML fuera del selector de temas.
- Proveer internacionalización para los nombres de los temas.

## Decisions

### 1. Theme Mechanism
**Decision:** Utilizar el atributo `data-theme` en el elemento `<html>` (o `<body>`) para inyectar y cambiar variables CSS.
**Rationale:** Permite encapsular las variables CSS en selectores `[data-theme="nombre-tema"]` dentro de `styles.css`. El navegador se encarga de re-computar y repintar instantáneamente los estilos cuando el atributo cambia en el DOM. Es una estrategia limpia, nativa y no requiere frameworks adicionales.
**Alternatives Considered:** Utilizar múltiples archivos CSS (uno por tema) y cambiar dinámicamente la etiqueta `<link>`. Fue descartado por introducir latencia de red al cambiar de tema, y aumentar la complejidad del bundle.

### 2. Token Naming
**Decision:** Mantener una convención funcional general para los tokens de UI (ej. `--color-primary`, `--bg-dark-base`, `--color-accent`) y mapear colores temáticos específicos a estas variables funcionales.
**Rationale:** La interfaz necesita tokens genéricos para poder adaptarse a cualquier paleta. Los tokens actuales (como `--color-sakura-neon` y `--color-cyan-electric`) son específicos al tema NekoSSH original. Se deben refactorizar a nombres semánticos (ej. `--color-accent-primary`, `--color-accent-secondary`) de forma que cada bloque `[data-theme="..."]` asigne sus colores únicos a la misma estructura de tokens esperada por la UI.
**Alternatives Considered:** Mantener los nombres actuales en la UI y asignar nuevos valores. Descartado porque nombres como `--color-sakura-neon` perderían sentido semántico bajo el tema "Hatsune Miku" o "Persona 5".

### 3. xterm.js Sync Strategy
**Decision:** Mantener una estructura de datos estática en TypeScript con los valores hexadecimales de cada tema y aplicar esta configuración al objeto `xterm.js` utilizando el método `term.options.theme` cada vez que se detecte un cambio de tema.
**Rationale:** `xterm.js` dibuja sus contenidos en un `<canvas>` y no puede leer directamente las variables CSS aplicadas en su contenedor de forma reactiva (requiere inyección directa de valores de color). Almacenar la definición de los 8 temas en un objeto TS/JS permite inyectar fácilmente los colores requeridos en el objeto `theme` de xterm.js de manera sincronizada.
**Alternatives Considered:** Leer las variables CSS computadas desde el DOM a través de `getComputedStyle` y pasarlas a `xterm.js`. Descartado por ser propenso a errores, ineficiente por el re-flow del DOM y complejo para manejar fallbacks.

### 4. UI Component Design (Theme Selector)
**Decision:** Diseñar el selector de temas como una grilla de esferas bi-color ("split-color balls") dentro del `#prefs-popover`. Cada esfera mostrará el color de fondo principal y el color de acento característico de cada tema usando un gradiente CSS lineal al 50%.
**Rationale:** Proveer una experiencia visual rica e inmediata sin necesidad de leer texto. Se ajusta perfectamente a la estética visual de la aplicación. Las esferas indicarán su estado "activo" a través de un anillo o sombra brillante.
**Alternatives Considered:** Usar un elemento `<select>` estándar o botones de texto. Descartado por no alinear con la estética y UX cuidada del proyecto.

### 5. Persistencia (localStorage)
**Decision:** Almacenar el tema seleccionado en la clave `nekossh_theme` en el `localStorage`.
**Rationale:** Solución estándar, sincrónica y rápida que permite recuperar el tema seleccionado inmediatamente en la carga de la página antes del primer renderizado completo, evitando parpadeos ("FOUC" - Flash of Unstyled Content).
**Alternatives Considered:** Almacenar la preferencia a nivel de sistema operativo o archivos locales. Descartado por ser innecesariamente complejo para una configuración puramente visual de la aplicación.

### 6. Gradientes y Colores Hardcodeados
**Decision:** Reemplazar todos los colores hardcodeados (como `#d82b7d` y `#140a16`) en `styles.css` con llamadas a variables CSS locales (`var(--...)`). Los gradientes se compondrán en base a los tokens redefinidos.
**Rationale:** Garantiza que absolutamente toda la interfaz pueda adaptarse al tema en uso, evitando artefactos visuales donde ciertas sombras o gradientes desentonen con la nueva paleta.
**Alternatives Considered:** Mantener componentes aislados sin tematización. Descartado porque rompe con la cohesión visual del sistema.

## Risks / Trade-offs
- **Performance de Renderizado de Canvas:** Llamar a `term.options.theme` causa un re-dibujado completo de la terminal activa. Puesto que los cambios de tema son infrecuentes y manuales, este trade-off de rendimiento es mínimo y completamente aceptable.
- **Complejidad de Mantenimiento de Estilos:** Se deben mantener sincronizados los colores definidos en CSS para la interfaz y los definidos en TypeScript para la terminal. Esta duplicidad semántica es necesaria por el funcionamiento de canvas, pero introduce un leve riesgo de divergencia visual si no se actualizan en conjunto durante futuros cambios.
- **Legibilidad (Contraste):** Los 8 temas propuestos son conceptuales. Existe un riesgo inherente de que ciertas combinaciones no cumplan con estrictos estándares de contraste de accesibilidad, aunque se priorizará la legibilidad de la terminal como requerimiento principal.
