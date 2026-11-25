## Why

Al probar NekoAI en uso real contra un servidor SSH, quedó en evidencia que la máquina de estados rígida con planes pre-congelados y validadores que abortan el turno (`Err`) rompía la fluidez operativa (rechazando incluso sintaxis estándar POSIX como `2>/dev/null`), empobrecía las explicaciones técnicas y ejecutaba listas estáticas sin inspeccionar la salida real entre pasos ni certificar la salud final del servicio. Se requiere evolucionar hacia un **Copiloto Conversacional Asistido de 4 Fases con Ejecución Progresiva (un comando por turno)** donde Rust clasifica riesgos y protege el PTY sin bloquear jamás la conversación ni depender de *whitelists* manuales.

## What Changes

- **BREAKING (Arquitectura interna del arnés): De FSM burocrática pre-congelada a Copiloto Conversacional Progresivo de 4 Fases (`prompt.rs`, `schema.rs`, `fsm.rs`, `orchestrator.rs`):**
  - Adopta la disciplina operativa de **4 Fases adaptativas**:
    1. **Fase 1 — Diagnóstico y Telemetría No Intrusiva** (cero suposiciones de stack; si la consulta es solo informativa, responde directo con riqueza técnica sin forzar fases posteriores).
    2. **Fase 2 — Diagnóstico de Causa Raíz, Respaldo y Rollback** (explica causa raíz, advierte riesgos de red/SSH/servicios, exige respaldo fechado `cp ... .bak.$(date +%F_%T)` y define `rollback_cmd`).
    3. **Fase 3 — Ejecución Progresiva de Un Comando por Turno + Dry-Run Pre-Reinicio** (entrega un único comando ejecutable por turno esperando el `exit_code` y output real antes de formular el siguiente, exigiendo validación sintáctica previa como `nginx -t`, `sshd -t`, `apache2ctl configtest`, `visudo -c`).
    4. **Fase 4 — Verificación y Health Check Post-Cambio** (certifica estabilidad real con `systemctl is-active`, `ss -tulpn` y `journalctl -n 20 --no-pager`).
  - Mantiene memoria conversacional continua durante toda la sesión SSH (un objetivo resuelto no borra el contexto ya aprendido del servidor; se añade acción explícita **Limpiar conversación** cuando el usuario desee reiniciar de cero).
- **Auditoría de Seguridad No Bloqueante y Cero Dependencia de Whitelists (`risk_guard.rs`, `orchestrator.rs`, `commands.rs`):**
  - Reconoce redirecciones estándar inocuas POSIX (`2>/dev/null`, `>/dev/null`, `1>/dev/null`, `&>/dev/null`, `2>&1`, `1>&2`) como `ReadOnly` sin elevarlas falsamente a escritura en disco.
  - Elimina todo bloqueo duro (`Err`) sobre propuestas de comandos: si un comando usa binarios desconocidos/propios, subshells `$()`, redirecciones reales o verbos mutantes, el backend **jamás aborta el turno**; en su lugar adjunta una nota explicativa de auditoría (`audit_warning`), desactiva la auto-ejecución y delega la decisión en la tarjeta humana **`[ Autorizar ]` / `[ Editar ]` / `[ Copiar ]` / `[ Omitir ]`**.
  - Degradación elegante del sobre de respuesta: ningún campo opcional ausente o texto libre del proveedor destruye la respuesta con pantalla roja.
- **Conciencia Instantánea de Pantalla Visible en `Escucha ON` (`main.ts`, `commands.rs`, `pty_tap.rs`):**
  - Al activar `Escucha ON` (y al sincronizar consola activa), `"main"` provee las últimas líneas visibles actuales del buffer de `xterm.js` sanitizadas por `SecretRedactor`, permitiendo preguntar *"¿por qué falló esto que tengo en pantalla?"* al instante sin repetir el comando. Al pasar a `Escucha OFF`, se purga todo a `0 bytes`.
- **Experiencia Visual y Riqueza Ejecutiva de Nivel Comercial en `"neko-ai"` (`ai-copilot.html`, `copilot.css`, `copilot-main.ts`, `markdown.ts`):**
  - Renderizador Markdown técnico propio seguro contra XSS (encabezados, viñetas, negritas, tablas de telemetría y bloques de código con botón **Copiar**).
  - Bloques de salida de consola **colapsables** con insignia `exit=0` / `exit!=0`.
  - Tarjeta de acción conversacional clara (*"¿Autorizas ejecutar este comando para `<propósito>`?"* + insignia de severidad + `audit_warning` + respaldo/rollback visible + rescate de 1 clic si `exit != 0` + `Ctrl+C` en vivo).
  - Compositor `<textarea>` multilínea (`Enter` envía, `Shift+Enter` salto de línea), chips rápidos de diagnóstico inicial y contenedor centrado legible (`max-width: 880px`) armonizado con los 8 temas.

## Capabilities

### New Capabilities
*(Ninguna carpeta nueva; se evolucionan integralmente las tres capacidades agénticas existentes).*

### Modified Capabilities
- `ai-harness-core`: Sustituye el candado bloqueante y la FSM rígida por clasificación no bloqueante con `audit_warning`, soporte POSIX `2>/dev/null` / `2>&1`, Constitución de 4 Fases con Dry-Run y Health Check, degradación elegante de esquema y memoria continua por sesión.
- `ai-copilot-window`: Añade renderizado Markdown técnico seguro (incluyendo tablas y copiar código), salidas de consola colapsables, sembrado instantáneo del buffer visible de `xterm.js` al encender `Escucha ON`, compositor multilínea, chips rápidos y layout centrado adaptable.
- `ai-guided-execution`: Evoluciona a ejecución progresiva de **un comando por turno** alimentada por el output real del centinela, tarjeta explicativa unificada con edición/copia/`Ctrl+C` y botón inmediato de rescate (`rollback_cmd`) ante fallos.

## Impact

- **Backend (`app/src-tauri/src/`):** `ai/prompt.rs`, `ai/schema.rs`, `ai/risk_guard.rs`, `ai/fsm.rs`, `ai/orchestrator.rs`, `ai/pty_tap.rs`, `ai/commands.rs`, `ai/drivers/agy.rs`, `lib.rs`.
- **Frontend (`app/`):** `ai-copilot.html`, `src/ai-copilot/copilot.css`, `src/ai-copilot/copilot-state.ts`, `src/ai-copilot/markdown-lite.ts` (nuevo módulo puro anti-XSS con tests `vitest`), `src/ai-copilot/copilot-main.ts`, `src/main.ts`.
- **Verificación Real:** Pruebas unitarias (`cargo test` + `vitest`) + **prueba de integración real multi-turno contra `agy.EXE`** verificando consulta directa de OS, propuesta de cambio con dry-run y certificación Fase 4.
