## ADDED Requirements

### Requirement: Personalización y CRUD de Imagen de Fondo
El cliente NekoSSH SHALL permitir a los usuarios seleccionar, aplicar, regular la opacidad y eliminar una imagen de fondo personalizada (desde URL o archivo local del SO) mediante el popover de preferencias. Las rutas locales de archivos MUST convertirse con la API de activos de Tauri (`convertFileSrc`) para renderizarse correctamente sobre la capa base del tema Cyber-Sakura.

#### Scenario: Seleccionar imagen local desde el explorador del SO
- **WHEN** el usuario hace clic en "Examinar..." en las preferencias de fondo y selecciona una imagen (.png, .jpg, .webp, etc.)
- **THEN** el sistema extrae la ruta del archivo, la convierte mediante `convertFileSrc` y actualiza la capa de fondo de la aplicación con la opacidad seleccionada

#### Scenario: Eliminar imagen de fondo
- **WHEN** el usuario hace clic en el botón "Quitar" en las preferencias de fondo
- **THEN** el sistema elimina la ruta guardada en `localStorage`, limpia la capa de imagen y restaura la vista con el color oscuro base
