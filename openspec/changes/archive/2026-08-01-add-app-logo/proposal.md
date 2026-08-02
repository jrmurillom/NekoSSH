## Why

Actualmente, el encabezado del panel lateral (sidebar) de NekoSSH muestra únicamente texto plano ("NekoSSH" y "Estética Cyber-Sakura"). Agregar el logo oficial de la aplicación al lado del nombre reforzará la identidad de marca visual y aportará a la estética premium Cyber-Sakura.

## What Changes

- Copiar el recurso de icono de alta nitidez (`128x128.png`) desde el directorio de iconos de Tauri hacia el directorio de assets del frontend.
- Modificar el encabezado del sidebar en el archivo HTML principal para encapsular el logo y los textos de título dentro de un nuevo contenedor de marca.
- Añadir estilos CSS específicos para asegurar una correcta alineación, redimensionamiento del logo a 24px o 32px, y espaciado consistente con los tokens de diseño visual.

## Capabilities

### New Capabilities

- Ninguna. No se introducen nuevas capacidades a nivel de sistema.

### Modified Capabilities

- Ninguna. No se modifican los requerimientos funcionales de las especificaciones existentes.

## Impact

- **Archivos de UI Frontend:** Afecta a [index.html](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html) y [styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css).
- **Recursos Estáticos:** Introduce un nuevo archivo de imagen en la carpeta [app/src/assets/](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/assets).
