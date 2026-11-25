## Context

La prueba en vivo sobre servidor real demostró dos fallas estructurales del diseño anterior:
1. **Bloqueo duro contraproducente:** `ReadOnlyEnforcer` lanzaba `Err` al ver `2>/dev/null` o cualquier construcción no trivial, destruyendo el turno incluso cuando el usuario estaba en `Modo Estricto` listo para autorizar manualmente.
2. **Metodología forzada e inferior al flujo real de un SysAdmin Senior:** Congelar un arreglo estático de $N$ tareas en Fase 2 impedía adaptar el paso $k+1$ a la salida real del paso $k$, sacrificaba la riqueza diagnóstica en Markdown y omitía dos prácticas críticas de producción: **validación sintáctica pre-reinicio (*dry-run*, ej. `nginx -t`)** y **Fase 4 de certificación post-cambio (*health check*: `is-active`, `ss -tulpn`, `journalctl`)**.

Este diseño reemplaza la burocracia interna manteniendo intactos los cimientos probados de seguridad y aislamiento (`AgyCliDriver` en sandbox, `PtyContextTap` opt-in, centinela POSIX `%s_%s:%d__` inmune a eco TTY, `SecretRedactor` y `CommandRiskGuard`).

## Goals / Non-Goals

**Goals:**
- Implementar el **Ciclo Operativo Adaptativo de 4 Fases** (*1. Diagnóstico*, *2. Propuesta con Causa Raíz/Backup/Rollback*, *3. Ejecución Progresiva de 1 Comando por Turno con Dry-Run*, *4. Certificación Health Check*) que además responde directo sin burocracia ante preguntas informativas simples.
- Transformar `ReadOnlyEnforcer` + `CommandRiskGuard` en un **clasificador y generador de alertas no bloqueante (`audit_warning`)**: reconoce `2>/dev/null`, `>/dev/null` y `2>&1` como lectura pura, auto-aprueba **únicamente** lecturas `🟢` verificadas en `Modo Flexible` y delega todo lo demás a la tarjeta humana **`[ Autorizar ]` / `[ Editar ]` / `[ Copiar ]` / `[ Omitir ]`** sin lanzar errores rojos ni requerir mantenimiento infinito de *whitelists*.
- Sembrar instantáneamente el contenido visible actual de `xterm.js` al activar `Escucha ON` para que el operador pueda preguntar sobre errores ya presentes en pantalla.
- Elevar la UI de `"neko-ai"` a estándar comercial: Markdown técnico seguro con tablas y copiar código, salidas de consola colapsables, compositor `<textarea>` multilínea, chips iniciales y ancho de lectura centrado (`max-width: 880px`).
- Validar de extremo a extremo contra **`agy.EXE` real** con múltiples turnos antes de cerrar el change.

**Non-Goals:**
- Ejecución desatendida de comandos mutantes (`🟡 / 🟠 / 🔴`): prohibida siempre en ambos modos.
- Dependencias pesadas externas de terceros para Markdown (se implementa un parser propio determinista con escapado previo anti-XSS cubierto con `vitest`).

## Decisions

### 1. Contrato Conversacional Unificado (`AgentTurnWireEnvelope` v2) con Degradación Elegante
- **Decisión:** Simplificar el esquema JSON solicitado al proveedor para maximizar la calidad de razonamiento en `message` (Markdown técnico completo) más un objeto opcional `step` (1 comando por turno):
  ```json
  {
    "phase": "diagnosis | proposal | execution | verification | direct_answer",
    "message": "Explicación técnica rica en Markdown (causa raíz, hallazgos, tablas, riesgos)...",
    "step": {
      "purpose": "identificar la distribución Linux y versión exacta del kernel",
      "command": "cat /etc/os-release 2>/dev/null || uname -a",
      "risk_level": "read_only",
      "backup_strategy": null,
      "rollback_cmd": null
    }
  }
  ```
- **Resiliencia:** Si `step` es `null`, es una respuesta conversacional pura o una propuesta conceptual en espera de visto bueno. Si `CommandRiskGuard` eleva `risk_level` y `rollback_cmd` está vacío, **no se rechaza el JSON**: Rust conserva el turno y rellena `audit_warning` explicando exactamente qué detectó. Si el proveedor devolviera texto plano sin JSON válido, `AgyCliDriver` lo envuelve como respuesta conversacional `direct_answer` en lugar de arrojar error.

### 2. Parser Consciente de Descriptores POSIX en `risk_guard.rs` + Clasificador No Bloqueante
- **Decisión:**
  - Añadir un analizador léxico de redirecciones en `risk_guard.rs` que distingue redirecciones seguras hacia `/dev/null` o entre descriptores estándar (`>/dev/null`, `1>/dev/null`, `2>/dev/null`, `&>/dev/null`, `2>&1`, `1>&2`, con o sin espacio previo al destino `/dev/null`) frente a redirecciones reales hacia rutas de archivo.
  - Exponer `CommandRiskGuard::evaluate_proposal(ai_risk, command) -> ProposalSafetyReport { effective_risk, auto_runnable_in_flexible, audit_warning }`:
    - `auto_runnable_in_flexible = true` **solo** cuando `effective_risk == ReadOnly` y `ReadOnlyEnforcer::check_pure_readonly(command)` es `Ok(())`.
    - Cuando `check_pure_readonly` devuelve `Err(reason)` o `effective_risk != ReadOnly`, `auto_runnable_in_flexible = false` y `audit_warning = Some(reason)`.
  - `execute_ai_command_cmd` (comando unificado de ejecución autorizada por humano o auto-lectura verificada) envuelve con `wrap_command_with_sentinel` e inyecta en el PTY respetando la decisión soberana del operador cuando `human_authorized == true`.

### 3. Sembrado Instantáneo desde `xterm.js` al Encender `Escucha ON`
- **Decisión:**
  - En `"main"` (`app/src/main.ts`), exponer un listener de evento interno / sincronización cuando cambia la escucha (`ai-request-visible-buffer`) o adjuntar un helper `extractVisibleBufferText(term, maxLines)` que lee `term.buffer.active` desde el viewport/base hacia abajo, limpiando líneas vacías finales, e invoca `seed_pty_initial_context_cmd(terminalId, visibleText)`.
  - En Rust (`pty_tap.rs`), `seed_initial_snapshot` solo escribe si `tap.listening == true` y el buffer estaba vacío; si `set_listening(false)` se invoca en cualquier instante, `clear_buffers()` borra tanto lo sembrado como lo capturado (`0 bytes`).

### 4. Retorno Progresivo Automático tras cada Comando y Botón de Rescate (`last_rollback`)
- **Decisión:**
  - Tanto en lecturas como en pasos de Fase 3 o Fase 4, cuando `ai-pty-command-finished` llega con el `exit_code` y `captured_output` sanitizado:
    - Si `exit_code == 0`: se agrega el bloque colapsable `✓ Comando completado (exit=0)` al feed y se envía automáticamente el resultado real al copiloto para que continúe con la siguiente acción progresiva (ej. tras `cp ... .bak` propone la edición; tras la edición propone `nginx -t`; tras `systemctl reload` lanza Fase 4 `is-active` + `ss -tulpn` + `journalctl`; o tras una lectura simple entrega la conclusión final). Un contador `MAX_AUTO_CHAIN_STEPS = 6` evita bucles infinitos en `Modo Flexible`.
    - Si `exit_code != 0`: frena cualquier auto-cadena, muestra el bloque expandido en coral `✕ Fallo detectado (exit=<code>)`, activa en primera fila el botón **`Ejecutar Rollback`** (si el comando fallido o el paso mutante inmediato anterior registró un `rollback_cmd`) y ofrece **`Analizar fallo con IA`** con 1 clic.

### 5. Renderizador `markdown-lite.ts` Seguro contra XSS + Layout Centrado
- **Decisión:** Crear `app/src/ai-copilot/markdown-lite.ts` puro y cubierto con `vitest`:
  1. Escapa primero `&`, `<`, `>`, `"`, `'` sobre todo el texto de entrada.
  2. Procesa bloques cercados ```` ```lang ... ``` ```` generando contenedor con cabecera de lenguaje y botón `data-copy-code` (**Copiar** vía `@tauri-apps/plugin-clipboard-manager` / `navigator.clipboard`).
  3. Procesa tablas GFM (`| col | col |`), encabezados `##`/`###`, listas `-`/`1.`, citas `>` (alertas técnicas), negritas `**...**` y código inline `` `...` ``.
  4. En `copilot.css`, centrar el contenido interno de toolbar, feed y compositor con `max-width: 880px; margin: 0 auto; width: 100%` para que luzca proporcionado tanto a `480px` en monitor lateral como maximizado a `1920px`.

## Risks / Trade-offs

- **[Riesgo] Bucle de lecturas encadenadas si un servidor devuelve salidas ambiguas en `Modo Flexible`** → **Mitigación:** Tope estricto `MAX_AUTO_CHAIN_STEPS = 6` por consulta del operador; al alcanzarlo, la siguiente lectura pausa y pide clic humano.
- **[Riesgo] `xterm.js` con miles de líneas históricas al activar `Escucha ON`** → **Mitigación:** `extractVisibleBufferText` extrae únicamente las últimas `context_max_lines` líneas no vacías (por defecto `200`) y pasa obligatoriamente por `SecretRedactor` antes de salir hacia el driver.
