# Proposal - Terminal Search

Añadir un buscador flotante de texto para las terminales activas en NekoSSH utilizando la biblioteca `@xterm/addon-search`.

## Motivación
Al depurar logs o ejecutar comandos extensos, los desarrolladores necesitan buscar rápidamente palabras clave o patrones en la salida de la terminal sin tener que copiar todo el historial.

## Alcance
- Integrar `@xterm/addon-search` en la inicialización de cada panel de terminal.
- Añadir interfaz flotante (`#terminal-search-bar`) integrada con el tema Cyber-Sakura.
- Ligar atajos de teclado (`Ctrl+F`, `Esc`).
