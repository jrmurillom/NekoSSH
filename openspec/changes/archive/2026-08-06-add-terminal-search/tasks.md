## 1. Control de Versiones (Git)

- [x] 1.1 Crear la rama de Git `feature/add-terminal-search`.

## 2. Dependencias

- [x] 2.1 Añadir `"@xterm/addon-search": "^0.16.0"` en [`app/package.json`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/package.json).
- [x] 2.2 Correr `npm install` en [`app/`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app) para instalar el Addon.

## 3. Frontend Layout & Estilos (HTML / CSS)

- [x] 3.1 Agregar la estructura HTML flotante `#terminal-search-bar` en [`app/index.html`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/index.html).
- [x] 3.2 Añadir estilos neón Cyber-Sakura en [`app/src/styles.css`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/styles.css).

## 4. Frontend Logic (TypeScript)

- [x] 4.1 Instanciar y cargar `SearchAddon` en [`app/src/main.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts).
- [x] 4.2 Ligar evento `keydown` (Ctrl+Shift+F y Esc) en [`app/src/main.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts).
- [x] 4.3 Programar lógica de navegación (`findNext`/`findPrevious`), contador de resultados y mayúsculas en [`app/src/main.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts).
- [x] 4.4 Añadir la propiedad `selectionInactiveBackground` en `THEME_TERMINAL_COLORS` en [`app/src/main.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts) para cada uno de los temas.

## 5. Compilación y Validación

- [x] 5.1 Correr `npm run build` en [`app/`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app) para validar que no haya errores de compilación.
- [x] 5.2 Probar manualmente en la terminal la búsqueda de cadenas del buffer y el comportamiento del resaltado inactivo.
