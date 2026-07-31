## Context

Este documento técnico de diseño describe la implementación de la Fase 1 de **NekoSSH**. Establecemos la estructura del frontend (TypeScript + CSS Vanilla con tokens Cyber-Sakura), la persistencia local de perfiles (SQLite) y el motor de conexión SSH en Rust embebido en Tauri v2.

## Goals / Non-Goals

**Goals:**
- Estructurar el layout principal (barra lateral de conexiones y área de terminales).
- Implementar el esquema de base de datos de perfiles y su CRUD local.
- Crear la comunicación PTY asíncrona entre xterm.js y el backend de Rust mediante Tauri commands y events.
- Implementar el estilo visual Cyber-Sakura (fondo translúcido, cursor parpadeante neón sakura y glow).

**Non-Goals:**
- Edición remota de archivos mediante Monaco Editor (Fase 3).
- Explorador y transferencia de archivos SFTP (Fase 2).
- Gestor de snippets o mascotas interactivas (Fase 4).

## Decisions

### 1. Motor de Conexión SSH: `ssh2` Crate en Rust
- **Decisión**: Se utilizará el crate `ssh2` en el backend para gestionar la inicialización del cliente, la autenticación y el canal de sesión interactiva (PTY).
- **Alternativa Considerada**: Crate `russh`. Se prefiere `ssh2` por su compatibilidad y estabilidad al interactuar con PTYs de Unix/Windows y su simplicidad de uso para canales interactivos.
- **Implementación**: Cada sesión SSH se ejecutará en un hilo secundario de Rust (`std::thread`), comunicando la salida a través de canales asíncronos y Tauri Events hacia el frontend.

### 2. Persistencia Local: `tauri-plugin-sql` con SQLite
- **Decisión**: Utilizar el plugin oficial de Tauri para SQLite.
- **Alternativa Considerada**: Crate `rusqlite` nativo. Se prefiere el plugin oficial de Tauri porque abstrae de forma segura la integración asíncrona entre el frontend de TypeScript y la base de datos local SQLite, reduciendo la necesidad de escribir comandos Tauri personalizados para consultas simples.

### 3. Emulador de Terminal: `xterm.js` y `xterm-addon-fit`
- **Decisión**: Usar xterm.js con el addon fit.
- **Alternativa Considerada**: Crear un emulador basado puramente en divs HTML y CSS. Se descarta debido a la complejidad de emular el comportamiento PTY, control de caracteres de escape ANSI y atajos del terminal de forma nativa.

### 4. Estilos y Tokens de Diseño: CSS Vanilla
- **Decisión**: Implementar variables CSS Custom Properties locales para encapsular el tema Cyber-Sakura (fondos púrpuras, rosa neón, etc.) y bordes con efecto glassmorphism.
- **Alternativa Considerada**: Tailwind CSS. Se descarta para evitar dependencias innecesarias de build de estilos ad-hoc y mantener el control completo de la fidelidad estética y animaciones personalizadas en CSS.

## Risks / Trade-offs

- **[Risk]** Bloqueo de la terminal debido a operaciones SSH síncronas.
  - *Mitigación*: Toda inicialización de la conexión SSH y lectura del flujo del socket se realizará dentro de hilos secundarios en Rust. La comunicación entre Rust y xterm.js en TypeScript será completamente asíncrona usando eventos bidireccionales de Tauri (`tauri::Window::emit` y `listen`).
- **[Risk]** Fuga de recursos al cerrar terminales abiertas sin cerrar las conexiones en Rust.
  - *Mitigación*: Se implementará un gestor de estados mutable (`tauri::State`) en Rust que mantenga un mapa de las conexiones activas asociadas a cada terminal ID. Cuando una pestaña de terminal se cierre en el frontend, se enviará un comando de desconexión explícito y se destruirá de forma segura el hilo asociado en Rust.

### Corrección de Ruta (Fix)

**Directiva:** Todo el código fuente de la aplicación Tauri v2 (frontend TypeScript/CSS y backend Rust) debe residir exclusivamente dentro del directorio `app/`. El directorio raíz del workspace (`C:\Users\Roberto\Documents\antigravity\NekoSSH\`) queda reservado para documentación (`docs/`), configuración de agentes (`.agents/`, `.gemini/`, etc.), artefactos OpenSpec (`openspec/`), scripts auxiliares (`scripts/`) y archivos de configuración del proyecto. Ninguna tarea de implementación debe crear ni modificar archivos de código fuente fuera de `app/`.

### Corrección de Tipografía Dinámica (Fix)

**Directiva:** La tipografía de la terminal no debe ser estática en TypeScript. Se debe leer el valor de la variable `--font-mono` definida en el sistema de diseño usando `getComputedStyle(document.documentElement).getPropertyValue('--font-mono')` en tiempo de ejecución. Esto garantiza compatibilidad absoluta con temas dinámicos.

