## Why

La ventana independiente `"neko-ai"` acumula actualmente cuatro franjas apiladas en la parte superior (cabecera de marca, selector de shell + controles rápidos, barra de 4 fases y área de tarjetas encajonadas), además de renderizar dos tarjetas con borde desde el estado vacío y encajonar cada respuesta conversacional del asistente. Esto reduce el área útil de lectura en monitores laterales (`500px–520px`) y genera fatiga visual al revisar tablas, logs y diagnósticos.

Adicionalmente, al cerrar únicamente la ventana `"neko-ai"` mientras `"main"` permanece abierto, el backend no cancela turnos de `agy` en curso ni desactiva la escucha PTY activa, dejando captura de terminal encendida sin ventana visible y perdiendo el hilo visual si el usuario vuelve a abrir `"neko-ai"` dentro de la misma sesión de la app.

## What Changes

- **Rediseño visual limpio estilo Copilot Pro en `"neko-ai"`**:
  - Consolidar la cabecera superior en una sola barra compacta (`~44px`) con marca `NekoAI`, selector de shell objetivo + botón *Pin Shell* iconográfico, *Limpiar conversación*, *Siempre visible* y *Ajustes*.
  - Eliminar las barras superiores redundantes (`copilot-toolbar` y `copilot-stepper`) moviendo el indicador contextual de fase operativa como etiqueta ligera en el encabezado de cada respuesta del asistente.
  - Hacer que las respuestas Markdown del asistente fluyan sin caja pesada (`background: transparent`, sin borde), dejando el encajonado con borde exclusivamente para las burbujas del usuario, errores, acordeones de consola y tarjetas interactivas que exigen autorización humana o rescate/rollback.
  - Unificar el estado vacío en un bloque hero minimalista centrado (resumen de escucha + accesos rápidos sin doble tarjeta).
  - Unificar el compositor inferior en un dock limpio (`textarea` multilínea + fila inferior con `Escucha ON/OFF`, `Modo Estricto/Flexible`, selector/refresco de modelo y botón `Enviar`).
- **Ciclo de vida limpio al cerrar `"neko-ai"` (Opción A)**:
  - Al destruirse la ventana `"neko-ai"` sin cerrar `"main"`, cancelar de inmediato cualquier turno `agy` en curso (`AiTurnCancelRegistry::cancel_all`) y apagar la escucha PTY en todas las sesiones activas (`PtyTapRegistry::disable_all_listening`), emitiendo `ai-shells-updated`.
  - Mantener en memoria RAM del proceso principal tanto el estado del arnés (`HarnessRegistry`) como un registro volátil del feed visual por sesión (`AiUiFeedRegistry`), permitiendo reabrir `"neko-ai"` con el historial intacto dentro de la misma ejecución de NekoSSH (limpiable con el botón de basurero).

## Capabilities

### New Capabilities
- *(Ninguna — todas las capacidades ya existen en `openspec/specs/`)*

### Modified Capabilities
- `ai-copilot-window`: Simplificación de la arquitectura visual de `"neko-ai"` (cabecera única, lienzo sin cajas en respuestas del asistente, estado vacío único y dock compositor inferior con controles operativos) y cierre aislado seguro (cancelación de turnos activos, apagado de escucha PTY y preservación de historial en memoria de sesión).

## Impact

- **Frontend (`app/`)**: `app/ai-copilot.html`, `app/src/ai-copilot/copilot.css`, `app/src/ai-copilot/copilot-state.ts`, `app/src/ai-copilot/copilot-state.test.ts`, `app/src/ai-copilot/copilot-main.ts`.
- **Backend (`app/src-tauri/`)**: `app/src-tauri/src/ai/pty_tap.rs`, `app/src-tauri/src/ai/orchestrator.rs` (o submódulo en `ai/` para `AiUiFeedRegistry`), `app/src-tauri/src/ai/commands.rs`, `app/src-tauri/src/ai/mod.rs`, `app/src-tauri/src/lib.rs`.
- **Documentación SSOT**: `docs/design/DESIGN.md` y `docs/design/ui-layout-contract.md` (sección de ventana independiente `"neko-ai"`).
