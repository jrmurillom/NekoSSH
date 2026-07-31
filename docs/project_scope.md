# Especificación y Alcance del Proyecto: NekoSSH

Este documento define el alcance, stack tecnológico y las fases de desarrollo planificadas para **NekoSSH**, un cliente SSH moderno y editor remoto visual integrado con una estética Cyberpunk-Anime estilizada y premium.

---

## 💻 Stack Tecnológico

1. **Framework Core**: **Tauri (v2)**
   - Backend en **Rust** (multithreading nativo para optimizar procesos SSH, concurrencia en transferencia de archivos SFTP y persistencia segura).
   - Frontend en **TypeScript + Vite + HTML/CSS**.
2. **Emulación de Terminal**: **xterm.js** + `xterm-addon-fit`.
3. **Editor de Código**: **Monaco Editor** integrado para edición remota.
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
    - Método de autenticación: Contraseña o llave privada (archivos `.pem`, `.key`, `.ppk`, etc., o pegado directo) junto a frase de paso (passphrase).
    - Intervalo de Keepalive.
    - Configuración de túneles SSH (Local y Dinámico/SOCKS Proxy).
* **Motor SSH**:
  - Implementación en Rust para establecer la conexión básica utilizando los perfiles guardados.
  - Integración de `xterm.js` en el frontend y paso bidireccional de entrada/salida de comandos.

---

### 📋 Fase 2: Conexión SFTP y Explorador de Archivos Sincronizado
* **Canales Concurrentes**:
  - Establecimiento de dos conexiones SSH en paralelo por servidor (una para terminal e interactividad en tiempo real y otra dedicada a SFTP) para evitar lag o bloqueos en la terminal.
* **Explorador de Archivos Visual**:
  - Estructuración de árbol de directorios remotos en el panel lateral.
* **Sincronización Bidireccional (Explorer ⇄ Terminal)**:
  - Sincronización del explorador de archivos para que siga la ruta remota activa cuando se realice navegación (`cd`) en la terminal.
  - Opción de "Abrir en Terminal" en el explorador de archivos mediante menú contextual para forzar un `cd` hacia esa ruta en la terminal abierta.

---

### 📋 Fase 3: Editor Monaco Integrado (Notepad++ Alternativa)
* **Editor Integrado**:
  - Inserción de Monaco Editor nativo dentro de la aplicación.
  - Soporte de funciones nativas básicas: Buscar (`Ctrl + F`), Reemplazar (`Ctrl + H`), resaltado de sintaxis multilinguaje y números de línea.
* **Lógica de Edición Silenciosa**:
  - Doble clic en el explorador SFTP descarga el archivo en un directorio temporal y lo abre en el editor integrado.
  - Al presionar `Ctrl + S`, la aplicación re-sube automáticamente y de manera silenciosa el archivo modificado al servidor de origen.

---

### 📋 Fase 4: Por Definir (Gestor de Snippets y Mascotas)
* **Alcance General**:
  - Implementación del diccionario de snippets local categorizado y su modal de inserción rápida.
  - Sincronización e integración de las mascotas interactivas basadas en el ecosistema Petdex local. 
  - *(Detalles técnicos exactos de funcionamiento y hooks a definir en futuras revisiones).*
