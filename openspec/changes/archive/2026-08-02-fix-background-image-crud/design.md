## Context

La personalización visual de NekoSSH contempla la posibilidad de establecer una imagen de fondo con opacidad regulable sobre la base Cyberpunk-Sakura. La implementación previa presentaba problemas donde la imagen ingresada no se mostraba en pantalla debido a bloqueos de seguridad del navegador WebView2 al cargar rutas de archivos locales sin el protocolo `asset://` de Tauri y al solapamiento del layer de color base negro opaco sobre el layer de la imagen.

## Goals / Non-Goals

**Goals:**
- Garantizar que cualquier imagen (URL remota `http/https` o ruta local en el disco del SO) se aplique y renderice correctamente como fondo de la aplicación.
- Ajustar la jerarquía visual CSS (`z-index`) para que la opacidad regulada (por defecto `0.30`) fusione la imagen de fondo con el color oscuro base (`#0c060d`).
- Proveer una experiencia CRUD fluida en las preferencias del footer: seleccionar desde el SO con "Examinar...", aplicar con "Aplicar", regular opacidad con el slider y eliminar con "Quitar".

**Non-Goals:**
- Subir imágenes a un servidor remoto o backend Rust (se gestionan rutas de archivos locales o URLs públicas).

## Decisions

### 1. Jerarquía CSS de Capas de Fondo
- **Decisión:** En `app/src/styles.css`:
  ```css
  .bg-overlay-layer {
    position: absolute;
    top: 0; left: 0; width: 100%; height: 100%;
    background-color: var(--bg-dark-base); /* #0c060d */
    z-index: -2; /* En el fondo absoluto */
  }
  .bg-image-layer {
    position: absolute;
    top: 0; left: 0; width: 100%; height: 100%;
    background-size: cover;
    background-position: center;
    z-index: -1; /* Encima del fondo negro base */
  }
  ```
- **Justificación:** La capa de fondo negro base se mantiene en `z-index: -2`. La capa de la imagen se ubica en `z-index: -1` con su propiedad `opacity` variando entre `0` y `1`. De este modo, la imagen se trasluce sobre el color negro base de forma exacta según el slider.

### 2. Uso de `convertFileSrc` de Tauri para Archivos Locales
- **Decisión:** Importar `convertFileSrc` desde `@tauri-apps/api/core`. En `applyBackgroundSettings(url: string, opacity: number)`:
  ```ts
  let displayUrl = url;
  if (url && !url.startsWith("http://") && !url.startsWith("https://") && !url.startsWith("data:")) {
    displayUrl = convertFileSrc(url);
  }
  bgImageLayer.style.backgroundImage = displayUrl ? `url("${displayUrl}")` : "";
  ```
- **Justificación:** Transforma rutas como `C:\Pictures\bg.png` en URLs compatibles con el protocolo de activos de Tauri (`asset://` o `https://asset.localhost/...`), permitiendo que el navegador las cargue sin violar CORS ni políticas de seguridad.

### 3. Interfaz CRUD en Preferencias
- **Decisión:** En `app/index.html` (dentro de `#prefs-popover`), estructurar el grupo de controles con:
  - Input `#config-bg-url` para ver o escribir la ruta/URL.
  - Botón `#btn-browse-bg` para activar un `<input type="file" id="file-input-bg" accept="image/*">` oculto.
  - Botón `#btn-apply-bg` para confirmar cambios.
  - Botón `#btn-clear-bg` para limpiar la imagen y resetear la preferencia.

### 4. Módulo de Ayuda y Pruebas Unitarias (`bg-settings-helper.ts` & `bg-settings-helper.test.ts`)
- **Decisión**: Extraer la lógica pura de resolución de URLs de fondo y formateo/validación de opacidad a un módulo desacoplado `app/src/bg-settings-helper.ts` y escribir su suite de pruebas unitarias en `app/src/bg-settings-helper.test.ts`.
- **Casos de prueba automatizados en Vitest**:
  - Conversión de rutas de archivos locales (Windows/Unix) aplicando el convertidor de activos de Tauri.
  - Preservación de URLs remotas (`http://`, `https://`, `data:`).
  - Manejo de valores vacíos o nulos (limpieza de background).
  - Clamping y formateo decimal de valores de opacidad entre `0.00` y `1.00`.

## Risks / Trade-offs

- **[Riesgo]** Rutas de archivos eliminados posteriormente del disco por el usuario.
  - *Mitigación:* Si la imagen no existe o falla en cargar, la app degrada limpiamente mostrando la capa base `#0c060d`.

### Corrección de Ruta (Fix — Normalización de Iconografía, Ajuste de Layout y Pruebas Unitarias)
* **Problema Identificado:** Los botones con etiquetas de texto ("Guardar", "Aplicar", "Examinar", "Quitar") en `.config-row` dentro del popover `#prefs-popover` desbordaban el contenedor lateral (`sidebar` de ~240px) hacia la derecha. Además, se requería asegurar cobertura de pruebas unitarias para la lógica del cambio de imagen de fondo.
* **Solución Técnica:**
  - **Eliminar Labels de Texto**: Convertir todos los botones de `.prefs-popover` en **icon-buttons de Lucide** compactos sin etiquetas de texto (usando tooltip/title nativo para accesibilidad).
  - **Fila de Editor Preferido**: Input de ruta + Botón icono **Examinar** (`AppIcons.folder`, activa explorador nativo del SO para ejecutables/scripts) + Botón icono **Guardar** (`AppIcons.check`).
  - **Fila de Fondo de Imagen**: Input de URL/ruta + Botón icono **Examinar** (`AppIcons.folderPlus` / `folder`, activa explorador nativo del SO para imágenes) + Botón icono **Aplicar** (`AppIcons.check`) + Botón icono **Quitar** (`AppIcons.trash2` / `x`).
  - **Corrección de CSS Anti-Desbordamiento**: Aplicar `min-width: 0` a los inputs de `.config-row`, `flex-shrink: 0` a los icon-buttons y `max-width: 100%` a `.prefs-popover` para garantizar que la tarjeta flotante jamás desborde el sidebar.
  - **Cobertura de Pruebas Unitarias**: Crear `app/src/bg-settings-helper.ts` y `app/src/bg-settings-helper.test.ts` para verificar la resolución de URLs y opacidad mediante Vitest (`npm run test`).
  - **Permisos de Seguridad Tauri v2**: Adición de `"core:asset:default"` en `app/src-tauri/capabilities/default.json` permitiendo al motor WebView2 resolver el protocolo de activos de disco `asset://`.
  - **Respaldo con FileReader Data URIs**: Implementación de `FileReader.readAsDataURL` al seleccionar una imagen desde el explorador nativo, asegurando que la imagen se renderice al 100% de manera inmediata en el WebView sin depender de permisos de filesystem locales.
  - **Transparencia en Tarjeta de Terminal (`.terminal-panel`) y `xterm.js`**: Reemplazo de fondo negro opaco `#080409` en `.terminal-panel` por capa glassmorphism traslúcida (`rgba(8, 4, 9, 0.75)` con `backdrop-filter: blur`), adición de `allowTransparency: true` y `theme.background: "transparent"` en la instancia de xterm.js para transparentar el viewport de la terminal sobre la imagen de fondo.

