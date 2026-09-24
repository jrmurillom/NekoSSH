# Changelog

Todos los cambios notables de este proyecto se documentan en este archivo.

El formato se basa en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/).

## [Unreleased]

## [0.1.8] - 2026-09-24

### Added

- **Descarga directa de archivos SFTP en streaming (`sftp-file-download-streaming`):**
  - Añadida la opción **"Descargar"** (con icono Lucide `Download`) en el menú contextual de archivos remotos del explorador SFTP.
  - Integrado selector nativo del sistema operativo **"Guardar como…"** (`rfd::FileDialog::save_file` vía `sftp_pick_download_path`) prellenado con el nombre del archivo remoto.
  - Motor de descarga en streaming por bloques de `32 KiB` en memoria constante $O(1)$ hacia un archivo temporal oculto local (`.<nombre>.nekossh.part`), con validación de integridad de tamaño en bytes y renombrado atómico al destino final, **sin la restricción de `10 MiB`** exclusiva del flujo de edición externa.
- **Telemetría de progreso SFTP en tiempo real (`sftp-transfer-progress-streaming`):**
  - Refactorizada la subida de archivos (`sftp_upload_file`) a streaming desde disco en bloques de `64 KiB` hacia archivo temporal remoto `.nekossh.part` con vaciado de buffers (`flush`), bombeo continuo de PTY (`pump_pty`), verificación `sftp.stat()` y renombrado atómico.
  - Canal IPC `tauri::ipc::Channel<TransferProgressEvent>` con *throttling* de `100ms` (porcentaje, bytes transferidos, bytes totales y velocidad en tiempo real) y barra de progreso unificada (`.files-status-progress-bar`) para **Descarga**, **Subida** y **Copiar/Pegar SCP**.
- **Cancelación cooperativa de transferencias SFTP (`sftp-transfer-cancellation`):**
  - Implementado `ActiveTransferRegistry` global con guardias RAII (`ActiveTransferGuard` en `Drop`) y banderas atómicas `Arc<AtomicBool>` por sesión y transferencia.
  - Añadido botón compacto `[ ✕ ]` (`.files-status-progress-cancel`) en el banner de progreso con confirmación `confirmDialog` A1 (`"Cancelar transferencia"`, `"Sí, cancelar"` / `"Seguir transfiriendo"`) sin interrumpir la transferencia mientras el diálogo permanece abierto.
  - Aborto cooperativo inmediato y eliminación automática de archivos parciales `.nekossh.part` (locales o remotos) al confirmar la cancelación, al cerrar la pestaña de sesión (`close_ssh_session`) o al salir de la aplicación (`RunEvent::Exit`).
- **Menú contextual en pestañas de terminal (`terminal-tab-context-menu`):**
  - Clic derecho sobre `.term-tab` con las 5 acciones estilo VS Code: *Cerrar pestaña*, *Cerrar otras pestañas*, *Cerrar pestañas a la izquierda*, *Cerrar pestañas a la derecha* y *Cerrar todas las pestañas* (`.is-danger`).
  - Soporte de ítems deshabilitados (`disabled?: boolean` / `.chrome-context-item.is-disabled` con `aria-disabled="true"`) en `showContextMenu`, confirmación única `confirmDialog` cuando el cierre afecta a $\ge 2$ pestañas y transferencia automática de foco (`switchActiveTerminal`) a la pestaña clicada cuando la pestaña activa previa forma parte del conjunto cerrado.

### Changed

- **Sincronización de Single Source of Truth (SSOT):**
  - Actualizados `docs/design/DESIGN.md`, `docs/design/ui-layout-contract.md`, `docs/project_scope.md` y las especificaciones maestras en `openspec/specs/` (`ui-overlays`, `sftp-explorer`, `terminal-layout`, `sftp-file-download`, `sftp-transfer-cancellation` y `terminal-tab-context-menu`).

## [0.1.7] - 2026-08-31

### Fixed

- **Integridad de subida de archivos binarios (`.zip`, imágenes, etc.):**
  - Implementado vaciado explícito de buffers (`remote.flush()`) con reintentos no bloqueantes y bombeo continuo de PTY (`pump_pty`) en `sftp_upload_file_blocking` antes de destruir el descriptor de archivo remoto.
  - Añadida verificación de integridad post-subida mediante `sftp.stat()` comparando byte a byte el tamaño remoto con el archivo local para evitar corrupción por terminación prematura del socket.
- **Descarga fresca mandatoria en edición externa:**
  - Corregido el problema de reapertura de versiones desactualizadas al presionar "Editar" en el explorador de archivos.
  - Al iniciar una edición sobre un archivo ya abierto previamente, el sistema detiene limpiamente el watcher anterior, fuerza una nueva descarga completa desde el servidor SFTP y establece un nuevo watcher activo sobre la versión fresca.

## [0.1.6] - 2026-08-13

### Added

- **Arrastrar y soltar para subir al explorador SFTP (Archivos):**
  - Overlay de dropzone derivado de tokens del tema (sin color fijo) e icono Lucide `upload`.
  - Destino por fila: carpeta bajo el cursor → esa ruta; archivo → carpeta contenedora; fondo → cwd actual.
  - Confirmación A1 siempre antes de subir (nombre o cantidad + destino) y confirmación aparte por sobreescritura.
  - Subida secuencial con `sftp_upload_file`, progreso en el estado del explorador, tolerancia a fallos parciales y refresco al terminar.
- Helper puro `explorer-drop-helper` con unit tests (resolución de destino, colisiones, texto de confirmación).
- Spec `sftp-explorer` sincronizado (drag & drop, sobreescritura y progreso) y change archivado `sftp-drag-drop-upload`.

## [0.1.5] - 2026-08-13

### Added

- **Filtro por nombre en el explorador SFTP (Archivos):**
  - Campo de filtro sobre las entradas ya pintadas del árbol (cwd + hijos de carpetas expandidas).
  - Coincidencia por substring sin distinguir mayúsculas/minúsculas; padres ancla cuando un hijo coincide.
  - Botón **×** para limpiar el filtro y volver a mostrar todo; estado “(sin coincidencias)” si no hay match.
  - Filtro 100 % cliente (sin listados SFTP adicionales ni búsqueda recursiva remota).
- Spec `sftp-explorer`: requisito de filtro por nombre sincronizado a main specs.
- Archivo OpenSpec de `sftp-explorer-name-search` y `notas`.

## [0.1.4] - 2026-08-06

### Added

- **Pestaña "Notas" (CRUD SQLite):**
  - Integrado soporte de base de datos local SQLite mediante la migración `007_notes.sql`.
  - Desarrollada la interfaz lateral de "Notas" con creación rápida (`+`), listado y ordenamiento.
  - Diseñado el modal de edición de notas flotante (glassmorphism) con título editable inline, editor de texto plano y borrado permanente con confirmación.
  - Auto-guardado inteligente (debounced a 1s o inmediato al perder el foco/cerrar el modal).
- **QoL & Mejoras Visuales:**
  - Rediseñado el botón de "Snippets" para usar el estilo **Outline Holográfico** con opacidades del color de acento mediante `color-mix`.
  - Agregado el botón de ayuda `?` con popover acrílico listando los atajos de teclado principales del sistema.
  - Añadido el atajo de teclado global `Ctrl + Shift + S` para abrir el modal flotante de Snippets.
  - Cargado dinámico de la versión real de la release de la app de Tauri en la barra lateral.
  - Mejoras en la barra de estado de File Explorer con estados coloreados (verde para éxito `✅` y rojo para error `❌`).

### Removed

- Removido el cuadro de bienvenida (welcome-card) de la vista home principal para lograr un aspecto minimalista y limpio.

### Fixed

- Corregidas caídas repentinas de conexión `PTY: transport read` en Windows al navegar de forma muy rápida con las flechas de dirección (history) o al presionar repetidamente la tecla Enter en logs, optimizando el comportamiento de vaciado del canal de SSH (`.flush()`).


### Added

- Buscador flotante en la terminal activa (`Ctrl + Shift + F`) para buscar texto en el buffer.
- Mapeado atajo `Esc` para cerrar el buscador y devolver el foco a la terminal de forma nativa.
- Configurada propiedad `selectionInactiveBackground` en todos los temas del emulador para conservar el resaltado visual brillante de la coincidencia activa (opacidad `0.45` coincidiendo con la fuerza de color de cada tema) cuando la terminal pierde el foco al enfocar el input de búsqueda.
- Integrado el complemento oficial `@xterm/addon-search` de forma dinámica para cada celda de terminal.

## [0.1.2] - 2026-08-06

### Added

- Pestaña "Monitor" en la barra lateral con visualización de recursos en tiempo real.
- Gráficos Canvas dinámicos con efecto de brillo neón Cyber-Sakura para CPU y RAM.
- Visualización de almacenamiento de Disco Duro con barra de progreso clásica.
- Monitoreo en tiempo real de velocidad de red (descarga/subida) y deltas por segundo.
- Módulo de "Top Procesos" ordenados por consumo de CPU con distintivos de memoria y carga.
- Visualización de Uptime y Sistema Operativo del servidor remoto.
- Integración nativa de la iconografía de Lucide (`Cpu`, `Database`, `HardDrive`, `Network`, `Clock`, `Server`, `Activity`, `Crown`, `Play`, `Pause`).
- Controles de refresco (2s, 5s, 10s) y botón de pausa/reanudar interactivo.

## [0.1.1] - 2026-08-05

### Added

- Persistencia de wallpaper por tema en SQLite (`theme_wallpapers`) con copia de imagen en el data dir de la app (`wallpapers/`).
- Commands IPC para get/set/clear de wallpaper (archivo, URL http(s), bytes/data URL de migración, opacidad).
- Migración one-shot desde `localStorage` (`nekossh-bg-by-theme` y claves legacy) hacia BD + disco.
- Protocolo `asset` de Tauri para renderizar wallpapers locales con `convertFileSrc`.
- Smoke `smoke_theme_wallpapers` y tests del módulo Rust/helper frontend.

### Changed

- El fondo de terminal ya no se guarda como data URL en `localStorage` (evita el techo ~5 MB).
- Specs `app-branding` y `conceptual-themes`: fuente de verdad SQLite + disco.
- `docs/design/DESIGN.md`: documenta la nueva persistencia.

### Fixed

- Imágenes grandes de fondo se pueden persistir entre reinicios (ya no “aplicado pero no guardado” por cupo de storage).

## [0.1.0] - 2026-08-05

- Release inicial publicado en GitHub.
