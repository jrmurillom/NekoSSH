## Context

NekoSSH actualmente cuenta con pestañas en la barra lateral para gestionar perfiles de servidores y el explorador de archivos. El usuario desea tener un monitor de recursos (CPU, RAM, Disco) en tiempo real para el servidor SSH conectado, integrado como una tercera pestaña de la barra lateral, siguiendo la estética y tokens de diseño existentes (Cyber-Sakura).

## Goals / Non-Goals

**Goals:**
- Agregar una tercera pestaña "Monitor" a la barra lateral del frontend.
- Implementar un comando en Tauri (`get_remote_sys_info`) que ejecute comandos livianos de lectura de recursos en el servidor SSH activo.
- Renderizar gráficos dinámicos de historial (Sparklines en Canvas) para CPU y RAM, actualizados por intervalo.
- Renderizar una barra de progreso clásica y de neón para el espacio de almacenamiento del Disco Duro.
- Proveer controles en la parte inferior para pausar/reanudar y cambiar el intervalo de actualización (2s, 5s, 10s).

**Non-Goals:**
- Monitorear la máquina local del cliente.
- Instalar scripts persistentes o demonios en el servidor remoto.
- Guardar o exportar el historial de rendimiento a archivos de base de datos.

## Decisions

### 1. Consulta remota de métricas sin agente
- **Elección:** Ejecutar una cadena de comandos rápidos mediante SSH: `cat /proc/stat && free -b && df -B1 /`.
- **Razón:** Es compatible con prácticamente cualquier distribución Linux sin necesidad de instalar agentes de monitoreo o utilidades adicionales. El procesamiento de strings se realiza en el frontend/backend reduciendo la carga del servidor remoto al mínimo.
- **Alternativas consideradas:** Usar `top` o `mpstat`. Fueron descartados porque sus salidas varían de formato según la versión del sistema operativo y consumen más tiempo de CPU en el servidor.

### 2. Renderizado de gráficas con Canvas 2D
- **Elección:** Usar Canvas HTML5 nativo para dibujar las sparklines (gráficos de historial).
- **Razón:** Dibujar una línea continua con gradientes semitransparentes de neón se realiza en pocos microsegundos usando aceleración por GPU. No recarga el árbol DOM del navegador, manteniendo la aplicación fluida.
- **Alternativas consideradas:** Usar SVG dinámico. Descartado porque requiere modificar nodos DOM constantemente (hasta 30 puntos por gráfica), lo que puede causar saltos de renderizado al tener terminales abiertas ejecutando flujos rápidos.

### 3. Vinculación del contexto de conexión (Herencia del padre)
- **Elección:** El monitor consultará al backend utilizando el `terminal_id` de la pestaña padre activa (el mismo que el SFTP).
- **Razón:** En NekoSSH se admiten múltiples sub-paneles o sub-terminales hijas divididas, pero todas comparten el mismo host y conexión subyacente. Consultar por pestaña padre evita duplicar la recolección de métricas sobre la misma máquina virtual, ahorrando recursos de CPU y red.

## Risks / Trade-offs

- **[Riesgo]** Servidores remotos que no sean Linux (ej. macOS o FreeBSD) fallarán al intentar leer `/proc/stat` o usar parámetros específicos de `free`/`df`.
  - *Mitigación:* Capturar errores de parsing en el frontend y mostrar estados controlados "N/A" de forma elegante, manteniendo la terminal y la app activas sin congelarse.
