# conceptual-themes

## Purpose

Define el sistema de temas conceptuales del cliente NekoSSH, permitiendo cambiar dinámicamente la paleta de colores de la interfaz y la terminal a través de propiedades personalizadas de CSS.

## Requirements

### Requirement: Theme Infrastructure
El sistema SHALL soportar múltiples temas visuales mediante propiedades personalizadas de CSS (CSS custom properties) aplicando un atributo `data-theme` en el elemento `<html>`. El tema predeterminado MUST ser 'nekossh' (Cyber-Sakura).

#### Scenario: Cambio dinámico de atributos
- **WHEN** el sistema inicializa o cambia el tema activo
- **THEN** el atributo `data-theme` del elemento `<html>` se actualiza al nombre del nuevo tema (ej. `data-theme="nekossh"`).

### Requirement: Theme Persistence
El tema seleccionado SHALL persistir en localStorage bajo la clave `nekossh-theme` y MUST ser restaurado cuando la aplicación se carga.

#### Scenario: Restauración de tema al inicio
- **WHEN** el usuario inicia la aplicación NekoSSH
- **THEN** el sistema lee el valor de `nekossh-theme` de localStorage y lo aplica al `<html>`, restaurando la selección previa.

### Requirement: Theme Selector UI
El popover de preferencias (`#prefs-popover`) SHALL incluir una sección de selección de temas con una lista etiquetada. Cada elemento de tema SHALL mostrar un círculo de vista previa de colores divididos (mostrando los 2 colores dominantes del tema) y el nombre del tema.

#### Scenario: Interacción con el selector de temas
- **WHEN** el usuario abre el popover de preferencias
- **THEN** se visualiza una lista de temas, cada uno con un círculo de vista previa bicolor y su nombre. Al hacer clic en un tema, se aplica de inmediato.

### Requirement: Terminal Color Sync
Los colores del terminal de xterm.js SHALL ser leídos desde las propiedades personalizadas de CSS del tema activo en el momento de la creación del terminal y SHALL ser actualizados cuando el tema cambia, en lugar de utilizar valores preprogramados (hardcoded).

#### Scenario: Actualización de colores del terminal xterm.js
- **WHEN** ocurre un cambio en el tema activo a través de la UI
- **THEN** el sistema extrae los nuevos valores de color de las propiedades de CSS del documento y aplica estos valores al objeto de configuración de tema del terminal xterm.js.

### Requirement: Theme Catalog
El sistema SHALL incluir exactamente 8 temas conceptuales: NekoSSH (default), Hatsune Miku, Rei Ayanami, Neon Evangelion (Unidad-01), Cyberpunk David, Cyberpunk Lucy, Persona 5 (Phantom Thieves), y Sailor Moon Serena.

#### Scenario: Disponibilidad del catálogo de temas
- **WHEN** el usuario navega por las opciones de temas en las preferencias
- **THEN** el catálogo muestra exactamente las 8 opciones conceptuales definidas listas para ser seleccionadas.

### Requirement: Hardcoded Color Elimination
Todos los valores literales de color preprogramados (hardcoded) en CSS y JS que correspondan a tokens del tema SHALL ser reemplazados con referencias a propiedades personalizadas de CSS.

#### Scenario: Eliminación de colores estáticos
- **WHEN** un componente de la interfaz de usuario se renderiza
- **THEN** sus estilos utilizan las variables CSS (ej. `var(--primary-color)`) asegurando consistencia con el tema actualmente activo, sin colores "hardcoded".

### Requirement: Brand Logo Sync with Theme
Al aplicar o restaurar un tema conceptual, el sistema SHALL actualizar el logo de marca del sidebar (`.brand-logo`) para usar el PNG asociado al id del tema activo, en el mismo ciclo en que se actualizan `data-theme` y los colores de xterm.js.

#### Scenario: Cambio de tema actualiza el logo
- **WHEN** el usuario selecciona un tema distinto en el popover de preferencias
- **THEN** el `src` del logo del sidebar apunta al PNG de ese tema y el título/acentos CSS ya reflejan la misma paleta

#### Scenario: Restauración de tema también restaura el logo
- **WHEN** la aplicación inicia y restaura el tema desde `localStorage`
- **THEN** el logo del sidebar coincide con el tema restaurado sin requerir un clic adicional del usuario
