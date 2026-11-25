## Context

La ventana `"neko-ai"` actual (`app/ai-copilot.html`, `app/src/ai-copilot/copilot.css`, `app/src/ai-copilot/copilot-main.ts`) evolucionó durante `ai-copilot-window`, `ai-guided-execution` y `ai-conversational-copilot-v2` agregando controles en múltiples franjas superiores:
1. `.copilot-header` (título + subtítulo + botones de ventana).
2. `.copilot-toolbar` (2 filas con selector de shell + botón *Pin Shell*, `Escucha ON/OFF`, `Modo Estricto/Flexible` y selector/refresco de modelo).
3. `.copilot-stepper` (barra dedicada de 4 pasos).
4. En estado vacío, dos `.copilot-empty-card` apiladas verticalmente; y en conversación, cada mensaje del asistente dentro de `.copilot-msg-card` con borde y fondo oscuro encajonado.

Por otro lado, `app/src-tauri/src/lib.rs` únicamente intercepta `WindowEvent::Destroyed` para `"main"`. Cuando el operador cierra `"neko-ai"`, los turnos activos de `agy` continúan hasta terminar o fallar en silencio, `PtyContextTap` permanece escuchando en segundo plano si estaba activo, y el arreglo `feedBySession` en memoria WebView se pierde al destruirse el WebView de `"neko-ai"`.

Restricciones técnicas vigentes (`docs/base-standards.md`, `DESIGN.md`, `ui-layout-contract.md`):
- Acentos planos sin *neon glow* (cero `box-shadow` luminosos), tokens heredados de `styles.css` para los 8 temas conceptuales, iconos Lucide outline (`currentColor`, cero emojis), `autocomplete="off"` en inputs y scrollbars temáticos.
- Sin cambios de esquema SQLite ni persistencia en disco del chat (solo memoria RAM del proceso principal).

## Goals / Non-Goals

**Goals:**
- Reducir el chrome fijo superior de `"neko-ai"` a **una sola barra compacta (`~44px`)** con identidad `NekoAI`, selector de shell + *Pin* iconográfico y controles de ventana (`Limpiar`, `Siempre visible`, `Ajustes`).
- Mover los controles operativos por turno (`Escucha ON/OFF`, `Modo Estricto/Flexible`, selector + refresco de modelo) al pie del **dock compositor unificado**, liberando el área de lectura.
- Renderizar respuestas del asistente sin caja pesada (`transparent` y sin borde), reservando tarjetas con borde para acciones que piden autorización, rescate/rollback y acordeones de consola.
- Garantizar cierre aislado seguro (**Opción A**) al destruir `"neko-ai"`: cancelar `AiTurnCancelRegistry`, apagar toda escucha en `PtyTapRegistry`, notificar `ai-shells-updated` y conservar `HarnessRegistry` + `AiUiFeedRegistry` en memoria RAM del proceso principal para rehidratar el chat al reabrir la ventana.

**Non-Goals:**
- No alterar la máquina de estados `AgentHarnessFsm`, el driver `agy`, el protocolo de centinela POSIX ni el renderizado de `markdown-lite.ts`.
- No persistir conversaciones de NekoAI en SQLite entre reinicios completos de la aplicación.

## Decisions

### 1. Reubicación de controles y jerarquía visual estilo Copilot Pro
- **Decisión:** Conservar todos los IDs funcionales clave en `ai-copilot.html` (`#copilot-shell-select`, `#btn-pin-shell`, `#btn-clear-chat`, `#btn-always-on-top`, `#btn-toggle-settings`, `#btn-toggle-listen`, `#btn-toggle-mode`, `#copilot-model-select`, `#btn-refresh-models`, `#copilot-prompt-input`, `#btn-send-prompt`), moviendo los controles de contexto/modelo al footer interno del compositor `.copilot-composer-dock` y pasando el indicador de fase operativa (`Fase 1 · Diagnóstico`, etc.) a una píldora sutil en `.copilot-msg-meta` mediante un helper puro en `copilot-state.ts` (`formatStagePill`).
- **Alternativa descartada:** Ocultar `Escucha`, `Modo` o `Modelo` dentro de un submenú desplegable. Se descartó porque el operador necesita visibilidad inmediata de si la terminal está siendo escuchada y qué modo de autorización aplica antes de enviar un prompt.

### 2. Renderizado diferencial en el feed y estado vacío único
- **Decisión:** Diferenciar `.copilot-msg-card.is-assistant` (transparente, sin borde, ancho completo legible dentro de `max-width: 820px`) frente a `.copilot-msg-card.is-user` (burbuja compacta alineada a la derecha) y `.copilot-action-card` / `.copilot-rollback-panel` (con borde y superficie glass). En estado vacío (`entries.length === 0`), consolidar privacidad de escucha y chips rápidos en un único bloque centrado `.copilot-empty-hero` con padding amplio y un botón contextual inline de `Activar Escucha` solo cuando `Escucha` está en `OFF`.
- **Alternativa descartada:** Mantener dos `.copilot-empty-card` apiladas con borde. Generaba ruido visual inmediato al abrir la ventana.

### 3. Cierre aislado Opción A con registro volátil `AiUiFeedRegistry` en backend
- **Decisión:**
  - Añadir `PtyTapRegistry::disable_all_listening(&self) -> bool` en `pty_tap.rs` (apaga `listening = false` y purga buffers en todos los taps).
  - Añadir `AiUiFeedRegistry` en `ai/` (mapa `Arc<Mutex<HashMap<String, Vec<AiFeedEntryDto>>>>` en memoria RAM manejado por Tauri `State`) con comandos `get_ai_feed_cmd`, `append_ai_feed_cmd` (invocados al añadir entradas o cambiar/recargar shell en `copilot-main.ts`) y limpieza explícita dentro de `clear_ai_conversation_cmd`.
  - En `lib.rs` -> `.on_window_event`, al recibir `WindowEvent::Destroyed` para `AI_COPILOT_WINDOW_LABEL`, ejecutar `ai_cancels.cancel_all()`, `pty_taps.disable_all_listening()` y emitir `"ai-shells-updated"` si hubo cambio, sin limpiar `HarnessRegistry` ni `AiUiFeedRegistry`.
- **Alternativa descartada:**
  - Guardar el feed en `localStorage`: filtraría conversaciones obsoletas entre reinicios completos de NekoSSH y duplicaría verdad fuera del backend.
  - No guardar el feed visual (solo `HarnessRegistry`): causaría desfase UX donde la ventana reabierta parece vacía aunque el modelo recuerde la sesión previa.

## Risks / Trade-offs

- **[Ancho estrecho en monitores de ~450px–520px]** → El footer del `.copilot-composer-dock` usa `flex-wrap: wrap` con límites `max-width` flexibles en `#copilot-model-select` y tooltips compactos para que nunca desborde horizontalmente.
- **[Crecimiento de memoria de `AiUiFeedRegistry` en sesiones muy largas]** → Acotar cada sesión a las últimas `MAX_FEED_ENTRIES_PER_SESSION = 150` entradas y limpiar al cerrar la sesión SSH correspondiente o pulsar Limpiar conversación.
- **[Condición de carrera al cerrar `"neko-ai"` justo cuando se dispara `append_ai_feed_cmd`]** → `AiUiFeedRegistry` vive en el estado global de la app Rust (`AppHandle`), por lo que las llamadas IPC en vuelo se completan o descartan de forma thread-safe bajo `Mutex` sin depender del WebView destruido.
