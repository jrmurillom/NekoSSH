## MODIFIED Requirements

### Requirement: Personalización y CRUD de Imagen de Fondo
El cliente NekoSSH SHALL permitir a los usuarios seleccionar, aplicar, regular la opacidad y eliminar una imagen de fondo personalizada (desde URL o archivo local del SO) mediante el popover de preferencias, el cual recibe una sección de selección de temas que coexiste con estos ajustes. La imagen, la etiqueta mostrada y la opacidad SHALL persistirse **asociadas al tema conceptual activo**; cambios de fondo MUST NOT alterar el wallpaper de otros temas. Las rutas locales de archivos MUST convertirse a un formato usable en el WebView (p. ej. data URL o `convertFileSrc`) para renderizarse correctamente sobre la capa base del tema activo.

#### Scenario: Seleccionar imagen local desde el explorador del SO
- **WHEN** el usuario hace clic en "Examinar..." en las preferencias de fondo y selecciona una imagen (.png, .jpg, .webp, etc.) con un tema activo
- **THEN** el sistema carga la imagen, la aplica al panel de terminal y la guarda como wallpaper de ese tema (no de los demás)

#### Scenario: Eliminar imagen de fondo
- **WHEN** el usuario hace clic en el botón "Quitar" en las preferencias de fondo
- **THEN** el sistema elimina solo el wallpaper del tema activo, limpia la capa de imagen de los paneles y deja los wallpapers de otros temas intactos

#### Scenario: Coexistencia con la sección de temas
- **WHEN** el usuario interactúa con el popover de preferencias
- **THEN** la interfaz presenta simultáneamente la sección de selección de temas y la sección de personalización de la imagen de fondo; cambiar de tema restaura el fondo guardado de ese tema

#### Scenario: Opacidad scoped al tema activo
- **WHEN** el usuario ajusta el slider de opacidad con el tema A activo
- **THEN** la opacidad se guarda para el tema A y al cambiar al tema B se aplica la opacidad (y fondo) de B

## ADDED Requirements

### Requirement: Wallpaper persistido por tema conceptual
El sistema SHALL mantener un mapa de wallpapers por id de tema en almacenamiento local del cliente. Al iniciar, si existen claves globales legacy de fondo, el sistema MUST migrarlas una sola vez al tema activo (o `nekossh`) y dejar de usarlas como fuente de verdad.

#### Scenario: Tema sin fondo configurado
- **WHEN** el usuario cambia a un tema que nunca tuvo wallpaper
- **THEN** el panel de terminal se muestra sin imagen de fondo (o con el overlay por defecto sin wallpaper)

#### Scenario: Migración desde claves globales
- **WHEN** la app arranca y encuentra `nekossh-bg-url` (u opacity) legacy sin mapa por tema migrado
- **THEN** esos valores se asignan al tema activo y las claves globales dejan de gobernar el fondo
