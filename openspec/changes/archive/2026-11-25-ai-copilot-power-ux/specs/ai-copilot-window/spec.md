## ADDED Requirements

### Requirement: Aceleradores de Usabilidad Comercial y Productividad en neko-ai y Main Terminal
La ventana principal `"main"` y la ventana independiente `"neko-ai"` SHALL proporcionar flujos de trabajo de alta velocidad sin comprometer la privacidad opt-in (`Escucha OFF`) ni la legibilidad minimalista del HUD.

#### Scenario: Preguntar a NekoAI sobre selección de terminal mediante Ctrl+Shift+E incluso con Escucha OFF
- **WHEN** el operador selecciona texto en una terminal conectada de `"main"` y presiona `Ctrl+Shift+E` (teniendo `Escucha` en `OFF` u `ON`)
- **THEN** el sistema SHALL abrir o enfocar `"neko-ai"` y precargar el texto seleccionado dentro de `#copilot-prompt-input` como bloque citado listo para editar o enviar, sin activar la escucha continua ni sufrir pérdida por carrera de inicialización de ventana

#### Scenario: Botón Insertar en terminal sin salto de línea en tarjeta de comando
- **WHEN** existe una tarjeta de comando pendiente en `"neko-ai"` y el operador hace clic en **Insertar en terminal**
- **THEN** `"neko-ai"` SHALL invocar `insert_ai_command_to_pty_cmd`, enviando el texto limpio al PTY activo de la sesión objetivo **sin** añadir `\n`, `\r` ni wrapper centinela para que el operador pueda completarlo o revisarlo antes de presionar `Enter`

#### Scenario: Vista previa de streaming limpia sin exponer JSON crudo
- **WHEN** el proveedor `agy` emite deltas incrementales del objeto `--json-schema` (`{"kind":"propose_step","message":"...`) durante `"Analizando con agy…"`
- **THEN** `#copilot-progress-stream` SHALL extraer y mostrar únicamente el texto legible del campo de respuesta/razonamiento (o `"Preparando respuesta técnica…"` mientras llegan metadatos JSON previos), evitando imprimir llaves JSON crudas en pantalla

#### Scenario: Indicador vivo de líneas capturadas, auto-ajuste de compositor e historial con flechas
- **WHEN** `Escucha` está en `ON` sobre el shell objetivo, el operador pega texto extenso en `#copilot-prompt-input`, o presiona `ArrowUp` / `ArrowDown` con el compositor vacío o navegando historial previo
- **THEN** la micro-etiqueta del chip SHALL mostrar `Escucha ON · N líneas`, el `<textarea>` SHALL auto-ajustar su altura suavemente hasta `140px`, y las teclas `ArrowUp` / `ArrowDown` SHALL recuperar prompts previos enviados en la sesión conservando cualquier borrador en curso
