# app-branding

## Purpose

Define las modificaciones a los requisitos de marca visual de la aplicación para soportar la coexistencia con el nuevo sistema de temas conceptuales.

## MODIFIED Requirements

### Requirement: Personalización y CRUD de Imagen de Fondo
El cliente NekoSSH SHALL permitir a los usuarios seleccionar, aplicar, regular la opacidad y eliminar una imagen de fondo personalizada (desde URL o archivo local del SO) mediante el popover de preferencias, el cual recibe una nueva sección de selección de temas que coexiste con estos ajustes. Las rutas locales de archivos MUST convertirse con la API de activos de Tauri (`convertFileSrc`) para renderizarse correctamente sobre la capa base del tema activo.

#### Scenario: Seleccionar imagen local desde el explorador del SO
- **WHEN** el usuario hace clic en "Examinar..." en las preferencias de fondo y selecciona una imagen (.png, .jpg, .webp, etc.)
- **THEN** el sistema extrae la ruta del archivo, la convierte mediante `convertFileSrc` y actualiza la capa de fondo de la aplicación con la opacidad seleccionada

#### Scenario: Eliminar imagen de fondo
- **WHEN** el usuario hace clic en el botón "Quitar" en las preferencias de fondo
- **THEN** el sistema elimina la ruta guardada en `localStorage`, limpia la capa de imagen y restaura la vista con el color base del tema actual

#### Scenario: Coexistencia con la sección de temas
- **WHEN** el usuario interactúa con el popover de preferencias
- **THEN** la interfaz presenta simultáneamente la sección de selección de temas y la sección de personalización de la imagen de fondo operando de forma independiente

## ADDED Requirements

### Requirement: Extracción de Tokens para Terminal
Los colores de la terminal xterm.js SHALL ser leídos desde los tokens del tema conceptual en uso, sustituyendo los colores estáticos "hardcoded".

#### Scenario: Sincronización de colores en el terminal
- **WHEN** se inicializa el terminal o cambia el tema
- **THEN** xterm.js utiliza los tokens definidos en el tema activo en lugar de colores codificados de manera rígida en la configuración.
