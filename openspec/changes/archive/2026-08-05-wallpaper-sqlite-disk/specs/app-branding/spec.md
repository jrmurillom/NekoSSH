## MODIFIED Requirements

### Requirement: Personalización y CRUD de Imagen de Fondo
El cliente NekoSSH SHALL permitir a los usuarios seleccionar, aplicar, regular la opacidad y eliminar una imagen de fondo personalizada (desde URL o archivo local del SO) mediante el popover de preferencias, el cual recibe una sección de selección de temas que coexiste con estos ajustes. La imagen, la etiqueta mostrada y la opacidad SHALL persistirse **asociadas al tema conceptual activo** en SQLite (metadatos) y, para archivos locales, como copia en el directorio de datos de la aplicación; cambios de fondo MUST NOT alterar el wallpaper de otros temas. Las imágenes de archivo local MUST renderizarse vía protocolo de assets / `convertFileSrc` sobre la ruta de la copia en el data dir — MUST NOT persistirse como data URL en `localStorage`. Las URLs `http`/`https` MAY persistirse como URL en la base de datos sin copiar el binario a disco.

#### Scenario: Seleccionar imagen local desde el explorador del SO
- **WHEN** el usuario hace clic en "Examinar..." en las preferencias de fondo y selecciona una imagen (.png, .jpg, .webp, etc.) con un tema activo
- **THEN** el sistema copia la imagen al data dir, aplica el fondo al panel de terminal y guarda metadatos (label, opacidad, referencia de archivo) como wallpaper de ese tema en SQLite

#### Scenario: Eliminar imagen de fondo
- **WHEN** el usuario hace clic en el botón "Quitar" en las preferencias de fondo
- **THEN** el sistema elimina solo el wallpaper del tema activo (fila en BD y archivo en disco si existía), limpia la capa de imagen de los paneles y deja los wallpapers de otros temas intactos

#### Scenario: Coexistencia con la sección de temas
- **WHEN** el usuario interactúa con el popover de preferencias
- **THEN** la interfaz presenta simultáneamente la sección de selección de temas y la sección de personalización de la imagen de fondo; cambiar de tema restaura el fondo guardado de ese tema

#### Scenario: Opacidad scoped al tema activo
- **WHEN** el usuario ajusta el slider de opacidad con el tema A activo
- **THEN** la opacidad se guarda para el tema A en SQLite y al cambiar al tema B se aplica la opacidad (y fondo) de B

#### Scenario: Imagen local grande persiste tras reinicio
- **WHEN** el usuario elige un archivo local cuyo tamaño excedería el cupo típico de `localStorage` y reinicia la aplicación
- **THEN** el wallpaper del tema activo se restaura desde BD + disco sin requerir volver a seleccionar el archivo

### Requirement: Wallpaper persistido por tema conceptual
El sistema SHALL mantener una fila por id de tema en la tabla SQLite de wallpapers (metadatos + referencia a archivo o URL remota). MUST NOT usar `localStorage` como fuente de verdad del mapa de fondos. Al iniciar, si existen claves legacy (`nekossh-bg-by-theme` o `nekossh-bg-url` / label / opacity), el sistema MUST migrarlas una sola vez hacia SQLite + disco (cuando aplique) y dejar de usarlas.

#### Scenario: Tema sin fondo configurado
- **WHEN** el usuario cambia a un tema que nunca tuvo wallpaper
- **THEN** el panel de terminal se muestra sin imagen de fondo (o con el overlay por defecto sin wallpaper)

#### Scenario: Migración desde mapa localStorage por tema
- **WHEN** la app arranca y encuentra `nekossh-bg-by-theme` con entries y la tabla de wallpapers aún no tiene esas filas
- **THEN** cada entry migrable se materializa en SQLite (y archivo en disco si era data URL o path local) y las claves de `localStorage` del mapa dejan de gobernar el fondo

#### Scenario: Migración desde claves globales legacy
- **WHEN** la app arranca y encuentra `nekossh-bg-url` (u opacity) legacy sin fila migrada para el tema destino
- **THEN** esos valores se asignan al tema activo (o `nekossh`) en SQLite/disco y las claves globales dejan de gobernar el fondo
