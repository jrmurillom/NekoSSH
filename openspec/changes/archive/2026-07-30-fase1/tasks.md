**Surface types:** desktop-ui, desktop-commands

## 0. Estructura del Workspace

- [x] 0.1 Inicializar la estructura del proyecto Tauri v2 dentro de la carpeta `app/` (el código fuente vive exclusivamente en `app/`, el root del workspace no contiene código de la aplicación)
- [x] 0.2 Inicializar el repositorio Git en la raíz del workspace, configurar la rama `feature/fase1` y realizar un commit de control de cambios.

## 1. Configuración de Interfaz y Estilos Base

- [x] 1.1 Configurar el archivo de estilos global con los tokens de diseño de la estética Cyber-Sakura
- [x] 1.2 Implementar el componente de layout principal con panel lateral izquierdo y contenedor principal
- [x] 1.3 Agregar soporte para la carga de fondo personalizado y la opacidad del contenedor translúcido

## 2. Base de Datos y Gestión de Perfiles

- [x] 2.1 Configurar la inicialización del esquema SQLite local en Rust (tablas `profiles`, `auth_credentials`, `ssh_tunnels` e índices)
- [x] 2.2 Implementar las funciones del backend Rust para el CRUD de perfiles y credenciales
- [x] 2.3 Diseñar el formulario en el frontend de TypeScript para crear, editar y eliminar perfiles de servidor
- [x] 2.4 Vincular la interfaz visual de perfiles con los comandos Tauri de persistencia local

## 3. Emulador y Motor de Conexión SSH

- [x] 3.1 Integrar `xterm.js` y `xterm-addon-fit` en el contenedor principal de terminales
- [x] 3.2 Implementar la personalización estética del cursor en xterm.js (parpadeo "blink blink" y sombra neón glow)
- [x] 3.3 Desarrollar la lógica nativa en Rust para iniciar la sesión SSH, PTY y hilos secundarios de lectura/escritura usando el crate `ssh2`
- [x] 3.4 Conectar el flujo bidireccional de entrada/salida entre el backend en Rust y xterm.js mediante Tauri events
- [x] 3.5 Implementar el control de desconexión y liberación segura de recursos al cerrar terminales

## 4. Corrección de Estilos y Tipografía

- [x] 4.1 Modificar la instanciación de xterm.js en el frontend para recuperar dinámicamente el valor de la variable de fuente `--font-mono` desde el DOM usando `getComputedStyle` y resolver el bug de fuentes proporcionales.

## 5. Pruebas Unitarias y de Integración (MANDATORIO)

- [x] 5.1 Crear e implementar pruebas unitarias para el módulo de base de datos CRUD de Rust en `app/src-tauri`.
- [x] 5.2 Ejecutar las pruebas unitarias y verificar el estado de la base de datos local, generando el reporte en `openspec/changes/fase1/reports/YYYY-MM-DD-step-N+1-unit-test-and-db-verification.md`.

## 6. Desktop UI Verification (MANDATORIO - AGENT MUST EXECUTE)

- [x] 6.1 Levantar la aplicación localmente, crear un perfil de prueba y validar que la terminal cargue la fuente monoespaciada y alinee correctamente las columnas y el ASCII art.
- [x] 6.2 Generar reporte `openspec/changes/fase1/reports/YYYY-MM-DD-step-desktop-ui-verification.md` con escenarios y resultados.

## 7. Documentación Técnica (MANDATORIO)

- [x] 7.1 Actualizar el archivo `README.md` maestro con instrucciones de instalación, dependencias agregadas (rusqlite, ssh2, xterm.js) y ejecución.

## 8. Fix: Quitar Neon Glow (aprobado vía preview)

Referencia: `docs/design/preview-no-glow.html` (modo Sin glow).

- [x] 8.1 Refactor UI: en `app/src/styles.css`, eliminar neon glow (`text-shadow`/`box-shadow` con `--glow-*` y sombras rosa/cian/verde de resplandor). Dejar acentos planos y conservar `--glass-shadow` solo como profundidad de panel.
- [x] 8.2 Actualizar `docs/design/DESIGN.md`: retirar el principio/tokens de neon glow; documentar acentos planos alineados al preview aprobado.
- [x] 8.3 Verificación desktop-ui (AGENT MUST EXECUTE): contrastar la app con el preview sin glow y generar `openspec/changes/fase1/reports/YYYY-MM-DD-step-no-glow-ui-verification.md`.

