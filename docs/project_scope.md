# Especificación y Alcance del Proyecto: NekoSSH

Este documento define el alcance, stack tecnológico y las fases de desarrollo planificadas para **NekoSSH**, un cliente SSH moderno y editor remoto visual integrado con una estética Cyberpunk-Anime estilizada y premium.

---

## 💻 Stack Tecnológico

1. **Framework Core**: **Tauri (v2)**
   - Backend en **Rust** (multithreading nativo para optimizar procesos SSH, concurrencia en transferencia de archivos SFTP y persistencia segura).
   - Frontend en **TypeScript + Vite + HTML/CSS**.
2. **Emulación de Terminal**: **xterm.js** + `xterm-addon-fit`.
3. **Edición remota**: editor **externo** preferido + sync con confirmación (Fase 3). Monaco Editor integrado queda diferido como **Fase 3b**.
4. **Persistencia**: **SQLite** local (vía plugin oficial de Tauri `tauri-plugin-sql`).
5. **Estilos y Temas**: CSS Vanilla basado en un sistema centralizado de **Tokens de Diseño** (CSS Custom Properties). 
   - Soporte para fondos personalizados configurables (fondos translúcidos de color o imagen).
   - Soporte para el comportamiento visual de la terminal, incluyendo la personalización del cursor (parpadeo o "blink blink", color de resplandor de neón).
6. **Mapeo de Red**: Crate Rust `ssh2` o `russh` para establecer conexiones y transferencias SFTP seguras de forma directa en el backend.

---

## 📅 Fases de Desarrollo y Alcance

### 📋 Fase 1: Esqueleto, CRUD de Perfiles y Motor SSH
* **Interfaz de Usuario (NekoSSH Theme Base)**:
  - Creación del cascarón principal con layout de panel lateral izquierdo (servidores/archivos) y área principal (terminales/editor).
  - Sistema de estilos base con CSS Tokens implementado para fuentes, espaciados, bordes y paleta de colores.
  - Personalización visual: Habilitación de cambio de background (opacidad y ruta de archivo) y estilo del cursor de la terminal (tipo de cursor, color y parpadeo "blink blink").
* **CRUD de Perfiles de Conexión**:
  - Almacenamiento local SQLite de servidores con las siguientes propiedades configurables:
    - Nombre del perfil, grupo/carpeta, Host (IP o dominio), Puerto (default 22), Usuario.
    - Método de autenticación: Contraseña o llave privada (contenido del archivo `.pem`/OpenSSH leído al seleccionar, persistido en el perfil; no se depende de rutas de SO) junto a frase de paso (passphrase).
    - Intervalo de Keepalive.
    - Configuración de túneles SSH (Local y Dinámico/SOCKS Proxy).
* **Motor SSH**:
  - Implementación en Rust para establecer la conexión básica utilizando los perfiles guardados.
  - Negociación criptográfica modular (`ssh_negotiation.rs`): matriz completa de KEX (Curvas Elípticas ECDH nativas aceleradas por hardware vía WinCNG con `LIBSSH2_ECDSA_WINCNG` en Windows, DH Group 16/14, DH-GEX), Ciphers (AES-GCM, ChaCha20, AES-CTR, y fallback a AES-CBC/3DES para servidores legacy y cPanel) y MACs (SHA-2/SHA-1), con prioridad estricta de seguridad y compatibilidad universal con servidores modernos (OpenSSH 10.0 / Debian 13).
  - Integración de `xterm.js` en el frontend y paso bidireccional de entrada/salida de comandos.

---

### 📋 Fase 2: Conexión SFTP y Explorador de Archivos Sincronizado
* **Canales Concurrentes**:
  - Establecimiento de sesiones multiplexadas con bombeo continuo del PTY (`pump_pty`) durante las lecturas/escrituras SFTP para evitar bloqueos o desconexiones en la terminal.
* **Explorador de Archivos Visual y Transferencias en Streaming**:
  - Estructuración de árbol de directorios remotos en el panel lateral con filtro por nombre en memoria.
  - **Arrastrar y soltar / Botón Subir**: arrastrar archivos locales sobre el panel Archivos muestra un overlay (derivado de los tokens del tema) con la ruta destino; carpeta bajo el cursor → esa carpeta, archivo → su carpeta contenedora, fondo → ruta actual. Al soltar se pide **siempre** confirmación A1 (nombre o cantidad + destino) y confirmación aparte por sobreescritura; la subida es en streaming de `64 KiB` hacia archivo temporal `.nekossh.part` + verificación `sftp.stat()` + rename atómico, tolera fallos individuales y refresca al terminar.
  - **Descarga directa de archivos (`Descargar`)**: opción en el menú contextual de archivos que abre el diálogo nativo `"Guardar como..."` (`rfd`) y descarga en streaming de `32 KiB` hacia un archivo temporal local `.nekossh.part` con validación de integridad y rename atómico al destino final, **sin el límite de `10 MiB`** exclusivo de la edición externa.
  - **Telemetría de progreso en vivo y cancelación cooperativa**: tanto la Descarga, la Subida como el flujo **Copiar/Pegar SCP** emiten telemetría con throttling de `100ms` (`Channel<TransferProgressEvent>`), mostrando una barra de progreso unificada con porcentaje, bytes y velocidad, acompañada de un botón `[ ✕ ]` (con confirmación `confirmDialog` A1) y cancelación cooperativa automática (`ActiveTransferRegistry`) con limpieza de `.nekossh.part` si el usuario cierra la pestaña o sale de la aplicación.
  - **Gestión avanzada de pestañas de sesión**: menú contextual estilo VS Code sobre `.term-tab` (*Cerrar pestaña*, *Cerrar otras pestañas*, *Cerrar pestañas a la izquierda*, *Cerrar pestañas a la derecha*, *Cerrar todas las pestañas*).
* **Sincronización (Explorer ⇄ Terminal)**:
  - Sincronización inicial vía **OSC 7** y opción de "Abrir en Terminal" en el explorador de archivos mediante menú contextual para enviar un `cd` hacia esa ruta en el PTY padre del contexto activo.

---

### 📋 Fase 3: Edición remota vía editor externo + sync con confirm
* **Flujo FileZilla-style**:
  - Doble clic (o menú contextual **Editar**) en un archivo del explorador SFTP descarga el remoto a un temp aislado de la app y lo abre con el **editor externo preferido** (ruta configurable) o, si no hay preferencia válida, con la asociación del SO.
  - Vigilancia del archivo local; al detectar cambios reales (debounce), dialog glass A1: **¿Subir al servidor?**
  - Confirmar → upload/replace del path remoto vía el canal SFTP de la sesión. Cancelar / Escape deja el remoto intacto.
  - Si el upload normal falla por **permisos**, dialog A1 opcional **«Subir con sudo»** (un reintento: temp remoto + `sudo -n cp`). Sin UI de contraseña; si sudo pide password / falla → alert y se conserva el temp local dirty. No es el camino por defecto.
  - **Sin** auto-upload silencioso.
* **Políticas**: rechazo amable >10 MiB; aviso A1 si el archivo parece binario; cleanup de temps; desconexión mid-edit detiene watchers y no sube.
* **Preferencia**: campo Settings “Editor externo preferido” persistido en SQLite (`app_preferences`).

### 📋 Fase 3b (futuro / diferido): Editor Monaco integrado
* Monaco nativo en el workspace (buscar/reemplazar, sintaxis, etc.) y posibles flujos de re-subida desde pestaña integrada.
* **Fuera del alcance de Fase 3** entregable; no bloquea el flujo de editor externo.

---

### 📋 Fase 4a: Gestor de Snippets
* Botón **Snippets** en el footer del sidebar → modal glass.
* Lista plana + chips de categoría (un nivel) + búsqueda.
* CRUD in-modal; **Copiar** al portapapeles (sin insertar en PTY); eliminar con confirm A1.
* Persistencia SQLite + seed demo (Apache, Tomcat, Permisos).
* Sin atajo de teclado en este slice.

### 📋 Fase 4b (futuro / diferido): Mascotas Petdex
* Sincronización e integración de mascotas interactivas del ecosistema Petdex local.
* Fuera del alcance de Fase 4a.

---

### 📋 Fase 5: Copiloto Agéntico Determinista (`NekoAI`)
* **Ventana Nativa Independiente (`"neko-ai"` / `ai-copilot.html`)**:
  - Desacoplada en ventana de sistema operativo multi-monitor (`Always on Top`, instancia única invocada desde `#btn-footer-ai` o `Ctrl+Shift+A`, cierre en cascada al destruir `"main"` y capability aislada `ai-copilot.json`). Al cerrar solo `"neko-ai"`, se cancelan turnos activos, se apaga la escucha y se conserva en RAM la memoria del arnés y feed visual/HITL por `terminal_id`, liberándose al cerrar la sesión SSH (`HarnessRegistry::remove`).
  - Sincronización en tiempo real de los 8 temas conceptuales y del catálogo de shells activos con soporte de **Pin Shell**, chip vivo `Escucha ON · N líneas`, auto-ajuste del compositor (`<= 140px`), historial con `↑`/`↓` y envío instantáneo de selección desde `xterm.js` mediante `Ctrl+Shift+E` aun con `Escucha OFF`.
* **Privacidad por Diseño, Telemetría `OSC 7` y Puerto Multi-Proveedor**:
  - Colector `PtyContextTap` estrictamente *opt-in* (`Escucha OFF` = `0 bytes` de `ssh-stdout` retenidos y purga inmediata al apagar), eco TTY resiliente a ANSI (`zsh`), detección viva de espera interactiva (`sudo`, `[y/N]`, `fish`), límite dinámico `context_max_lines` (`50..1000`), limpieza ANSI/OSC/pantalla alterna, presupuesto *Head + Tail*, seguimiento no intrusivo de `current_cwd` vía `OSC 7` y sanitización ampliada de secretos (`SecretRedactor`: PEM, `*_KEY`/`*_SECRET`/`*_TOKEN`/`*_PASSWORD`, JSON/YAML y prefijos `sk-`/`ghp_`/`gho_`/`github_pat_`/`glpat-`/`AKIA...`).
  - Puerto desacoplado `trait AiProviderDriver` con adaptador inicial `AgyCliDriver` aislado en `{appData}/ai-sandbox/` (`--sandbox`, `--disable-slash-commands`, `--json-schema`, `CREATE_NO_WINDOW`, terminación recursiva del árbol de procesos en Windows con `taskkill /F /T` y streaming limpio sin exponer JSON crudo en `#copilot-progress-stream`).
* **Arnés Conversacional Progresivo de 4 Fases con Supervisión Humana (*Human-in-the-Loop*)**:
  - **Auditoría No Bloqueante (`CommandRiskGuard`):** clasifica riesgos en 4 niveles (`read_only`, `config_change`, `service_impact`, `destructive`), desenvuelve wrappers encadenados (`xargs`, `timeout`, `nice`, `ionice`, `stdbuf`, `setsid`, `sudo`, `env`, `nohup`, `command`), reconoce redirecciones inocuas POSIX (`2>/dev/null`, `2>&1`) sin falsos bloqueos, clasifica purgas de bitácora (`journalctl --vacuum*`/`--rotate`) como `config_change` y adjunta advertencias explicativas (`audit_warning`) sin depender de *whitelists* manuales.
  - **Ciclo Adaptativo de 4 Fases (1 Comando por Turno + Puente `ProposePlan`):** (1) Diagnóstico no intrusivo (con respuesta directa y tablas Markdown en consultas informativas), (2) Causa raíz + respaldo fechado + `rollback_cmd` + soporte accionable de planes multi-paso (`ProposePlan`), (3) Ejecución progresiva un comando por turno con validación *dry-run* pre-reinicio (`nginx -t`, `sshd -t`), edición inline, **Insertar en terminal** sin `Enter`, copia, `Ctrl+C` y rescate/rollback inmediato ante fallo, y (4) Certificación *Health Check* (`is-active`, `ss -tulpn`, `journalctl`).
  - **Sembrado Instantáneo de Pantalla:** al encender `Escucha ON`, `"main"` siembra las últimas líneas visibles de `xterm.js` (pasadas por `SecretRedactor`) permitiendo diagnosticar errores ya presentes en consola.
