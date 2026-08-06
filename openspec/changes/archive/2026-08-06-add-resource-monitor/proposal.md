## Why

Actualmente, NekoSSH permite la conexión a terminales SSH y la gestión de archivos por SFTP, pero no ofrece visibilidad sobre el estado de salud del servidor remoto. Los desarrolladores y administradores de sistemas necesitan monitorear el uso de CPU, RAM y almacenamiento (Disco) en tiempo real para reaccionar a bloqueos o picos de carga durante sus sesiones de terminal sin salir de la aplicación ni ejecutar comandos manuales repetitivos.

## What Changes

- Se añadirá una pestaña dedicada llamada "Monitor" en la barra lateral del cliente SSH (junto a "Servidores" y "Archivos").
- Al conectarse a un servidor remoto, esta pestaña mostrará el consumo actual de CPU, RAM y almacenamiento en disco con diseño Cyberpunk-Anime.
- Se implementará un mecanismo en segundo plano en el backend (Rust) para consultar estos recursos mediante comandos del sistema ligeros una vez por intervalo.
- El usuario podrá elegir el intervalo de refresco (frecuencia) o pausar/reanudar el monitoreo en tiempo real desde controles en la UI.
- La CPU y la RAM se mostrarán mediante gráficos dinámicos de historial de línea (sparklines), y el almacenamiento en disco se mostrará mediante una barra de progreso clásica.

## Capabilities

### New Capabilities
- `resource-monitor`: Proporciona visualización en tiempo real y controles del consumo de CPU, RAM y Disco del servidor SSH remoto activo en la barra lateral.

### Modified Capabilities
- Ninguna.

## Impact

- **Frontend:** Añadir pestaña, controles e interfaces de gráficas de Canvas y barra de progreso en `app/src/main.ts` e `app/index.html`.
- **Backend:** Añadir comando Tauri en `app/src-tauri/src/lib.rs` para interrogar al canal SSH del terminal activo de forma segura y liviana.
