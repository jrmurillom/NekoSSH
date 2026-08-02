## ADDED Requirements

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
