# Propuesta: Fase 1 - Esqueleto UI, CRUD de Perfiles y Motor SSH

## Why

El objetivo de esta propuesta es sentar las bases físicas de NekoSSH mediante la creación del esqueleto de interfaz de usuario Cyber-Sakura, la implementación del motor nativo en Rust para las conexiones SSH y la persistencia de perfiles de servidores en SQLite local. Esto es necesario para habilitar la funcionalidad básica de conexión remota interactiva de NekoSSH.

## What Changes

- **Esqueleto Visual del Frontend**: Layout con panel lateral de servidores y archivos, área central de terminales y editor translúcido con estilos CSS Vanilla y el tema Cyber-Sakura.
- **Persistencia en SQLite**: CRUD de servidores en base de datos local que incluye Host, Puerto, Usuario, autenticación (contraseña o llave SSH) y configuración de túneles SSH.
- **Motor SSH interactivo**: Conexión nativa en Rust que enlaza el backend Tauri con `xterm.js` en el frontend permitiendo el paso bidireccional de entrada y salida de datos de terminal.

## Capabilities

### New Capabilities

- `connection-profiles`: Gestión, almacenamiento y lectura de perfiles de conexión SSH (incluyendo credenciales y túneles locales/dinámicos) en SQLite local.
- `ssh-terminal`: Emulación de terminal interactiva utilizando `xterm.js` con soporte estético Cyber-Sakura y comunicación nativa bidireccional mediante Rust en el backend de Tauri.

### Modified Capabilities

## Impact

- **Backend (src-tauri)**: Incorporación de comandos Tauri e integración con `tauri-plugin-sql` y crate `ssh2` o `russh`.
- **Frontend (src)**: Estructura del layout base en TypeScript, configuración del emulador `xterm.js` y del CSS global.
- **Persistencia**: Creación de la base de datos `nekossh.db` en el directorio de datos locales de la aplicación.
