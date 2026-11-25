## ADDED Requirements

### Requirement: Telemetría Ambiental OSC 7, Conteo de Contexto e Inyección Pasiva en PTY
El backend del arnés SHALL rastrear metadatos ambientales no sensibles por sesión (`profile_name`, `current_cwd` extraído de secuencias `OSC 7` y `captured_lines` actuales), exponiéndolos hacia `"neko-ai"` y adjuntándolos automáticamente al contexto de turno de `run_ai_harness_turn_cmd`, además de ofrecer un canal de inserción pasiva en PTY sin ejecución automática.

#### Scenario: Seguimiento no invasivo de cwd OSC 7 y conteo de líneas en PtyTapRegistry
- **WHEN** la salida PTY incluye una secuencia `OSC 7` válida (p. ej. `\x1b]7;file://host/var/www/app\x07`, incluso fragmentada entre chunks) o líneas acumuladas bajo `Escucha ON`
- **THEN** `PtyTapRegistry` SHALL actualizar `current_cwd` por `terminal_id` sin retener bytes del flujo `ssh-stdout` cuando `Escucha` esté en `OFF`, y `ActiveShellInfoDto` SHALL exponer `current_cwd` y `captured_lines` reales hacia la ventana `"neko-ai"`

#### Scenario: Inyección automática de encabezado ambiental no sensible en turno agéntico
- **WHEN** se invoca `run_ai_harness_turn_cmd` para una terminal activa que tiene perfil conectado y opcionalmente un `current_cwd` detectado
- **THEN** el backend SHALL anteponer al contexto del turno (sanitizado por `SecretRedactor`) la referencia compacta del perfil y directorio actual evitando que el modelo deba consultar `pwd` inicialmente

#### Scenario: Inserción pasiva de comando sin salto de línea ni centinela
- **WHEN** se invoca `insert_ai_command_to_pty_cmd` con un comando válido
- **THEN** el backend SHALL eliminar retornos de carro o saltos de línea finales, rechazar entradas vacías y escribir el texto exacto sobre `write_ssh_raw` sin armar centinela POSIX ni alterar el estado de la FSM
