# app-branding

## Purpose

Define los requisitos de marca visual de la aplicación, incluyendo el logotipo y elementos identificadores principales del cliente NekoSSH dentro de la UI, asegurando nitidez e integridad con el diseño Cyber-Sakura.
## Requirements
### Requirement: Visual App Logo
La interfaz del cliente NekoSSH SHALL mostrar el logo oficial al lado izquierdo del título en la barra lateral, asegurando una presentación limpia y estéticamente alineada.

#### Scenario: Visualización del Logo en Sidebar
- **WHEN** la aplicación es cargada en el cliente
- **THEN** se debe renderizar el elemento del logo junto al título "NekoSSH" manteniendo el espaciado de la estética Cyber-Sakura.

### Requirement: Personalización y CRUD de Imagen de Fondo
El cliente NekoSSH SHALL permitir a los usuarios seleccionar, aplicar, regular la opacidad y eliminar una imagen de fondo personalizada (desde URL o archivo local del SO) mediante el popover de preferencias. Las rutas locales de archivos MUST convertirse con la API de activos de Tauri (`convertFileSrc`) para renderizarse correctamente sobre la capa base del tema Cyber-Sakura.

#### Scenario: Seleccionar imagen local desde el explorador del SO
- **WHEN** el usuario hace clic en "Examinar..." en las preferencias de fondo y selecciona una imagen (.png, .jpg, .webp, etc.)
- **THEN** el sistema extrae la ruta del archivo, la convierte mediante `convertFileSrc` y actualiza la capa de fondo de la aplicación con la opacidad seleccionada

#### Scenario: Eliminar imagen de fondo
- **WHEN** el usuario hace clic en el botón "Quitar" en las preferencias de fondo
- **THEN** el sistema elimina la ruta guardada en `localStorage`, limpia la capa de imagen y restaura la vista con el color oscuro base

### Requirement: Imagen de Fondo y Opacidad Exclusiva en la Terminal
El sistema NekoSSH MUST aplicar la imagen de fondo seleccionada y su regulador de opacidad (`#config-bg-opacity`) exclusivamente dentro del contenedor de la tarjeta de la terminal (`.terminal-panel`). El resto de la aplicación (barra lateral y contenedores globales) MUST permanecer con sus estilos originales sin alteraciones.

#### Scenario: Configurar fondo y ajustar opacidad en la terminal
- **WHEN** el usuario selecciona una imagen y ajusta el slider de opacidad en el panel de preferencias
- **THEN** la imagen se renderiza dentro del recuadro de la terminal y la opacidad regula la visibilidad de la imagen debajo del texto de la consola SSH

