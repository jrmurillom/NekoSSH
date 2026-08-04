## ADDED Requirements

### Requirement: Desactivación de Autocompletado Nativo en Inputs
Todos los elementos `<input>` del layout de la aplicación (excepto de tipo hidden o file) SHALL tener configurado de forma explícita el atributo `autocomplete="off"` para mantener la apariencia y comportamiento de una aplicación de escritorio nativa pura.

#### Scenario: Foco en cualquier input de texto o búsqueda
- **WHEN** el usuario hace clic o enfoca cualquier campo de entrada de texto, número o búsqueda en la aplicación
- **THEN** el sistema no debe mostrar la lista desplegable de historial del navegador nativo.
