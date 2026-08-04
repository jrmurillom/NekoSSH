## Why

El autocompletado nativo del motor de WebView/navegador en los campos de texto del cliente rompe la estética de aplicación de escritorio nativa al superponer listas desplegables del navegador sobre el diseño personalizado de la UI. Este cambio desactivará globalmente esta función en el HTML y la registrará formalmente como una directiva de desarrollo en las normas generales del proyecto (SSOT).

## What Changes

- **Desactivación de autocompletado en el frontend**: Agregar el atributo `autocomplete="off"` a todos los elementos `<input>` relevantes (excepto los de tipo `file` o `hidden`) en `app/index.html`.
- **Actualización de estándares del proyecto (SSOT)**: Modificar `docs/base-standards.md` agregando una sección sobre estándares de inputs de interfaz de usuario donde se estipule de forma mandatoria que ningún input del proyecto debe almacenar o recordar datos del navegador.

## Capabilities

### New Capabilities
<!-- Ninguna nueva -->

### Modified Capabilities
- `terminal-layout`: Garantizar que los inputs dentro de la aplicación mantengan el comportamiento de UI nativo sin dropdowns del navegador.

## Impact

- **`app/index.html`**: Modificación de atributos en múltiples etiquetas `<input>`.
- **`docs/base-standards.md`**: Adición de la directiva de desarrollo mandatoria de inputs nativos en el SSOT.
