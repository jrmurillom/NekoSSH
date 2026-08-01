## ADDED Requirements

### Requirement: Aviso de sesión SSH desconectada
Cuando la Session SSH asociada a una pestaña de terminal termina de forma no solicitada por el usuario (EOF, error de transporte, cierre remoto), el sistema SHALL mostrar en el viewport de esa terminal un mensaje claro de desconexión e indicar que puede reconectar con **Ctrl+R**. El indicador de estado de la pestaña (dot + texto) SHALL pasar a un estado de desconectado (o error, si el cierre fue por fallo de conexión).

#### Scenario: Sesión muerta por el servidor
- **WHEN** el backend emite el evento de cierre/error para un `terminal_id` que estaba conectado
- **THEN** la terminal muestra un aviso de desconexión que incluye la pista de Ctrl+R, marca `isConnected` en falso y actualiza el indicador a desconectado (o error)

#### Scenario: Cierre voluntario de pestaña
- **WHEN** el usuario cierra la pestaña de terminal
- **THEN** el sistema no presenta el flujo de “Ctrl+R para reconectar” como acción disponible (la pestaña deja de existir)

### Requirement: Reconexión manual con Ctrl+R
El sistema SHALL permitir reconectar la misma pestaña de terminal con el mismo perfil de conexión cuando el usuario pulsa **Ctrl+R** y esa pestaña está desconectada. Mientras la sesión esté activa, Ctrl+R NO MUST interceptarse como reconexión (debe poder llegar al shell remoto). La reconexión SHALL reutilizar el `terminal_id` de la pestaña y el perfil asociado a ella. El sistema SHALL evitar lanzar dos reconexiones concurrentes sobre la misma pestaña.

#### Scenario: Reconectar tras desconexión
- **WHEN** la pestaña está desconectada y el usuario pulsa Ctrl+R
- **THEN** el sistema inicia de nuevo la sesión SSH con el perfil de esa pestaña, actualiza el indicador a conectando y, si tiene éxito, a conectado

#### Scenario: Ctrl+R con sesión viva
- **WHEN** la pestaña está conectada y el usuario pulsa Ctrl+R
- **THEN** el cliente no dispara el flujo de reconexión NekoSSH (la pulsación no se consume como reconnect)

#### Scenario: Indicador durante reconexión
- **WHEN** el usuario inicia reconexión con Ctrl+R
- **THEN** el indicador bajo la pestaña refleja estado de conectando hasta el resultado (conectado o error)
