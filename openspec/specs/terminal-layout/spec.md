# terminal-layout

## Purpose

Layout visual del área de terminal: fusión pestaña–panel con glow sakura, padding de seguridad frente al border-radius, cuadrícula interna de celdas y comportamiento nativo de los inputs del shell de la app.

## Requirements

### Requirement: Unified Terminal Tabs Visual Layout
La interfaz de terminal del cliente NekoSSH SHALL unificar visualmente la pestaña de sesión activa con el panel contenedor de la terminal, presentando un contorno continuo redondeado y con un glow sakura sutil.

#### Scenario: Fusión de Pestaña Activa y Consola
- **WHEN** el usuario visualiza la terminal de una sesión SSH activa
- **THEN** la pestaña activa se debe fundir con el borde superior del panel sin líneas divisorias intermedias, mostrando un color de fondo translúcido uniforme y un resplandor rosa sakura que rodea el bloque unificado.

#### Scenario: Pestañas Inactivas Apagadas
- **WHEN** una pestaña se encuentra en segundo plano (inactiva)
- **THEN** se debe renderizar de forma opaca y sin resplandor visual en su contorno para separarse del bloque de la terminal activa.

### Requirement: Terminal Container Padding Safety
El panel de la terminal SHALL incorporar un padding interno mínimo de seguridad de `24px` para proteger los caracteres de la consola de ser recortados por la curvatura de las esquinas del contenedor.

#### Scenario: Visualización del Texto en Esquinas Curvas
- **WHEN** los comandos se ejecutan y muestran resultados cerca de los bordes o esquinas inferiores del panel con `border-radius: 24px`
- **THEN** el texto de la terminal debe quedar completamente visible dentro de la zona de seguridad delimitada por el padding.

### Requirement: Desactivación de Autocompletado Nativo en Inputs
Todos los elementos `<input>` del layout de la aplicación (excepto de tipo hidden o file) SHALL tener configurado de forma explícita el atributo `autocomplete="off"` para mantener la apariencia y comportamiento de una aplicación de escritorio nativa pura.

#### Scenario: Foco en cualquier input de texto o búsqueda
- **WHEN** el usuario hace clic o enfoca cualquier campo de entrada de texto, número o búsqueda en la aplicación
- **THEN** el sistema no debe mostrar la lista desplegable de historial del navegador nativo.

### Requirement: Cuadrícula interna del panel unificado
El `.terminal-panel` de la pestaña activa SHALL contener una cuadrícula de celdas de terminal (hasta cuatro) sin romper la fusión visual pestaña–panel ni el glow sakura del contenedor unificado. El padding de seguridad del panel SHALL seguir evitando que el border-radius recorte el texto de cualquier celda visible.

#### Scenario: Panel con varias celdas mantiene glow unificado
- **WHEN** el contexto muestra dos o más shells en el grid
- **THEN** el resplandor y el borde redondeado envuelven el bloque completo del panel, no cada celda por separado

#### Scenario: Texto visible en celdas del grid
- **WHEN** hay output cerca del borde de una celda en layout 2×2
- **THEN** el texto permanece legible dentro del área útil (sin recorte por el radius del panel)
