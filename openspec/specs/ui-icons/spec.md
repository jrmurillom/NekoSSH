# ui-icons

## Purpose

Sistema de iconos Lucide de contorno en la UI desktop (chrome: explorador, perfiles, pestañas). Color vía tokens de tema (`currentColor` / `--color-*`); accesibilidad en controles solo-icono.

## Requirements

### Requirement: Iconos Lucide de contorno en chrome UI
El sistema SHALL renderizar los iconos de la interfaz de chrome (botones de acción, nodos del explorador, acciones de perfil y cierre de pestaña) usando iconos [Lucide](https://lucide.dev/icons/) en variante de **contorno** (stroke), no rellenos sólidos. El sistema SHALL NOT usar emojis ni glifos Unicode como iconografía primaria de esas acciones.

#### Scenario: Botones del explorador con iconos outline
- **WHEN** el usuario ve la barra del explorador de archivos (Subir, Ir, Actualizar)
- **THEN** cada control muestra un icono Lucide de contorno en lugar de caracteres `↑` / `→` / `↻`

#### Scenario: Nodos del árbol con iconos de carpeta/archivo
- **WHEN** el explorador lista directorios y archivos
- **THEN** cada nodo muestra un icono Lucide outline de carpeta o archivo (no el texto `DIR`/`FIL` como icono primario)

#### Scenario: Acciones de perfil sin emoji
- **WHEN** el usuario ve las acciones Editar y Eliminar de un perfil
- **THEN** los controles usan iconos Lucide outline (p. ej. lápiz / papelera) en lugar de emojis

### Requirement: Color de iconos gobernado por tema
El sistema SHALL colorear los iconos Lucide mediante herencia CSS (`currentColor` / tokens `--color-*` del tema). El sistema SHALL NOT fijar colores de relleno o stroke hardcodeados en el markup del icono que ignoren el tema activo.

#### Scenario: Icono hereda color del contenedor
- **WHEN** un botón o fila aplica un color de texto/token de tema
- **THEN** el stroke del icono Lucide hijo coincide visualmente con ese color (vía `currentColor`)

#### Scenario: Estado danger/muted
- **WHEN** una acción usa clase de peligro o muted según el design system
- **THEN** el icono refleja el token correspondiente sin SVG con color fijo distinto al tema

### Requirement: Accesibilidad de controles solo-icono
El sistema SHALL preservar `title` y/o `aria-label` (o texto visible equivalente) en controles que muestren solo icono, de modo que el nombre de la acción siga siendo perceptible.

#### Scenario: Title en Subir/Ir/Actualizar
- **WHEN** el usuario enfoca o inspecciona los botones icónicos del explorador
- **THEN** cada control expone un nombre accesible (p. ej. “Subir”, “Ir”, “Actualizar”)
