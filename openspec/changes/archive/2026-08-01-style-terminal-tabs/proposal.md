## Why

La barra de pestañas y el contenedor de terminal actuales se renderizan de forma plana y rectangular tradicional, compitiendo visualmente en lugar de presentarse como una consola Cyber-Sakura estilizada. Se requiere rediseñar visualmente estos elementos para lograr un contenedor de terminal con esquinas redondeadas y glow de neón en su contorno, que se integre físicamente con la pestaña activa como una sola superficie unificada.

## What Changes

- Modificar exclusivamente los estilos del contenedor de la terminal y de las pestañas en el archivo CSS del frontend para lograr la curvatura y el glow deseados.
- Fundir visualmente la pestaña activa con el contenedor de la terminal superponiendo la pestaña en el borde superior y ocultando la línea divisoria.
- Aplicar un padding de seguridad en el contenedor de xterm.js para evitar que la curvatura de las esquinas recorte texto de la consola.
- Mantener intacto todo el HTML estructural de las pestañas y terminal existentes.
- Utilizar de forma estricta los tokens de diseño CSS ya establecidos, sin colores hardcodeados de ninguna clase.

## Capabilities

### New Capabilities

- Ninguna. No se introducen nuevas capacidades a nivel de sistema.

### Modified Capabilities

- Ninguna. No se modifican los requerimientos funcionales de las especificaciones existentes.

## Impact

- **Archivos de Estilos Frontend:** Afecta a [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css).
- **Archivos de UI Frontend:** Ningún cambio en estructura HTML, solo consumo de clases existentes.
