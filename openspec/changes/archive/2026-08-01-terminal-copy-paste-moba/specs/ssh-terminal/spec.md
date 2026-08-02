## ADDED Requirements

### Requirement: Auto-copiar selección en la terminal
Cuando el usuario selecciona texto en el viewport xterm de una sesión, el sistema SHALL copiar automáticamente ese texto al clipboard del sistema vía la API nativa de clipboard de Tauri (plugin clipboard-manager). Una selección vacía NO MUST escribir al clipboard. Ctrl+C NO MUST redefinirse como copiar: MUST seguir disponible para el shell remoto (interrupt). El producto NO MUST depender del prompt de permiso de `navigator.clipboard` del WebView para este gesto.

#### Scenario: Copiar al soltar selección
- **WHEN** el usuario selecciona uno o más caracteres visibles en el emulador xterm
- **THEN** el texto seleccionado queda en el clipboard del sistema

#### Scenario: Selección vacía
- **WHEN** no hay texto seleccionado en el emulador
- **THEN** el sistema no sobrescribe el clipboard por este gesto

#### Scenario: Ctrl+C no es copy de producto
- **WHEN** el usuario pulsa Ctrl+C con sesión SSH activa
- **THEN** el cliente no consume la pulsación como “copiar selección”; el comportamiento hacia el remoto permanece el de interrupt/input normal

#### Scenario: Sin prompt de permiso WebView al copiar
- **WHEN** el usuario selecciona texto en la terminal dentro de la app Tauri
- **THEN** la copia al clipboard del SO ocurre sin diálogo de permiso del WebView por `navigator.clipboard`

### Requirement: Pegar con clic derecho y strip del final
El clic derecho sobre el viewport xterm de una terminal SHALL pegar el contenido del clipboard hacia el PTY (mismo canal de entrada que el teclado), leyendo el clipboard con la API nativa de Tauri (plugin clipboard-manager). Antes de enviar, el sistema MUST eliminar únicamente saltos de línea y/o whitespace **al final** del texto pegado (p. ej. `\n`, `\r`, `\r\n`, espacios/tabs finales). Los saltos de línea en medio del texto MUST conservarse. Este gesto NO MUST abrir un menú contextual en la terminal. El producto NO MUST depender del prompt de permiso de `navigator.clipboard` del WebView para este gesto.

#### Scenario: Pegar con clic derecho
- **WHEN** el usuario hace clic derecho en el canvas de la terminal con texto en el clipboard
- **THEN** ese texto (tras el strip del final) se envía al PTY como input

#### Scenario: Quitar Enter solo al final
- **WHEN** el clipboard contiene `ls -la` seguido de un salto de línea final
- **THEN** al pegar con clic derecho se envía `ls -la` sin el salto final

#### Scenario: Multilínea interna intacta
- **WHEN** el clipboard contiene varias líneas con Enter entre ellas y opcionalmente Enter al final
- **THEN** los Enter entre líneas se conservan y solo se elimina el trailing del final del texto completo

#### Scenario: Sin menú contextual en terminal
- **WHEN** el usuario hace clic derecho en el viewport xterm
- **THEN** no aparece un menú contextual de chrome; se realiza el paste (o nada si el clipboard no se puede leer)

#### Scenario: Sin prompt de permiso WebView al pegar
- **WHEN** el usuario hace clic derecho para pegar en la terminal dentro de la app Tauri
- **THEN** la lectura del clipboard del SO ocurre sin diálogo de permiso del WebView por `navigator.clipboard`
