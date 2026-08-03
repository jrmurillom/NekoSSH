## ADDED Requirements

### Requirement: Imagen de Fondo y Opacidad Exclusiva en la Terminal
El sistema NekoSSH MUST aplicar la imagen de fondo seleccionada y su regulador de opacidad (`#config-bg-opacity`) exclusivamente dentro del contenedor de la tarjeta de la terminal (`.terminal-panel`). El resto de la aplicación (barra lateral y contenedores globales) MUST permanecer con sus estilos originales sin alteraciones.

#### Scenario: Configurar fondo y ajustar opacidad en la terminal
- **WHEN** el usuario selecciona una imagen y ajusta el slider de opacidad en el panel de preferencias
- **THEN** la imagen se renderiza dentro del recuadro de la terminal y la opacidad regula la visibilidad de la imagen debajo del texto de la consola SSH
