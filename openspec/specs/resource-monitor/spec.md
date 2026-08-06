# resource-monitor Specification

## Purpose
TBD - created by archiving change add-resource-monitor. Update Purpose after archive.
## Requirements
### Requirement: Pestaña de Monitorización en Barra Lateral
La barra lateral del frontend de NekoSSH SHALL incluir una tercera pestaña denominada "Monitor" (junto a "Servidores" y "Archivos") visible en todo momento.

#### Scenario: Visualización de Pestaña Monitor
- **WHEN** la aplicación es iniciada
- **THEN** la barra lateral muestra el botón "Monitor" en la zona de navegación de pestañas

### Requirement: Monitoreo de Servidor SSH Activo
El sistema SHALL monitorear el consumo de CPU, memoria RAM y almacenamiento en disco del servidor remoto al que se encuentre conectado el terminal activo.
- La CPU y la RAM SHALL visualizarse mediante gráficas de historial de línea (canvas sparkline) en tiempo real.
- El Disco Duro SHALL visualizarse mediante una barra de progreso clásica que indica el espacio usado sobre el total.
- Si no hay terminal activo o sesión SSH establecida, la pestaña SHALL mostrar un estado vacío informativo que indique conectar un servidor.

#### Scenario: Conexión activa iniciada
- **WHEN** el usuario se conecta a un servidor SSH
- **THEN** la pestaña Monitor muestra el nombre del servidor activo, el indicador de latencia activo, y renderiza las gráficas de CPU/RAM y la barra de Disco con datos actualizados

#### Scenario: Sin sesión SSH activa
- **WHEN** no hay ninguna sesión de terminal activa abierta
- **THEN** la pestaña Monitor muestra el panel de estado vacío con el mensaje "Conéctate a un servidor para ver los recursos del sistema en tiempo real."

### Requirement: Controles de Pausa e Intervalo de Refresco
El sistema SHALL proveer controles en la parte inferior de la pestaña Monitor para interactuar con la captura de métricas:
- Un botón para Pausar/Reanudar el monitoreo en tiempo real.
- Un menú desplegable para elegir el intervalo de refresco entre 2s, 5s y 10s.

#### Scenario: Pausar monitoreo
- **WHEN** el usuario hace clic en el botón "Pausar"
- **THEN** la captura de métricas en segundo plano se suspende y el botón cambia su etiqueta a "Reanudar"

#### Scenario: Cambiar intervalo de refresco
- **WHEN** el usuario selecciona un nuevo intervalo de refresco en el menú desplegable
- **THEN** el temporizador en segundo plano cambia su frecuencia de disparo al valor seleccionado

