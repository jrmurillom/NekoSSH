## ADDED Requirements

### Requirement: Inhabilitación del Menú Contextual Nativo del Navegador
El cliente NekoSSH SHALL bloquear la aparición del menú contextual predeterminado del motor WebView2/browser (`contextmenu` nativo del sistema operativo) en todas las áreas de la aplicación donde no existan componentes con menús contextuales de producto.

#### Scenario: Clic derecho en zona neutra de la aplicación
- **WHEN** el usuario realiza clic derecho en un área sin menú contextual personalizado (barra de título, headers, fondo de ventana, modales, etc.)
- **THEN** el sistema previene el evento nativo y no despliega el menú contextual del navegador
