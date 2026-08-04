## 1. Setup: Crear rama de feature (OBLIGATORIO)

- [x] 1.1 Crear y cambiar a la rama `feature/explorer-status-toast` desde `main`.
- [x] 1.2 Verificar que la rama activa en Git sea `feature/explorer-status-toast`.

## 2. HTML: Ajustar el elemento de estado

- [x] 2.1 En [app/index.html](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html), eliminar el atributo `style="display: none;"` del `<div id="files-status">`. El contenedor se controla ahora por clases CSS (`.is-visible`).

## 3. CSS: Rediseñar como overlay flotante HUD

- [x] 3.1 En [app/src/styles.css](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css), reemplazar los estilos de `.files-status` por el overlay flotante (position: absolute, bottom: 16px, glassmorphism, borde neón sakura, backdrop-filter blur, opacity/transition, pointer-events).
- [x] 3.2 Agregar la clase `.files-status.is-visible` para controlar la visibilidad (opacity: 1, pointer-events: auto).
- [x] 3.3 Actualizar `.files-status.error` para usar borde y box-shadow con `var(--color-error-neon)`.
- [x] 3.4 Agregar keyframe `@keyframes status-glow` para la animación sutil del borde neón.

## 4. TypeScript: Ajustar la función de estado con auto-dismiss

- [x] 4.1 En [app/src/main.ts](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts), declarar una variable `statusDismissTimer` para gestionar el timeout de auto-dismiss.
- [x] 4.2 Modificar `setExplorerStatus` para usar la clase `.is-visible` en vez de `display`, limpiar el timer anterior, y programar auto-dismiss de 3s para mensajes no-error.
- [x] 4.3 Ajustar `showExplorerEmpty` para que invoque `setExplorerStatus("")` en vez de manipular `display` directamente.

## 5. Verificación

- [x] 5.1 Ejecutar `npm run build` para validar que TypeScript compila sin errores.
