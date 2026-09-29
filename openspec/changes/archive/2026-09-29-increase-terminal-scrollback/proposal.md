# Propuesta: Aumentar el límite de scrollback del terminal a 10,000 líneas

## Why

Actualmente, las instancias de `Terminal` (xterm.js) se inicializan sin definir la opción `scrollback`, por lo que el emulador adopta el límite por defecto de 1,000 líneas. En escenarios reales de administración de servidores, como ejecutar `tail -n2000 catalina.log`, compilaciones o volcados de logs de contenedores, más del 50% de la salida se descarta inmediatamente del buffer. Aumentar el scrollback a 10,000 líneas resuelve este problema, alineando NekoSSH con el estándar de facto de la industria (Windows Terminal, Alacritty) sin comprometer el consumo de memoria ni el rendimiento de renderizado.

## What Changes

- Configurar explícitamente `scrollback: 10000` (10,000 líneas) en la instanciación de `Terminal` dentro de la función `createShellPane` en el frontend.
- Definir una constante `DEFAULT_TERMINAL_SCROLLBACK = 10000` exportable o en un helper dedicado para mantener la modularidad y evitar valores mágicos dispersos.
- Incorporar tests unitarios para verificar que la configuración de terminal y sus opciones se apliquen de forma consistente.

## Capabilities

### New Capabilities
<!-- Ninguna nueva capacidad de alto nivel requerida -->

### Modified Capabilities
- `ssh-terminal`: Añade el requisito de retención de buffer de scrollback con un mínimo de 10,000 líneas en todas las instancias de emulación (shell principal y shells hijos).

## Impact

- **Frontend:** [`app/src/main.ts`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src/main.ts) o helper modular de opciones de terminal.
- **Rendimiento / Memoria:** Incremento controlado de RAM de ~10-20 MB por shell activo con buffer lleno, totalmente seguro para Tauri v2 / WebView2.
- **Tests:** Pruebas unitarias en [`app/tests/`](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/tests/).
