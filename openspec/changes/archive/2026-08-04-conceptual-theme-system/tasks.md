**Surface types:** desktop-ui

## 0. Setup: Create Feature Branch (MANDATORY)

- [x] 0.1 Crear rama `feature/conceptual-theme-system` y cambiar a ella
- [x] 0.2 Verificar rama actual con `git branch --show-current`

## 1. Refactorizar tokens CSS a nombres semánticos

- [x] 1.1 Renombrar los tokens actuales de `:root` en `app/src/styles.css` a nombres funcionales/semánticos (ej. `--color-sakura-neon` → `--color-accent-primary`, `--color-cyan-electric` → `--color-accent-secondary`, etc.) manteniendo los mismos valores
- [x] 1.2 Actualizar todas las referencias `var(--color-sakura-*)`, `var(--color-cyan-*)`, `var(--color-purple-*)` en `styles.css` para usar los nuevos nombres semánticos
- [x] 1.3 Reemplazar colores hardcoded en `styles.css` (ej. `#d82b7d`, `#140a16`, `#ff6b6b`, `#ff8a8a`) con variables CSS correspondientes
- [x] 1.4 Agregar nuevos tokens de terminal al `:root` (`--term-foreground`, `--term-cursor`, `--term-cursor-accent`, `--term-selection-bg`, `--term-black`, `--term-red`, `--term-green`, `--term-yellow`, `--term-blue`, `--term-magenta`, `--term-cyan`, `--term-white`) con los valores actuales hardcoded de xterm.js
- [x] 1.5 Verificar que la app se renderiza idéntica tras el refactor (sin cambios visuales)

## 2. Infraestructura de switching de temas

- [x] 2.1 Implementar en `app/src/main.ts` la función `applyTheme(themeName: string)` que establezca `document.documentElement.dataset.theme = themeName` y persista en `localStorage` bajo la clave `nekossh-theme`
- [x] 2.2 Implementar la lectura de tema al inicio de la app: leer `nekossh-theme` de `localStorage` y aplicar `data-theme` antes del render principal; si no existe, usar `"nekossh"` como default
- [x] 2.3 Implementar la sincronización de colores de xterm.js: crear un mapa TS de temas con los valores hexadecimales de terminal para cada tema, y aplicar `term.options.theme` al cambiar de tema (para todos los terminales abiertos)
- [x] 2.4 Eliminar los colores hardcoded del objeto `theme` en la creación de `new Terminal()` en `main.ts` y leerlos desde el mapa de temas TS según el tema activo

## 3. Selector de tema en el popover de preferencias

- [x] 3.1 Agregar la sección HTML de selección de tema dentro del `#prefs-popover` en `app/index.html` con la lista de los 8 temas, cada uno con su círculo de vista previa bicolor (split-color ball) y nombre
- [x] 3.2 Agregar los estilos CSS del selector de temas en `app/src/styles.css`: layout de la lista, estilo de las esferas bicolor (`background: linear-gradient(...)` dividido 50/50), indicador de tema activo, hover
- [x] 3.3 Implementar en `app/src/main.ts` la lógica de interacción: click en un tema → llamar `applyTheme()` → actualizar indicador visual del tema activo → sincronizar terminales xterm.js abiertos

## 4. Definición de los 8 temas conceptuales (CSS + TS)

- [x] 4.1 Crear bloque `[data-theme="nekossh"]` en `styles.css` con los tokens del tema default (Cyber-Sakura — valores actuales de `:root`)
- [x] 4.2 Crear bloque `[data-theme="hatsune-miku"]` — paleta turquesa/cyan dominante, negro, rosa acento
- [x] 4.3 Crear bloque `[data-theme="rei-ayanami"]` — paleta blanco azulado, azul profundo, rojo sutil
- [x] 4.4 Crear bloque `[data-theme="neon-evangelion"]` — paleta verde neón, púrpura oscuro, naranja acento
- [x] 4.5 Crear bloque `[data-theme="cyberpunk-david"]` — paleta amarillo dorado, negro, rojo intenso
- [x] 4.6 Crear bloque `[data-theme="cyberpunk-lucy"]` — paleta rosa/magenta, blanco plateado, azul eléctrico
- [x] 4.7 Crear bloque `[data-theme="persona5"]` — paleta rojo carmesí, negro absoluto, blanco puro
- [x] 4.8 Crear bloque `[data-theme="sailor-moon"]` — paleta dorado cálido, azul marino, rosa pastel
- [x] 4.9 Agregar los valores de terminal (foreground, cursor, colores ANSI) de cada tema al mapa TS en `main.ts`

## 5. Review and Update Existing Unit Tests (MANDATORY)

- [x] 5.1 Revisar y ajustar las pruebas unitarias existentes afectadas por los cambios (ej. `bg-settings-helper.test.ts` si aplica)

## 6. Run Unit Tests and Verify Local State (MANDATORY)

- [x] 6.1 Ejecutar `npm run test` en `app/` y verificar que todos los tests pasan
- [x] 6.2 Report: `openspec/changes/conceptual-theme-system/reports/YYYY-MM-DD-step-6-unit-test-verification.md`

## 7. Desktop UI Verification (MANDATORY — AGENT MUST EXECUTE)

- [x] 7.1 Arrancar la app con `npm run dev` en `app/`
- [x] 7.2 Verificar que el tema default (NekoSSH) se renderiza idéntico al estado actual
- [x] 7.3 Verificar que el selector de temas aparece en el popover de preferencias con los 8 temas y las bolitas bicolor
- [x] 7.4 Verificar que al cambiar de tema: la UI completa cambia de paleta, la terminal sincroniza sus colores, y al recargar la app persiste el tema elegido
- [x] 7.5 Report: `openspec/changes/conceptual-theme-system/reports/YYYY-MM-DD-step-7-desktop-ui-verification.md`

## 8. Update Technical Documentation (MANDATORY)

- [x] 8.1 Actualizar `docs/design/DESIGN.md` con la arquitectura de theming, el mecanismo `data-theme` y el catálogo de tokens semánticos
- [x] 8.2 Actualizar `docs/design/ui-layout-contract.md` si la sección del selector de tema altera el layout del popover
