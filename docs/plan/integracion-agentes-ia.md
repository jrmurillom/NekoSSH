---
description: Plan maestro de arquitectura, alcance e hitos para la integración del Agente Copiloto SSH (NekoAI)
alwaysApply: false
---

# Plan Maestro: Integración de Agente Copiloto IA (`NekoAI`)

Este documento constituye la **guía maestra de arquitectura, seguridad y alcance** para integrar un asistente agéntico *Human-in-the-Loop (HITL)* de grado comercial dentro de **NekoSSH**.

> **Gobernanza con OpenSpec:** Este documento define la visión técnica invariable y los **Hitos controlables** del proyecto. **No** sustituye a los artefactos `proposal.md`, `design.md`, `specs/` ni define las tareas granulares (`tasks.md`), los cuales serán generados e implementados hito por hito mediante la CLI de OpenSpec (`/opsx-propose` → `/opsx-apply` → `/opsx-archive`) conforme a `docs/base-standards.md`.

---

## 1. Objetivos del Proyecto

1. **Eliminar el ciclo manual de copiar y pegar externo:** Integrar un copiloto especializado en administración de sistemas, redes y despliegues remotos capaz de leer el contexto autorizado de la sesión SSH activa y proponer rutas de acción directamente conectadas a la consola.
2. **Control Humano Estricto (*Human-in-the-Loop* Inviolable):** Ningún comando (ni siquiera de diagnóstico o lectura) puede ejecutarse en el servidor remoto sin autorización explícita del usuario por paso, permitiendo autorizar, editar el comando propuesto, rechazarlo o cuestionar/debatir la estrategia antes de tocar el servidor.
3. **Comportamiento Determinista mediante Arnés en Rust:** Evitar depender únicamente del *prompt* para la seguridad operativa. Un **Arnés Determinista (`AgentHarness`)** en el backend Rust gobierna las fases de trabajo, valida contratos JSON estrictos, audita estáticamente el riesgo de cada comando y frena en seco ante cualquier código de salida fallido (`exit_code != 0`).
4. **Arquitectura Multi-Proveedor Desacoplada:** Iniciar con **`agy` (Google Antigravity CLI)** como primer motor de razonamiento a través de un puerto abstracto (`trait AiProviderDriver`), permitiendo incorporar futuros proveedores (Anthropic Claude, OpenAI, Google Gemini API, DeepSeek u Ollama local) sin modificar el Arnés, el canal SSH ni la interfaz gráfica.
5. **Espacio de Trabajo Independiente Multi-Monitor:** Operar sobre una ventana nativa independiente del sistema operativo (`WebviewWindow` en Tauri v2) redimensionable y con soporte *Always on Top*, manteniendo el viewport principal de terminales (`xterm.js`) 100% despejado y fiel a `docs/design/ui-layout-contract.md`.

---

## 2. Tablero de Hitos y Estatus

Cada hito está dimensionado para abordarse como un *Change* autocontenido y verificable dentro del flujo de OpenSpec.

| Hito | Nombre del Entregable | Superficies OpenSpec | Estatus | Change OpenSpec Asociado |
| :--- | :--- | :--- | :--- | :--- |
| **Hito 1** | Núcleo del Arnés Determinista, Guardrails y Preferencias (Backend Core) | `desktop-commands` | `✅ COMPLETADO` | `ai-harness-core` |
| **Hito 2** | Puerto `AiProviderDriver` y Adaptador `AgyCliDriver` Aislado | `desktop-commands` | `✅ COMPLETADO` | `ai-provider-driver-agy` |
| **Hito 3** | Ventana Nativa `neko-ai`, Ajustes y Flujo de Reconocimiento + Plan (Fases 1–2) | `desktop-ui`, `desktop-commands` | `✅ COMPLETADO` | `ai-copilot-window` |
| **Hito 4** | Ejecución Guiada en PTY con Centinela, Edición Inline, Freno de Error y Rollback (Fase 3) | `desktop-ui`, `desktop-commands` | `✅ COMPLETADO` | `ai-guided-execution` |

*(Leyenda de estatus: `⏳ PENDIENTE` · `🔄 EN PROGRESO` · `✅ COMPLETADO` · `⏸️ PAUSADO`)*

---

## 3. Arquitectura Técnica de Referencia

### 3.1 Visión General de Capas

```text
┌─────────────────────── FRONTEND (TypeScript + Vite + CSS Tokens) ───────────────────────┐
│  Ventana "main" (index.html)                 Ventana Nativa "neko-ai" (ai-copilot.html) │
│  • Sesiones xterm.js + SFTP                  • Stepper 3 Fases + Checklist del Plan     │
│  • Notifica focusedTerminalId y tema         • Tarjetas HITL (Autorizar/Editar/Debatir) │
│  • Botón disparador ✦ IA + atajo             • Panel ⚙️ Ajustes (Directivas Operador)   │
└──────────────────────┬───────────────────────────────────────────┬──────────────────────┘
                       │ Eventos / Commands IPC Tauri v2           │ Capability aislada
                       ▼                                           ▼
┌───────────────────────────────── BACKEND RUST (src-tauri) ──────────────────────────────┐
│                                  Módulo `src/ai/`                                       │
│                                                                                         │
│  ┌─────────────────────┐   ┌─────────────────────────┐   ┌───────────────────────────┐  │
│  │   PtyContextTap     │──▶│   AgentHarness (FSM)    │──▶│  Policy & Safety Engine   │  │
│  │ • Opt-in Escucha    │   │ • Fase 1: Recon (RO)    │   │ • ReadOnlyEnforcer (F1)   │  │
│  │ • Strip ANSI/OSC    │   │ • Fase 2: Debate & Plan │   │ • CommandRiskGuard        │  │
│  │ • Head/Tail Budget  │   │ • Fase 3: Ejecución     │   │ • SecretRedactor (PEM/Key)│  │
│  │ • Sentinel Nonce    │   │ • PromptAssembler (2C)  │   └───────────────────────────┘  │
│  └──────────▲──────────┘   └────────────┬────────────┘                                  │
│             │                           │                                               │
│             ▼                           ▼                                               │
│  `SshConnections` (`LiveSsh`)    `trait AiProviderDriver`                               │
│  • `ssh-stdout` / `write_ssh`    └── `AgyCliDriver` (headless + JSON Schema + sandbox)  │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Flujo Operativo Adaptativo de 4 Fases (Evolución `ai-conversational-copilot-v2`)

El copiloto opera bajo ejecución progresiva conversacional de **un comando por turno** con auditoría determinista en Rust (nunca bloqueante de la conversación):

1. **Estado `Idle` (Escucha Desactivada — Default):**
   - `listening = false`. Cero captura de consola. Al activar `Escucha ON`, `"main"` siembra las últimas líneas visibles de `xterm.js` (sanitizadas por `SecretRedactor`).
2. **Fase 1 — Diagnóstico y Telemetría No Intrusiva:**
   - Cero suposiciones de distribución, kernel o servicios. Si la consulta es informativa directa, responde con Markdown estructurado (`direct_answer`) conservando la memoria de sesión.
   - `CommandRiskGuard::evaluate_proposal` reconoce redirecciones inocuas POSIX (`2>/dev/null`, `2>&1`) como `read_only`; en `Modo Flexible` corren automáticamente hasta `MAX_AUTO_CHAIN_STEPS`, mientras en `Modo Estricto` o con `audit_warning` piden clic humano explicando para qué sirve el comando.
3. **Fase 2 — Diagnóstico de Causa Raíz, Respaldo y Rollback:**
   - Explica causa raíz, advierte riesgos de red/SSH/servicios, define respaldo fechado (`cp archivo archivo.bak.$(date +%F_%T)`) y `rollback_cmd`.
4. **Fase 3 — Ejecución Progresiva de Un Comando por Turno + *Dry-Run*:**
   - Entrega un único comando por turno esperando su `exit_code` y salida real antes de formular el siguiente, exigiendo validación sintáctica pre-reinicio (`nginx -t`, `sshd -t`, `apache2ctl configtest`, `visudo -c`). Todo comando con riesgo/alerta o en `Modo Estricto` requiere clic explícito (**Autorizar**, **Editar**, **Copiar**, **Omitir**, **`Ctrl+C`** en vivo) y habilita **Preparar Rollback** con 1 clic si `exit_code != 0`.
5. **Fase 4 — Verificación y *Health Check* Post-Cambio:**
   - Audita `systemctl is-active`, `ss -tulpn` y `journalctl -n 20 --no-pager` antes de certificar éxito final.

### 3.3 Arquitectura de Prompt en 2 Capas (`PromptAssembler`)

Para impedir que una edición accidental en configuración corrompa los contratos del sistema:

- **Capa 1 — Constitución del Arnés (Inmutable en Rust):**
  Define las reglas mecánicas intocables: respeto a la fase activa, obligatoriedad de respuesta bajo JSON Schema estricto, prohibición de lanzar programas interactivos de pantalla completa en el PTY (`vim`, `nano`, `htop`, `less`, exigiendo flags no interactivos como `--no-pager` o `DEBIAN_FRONTEND=noninteractive`) y obligación de documentar respaldo/reversión en cambios de configuración o servicios.
- **Capa 2 — Directivas del Operador (Configurable en UI y persistida en SQLite `app_preferences`):**
  Editable desde el panel `⚙️ Ajustes` dentro de la ventana `neko-ai` (con preset profesional de fábrica y botón *"Restaurar predeterminado"*):
  - `ai_operator_persona_prompt`: Rol, tono y prioridades técnicas del operador.
  - `ai_operator_infra_rules`: Restricciones fijas de la infraestructura del usuario (ej. políticas de firewall, convenciones de rutas o respaldos).
  - `ai_active_driver`: Identificador del proveedor activo (inicialmente `"agy"`).
  - `ai_agy_binary_path`: Ruta personalizada opcional al binario `agy` cuando no resida en `PATH`.
  - `ai_context_max_lines`: Límite de líneas limpias de consola enviadas bajo escucha activa (ej. `200` por defecto).

### 3.4 Contratos Tipados y Validadores de Seguridad en Rust

- **Contrato JSON Estricto (`schema.rs`):**
  Comunicación estructurada validada con `serde` (cero extracción frágil de bloques Markdown con expresiones regulares). Contempla variantes discriminadas para propuesta de comando de lectura, propuesta de plan estructurado, propuesta de paso de ejecución (con `command`, `risk`, `expected_outcome`, `backup_strategy`, `rollback_cmd`), respuesta de debate y cierre de objetivo.
- **Auditor Estático de Riesgo (`CommandRiskGuard` en `risk_guard.rs`):**
  Clasificador determinista en Rust que evalúa tokens, tuberías y operadores de shell (`;`, `&&`, `||`, `|`, `>`, `>>`, `$()`, `` ` ``). Si la IA etiqueta erróneamente un comando mutante o destructivo como `read_only`, Rust **eleva forzosamente** su severidad a `config_change`, `service_impact` o `destructive` (`🔴`) antes de renderizar la tarjeta en la UI.
- **Sanitizador Anti-Fuga de Secretos (`SecretRedactor` en `redactor.rs`):**
  Filtro determinista previo al envío de cualquier captura de terminal al driver externo. Enmascara bloques PEM (`-----BEGIN ... PRIVATE KEY-----`), encabezados `Bearer`, credenciales embebidas en URIs (`scheme://user:pass@host`) y asignaciones sensibles (`PASSWORD=`, `PASSWD=`, `SECRET=`, `API_KEY=`, `TOKEN=`).

### 3.5 Puerto Multi-Proveedor y Adaptador `AgyCliDriver`

- **Abstracción (`trait AiProviderDriver`):**
  Interfaz interna agnóstica de transporte que recibe `AgentTurnRequest` (instrucciones ensambladas, estado de fase, snapshot sanitizado, `provider_conversation_id` opcional y esquema esperado) junto a un mecanismo de cancelación cooperativa, devolviendo `Result<AgentTurnOutput, AiDriverError>` más un método de diagnóstico `probe_health()`.
- **Implementación `AgyCliDriver` (`src/ai/drivers/agy.rs`):**
  - Invoca `agy` en modo *headless* no interactivo (`-p`, `--output-format json` / `stream-json`, `--json-schema`, `--conversation <id>`).
  - **Aislamiento estricto:** Ejecuta el subproceso con `current_dir` apuntando a un directorio vacío dedicado `{appData}/ai-sandbox/` y **jamás** incluye `--dangerously-skip-permissions`, impidiendo que el CLI ejecute comandos o lea archivos locales en la máquina anfitriona.
  - **Compatibilidad Windows de producción:** Resolución explícita de `agy.exe` / `agy.cmd` y aplicación obligatoria del flag `CREATE_NO_WINDOW` (`0x08000000` vía `std::os::windows::process::CommandExt`) para evitar parpadeos de ventanas de consola CMD.
  - **Control de procesos:** Timeout configurable por turno y terminación inmediata del proceso hijo si el usuario cancela el turno, cierra la pestaña SSH o cierra la aplicación.

### 3.6 Captura de Consola y Ejecución con Centinela en PTY (`pty_tap.rs`)

- **Limpieza y Presupuesto de Contexto:**
  Cuando `Escucha = ON` para un `terminal_id`, Rust procesa el flujo eliminando secuencias de escape ANSI/OSC y retornos de carro transitorios (`\r` de barras de progreso), conservando un *ring buffer* acotado. Ante salidas masivas se aplica truncamiento determinista **Head + Tail** (primeras $N$ líneas + indicador de líneas omitidas + últimas $M$ líneas).
- **Centinela POSIX con Nonce Anti-Inyección:**
  Al autorizar un comando en el PTY real (`write_ssh_input`), el ejecutor anexa un marcador compatible con `sh`/`dash`/`bash`/`zsh`:
  ```sh
  <comando_autorizado> ; printf '\n__NEKO_END_%s:%d__\n' '<nonce_unico>' "$?"
  ```
  - El `<nonce_unico>` efímero por turno impide que contenido antiguo del scrollback o texto malicioso dentro de archivos de log remotos falsifique el fin de ejecución.
  - El interceptor en backend detecta el marcador exacto, extrae el `exit_code` real, filtra la línea interna `__NEKO_END_...__` para no ensuciar la terminal visual del usuario y despierta al Arnés con la salida exacta del paso.
  - Mientras el comando corre en el PTY real, el usuario conserva control total para escribir entradas interactivas inevitables (como contraseñas de `sudo`) o enviar interrupción inmediata (`\x03` / `Ctrl+C`) desde cualquiera de las dos ventanas.

### 3.7 Integración Multi-Ventana en Tauri v2 y Vite

- **Empaquetado Multi-Página (`app/vite.config.ts`):**
  Configuración explícita de `build.rollupOptions.input` con dos entradas HTML (`index.html` para `"main"` y `ai-copilot.html` para `"neko-ai"`), garantizando que el binario de producción (`tauri build`) resuelva ambas vistas sin errores 404.
- **Permisos Aislados (`app/src-tauri/capabilities/ai-copilot.json`):**
  *Capability* dedicada para la etiqueta de ventana `"neko-ai"` con privilegios mínimos (comandos IPC del módulo `ai`, eventos de sincronización de estado/tema y `clipboard-manager`).
- **Ciclo de Vida y Sincronización:**
  - Si la ventana `"main"` se destruye (`WindowEvent::Destroyed`), el backend cancela cualquier turno activo de IA, cierra la ventana `"neko-ai"` y ejecuta la clausura ordenada de todas las sesiones SSH y transferencias activas.
  - Si se cierra únicamente `"neko-ai"`, las sesiones SSH continúan intactas en `"main"` y la escucha activa pasa a modo seguro según política del arnés.
  - Sincronización en tiempo real de los 8 temas conceptuales (`data-theme` de `docs/design/DESIGN.md`, acentos planos sin *neon glow*, scrollbars Cyber-Sakura e iconos Lucide outline) y vinculación clara al `terminal_id` activo con opción de **Fijar Shell (*Pin*)** para entornos multi-pestaña y cuadrícula multi-shell.

---

## 4. Stack, Librerías y Restricciones de Dependencias

Para preservar ligereza del binario, tiempos de compilación y mantenibilidad:

| Capa | Librerías / Crates a Reutilizar (Ya en el proyecto) | Adiciones Mínimas Permitidas / Estrategia |
| :--- | :--- | :--- |
| **Backend Rust (`app/src-tauri`)** | `tauri` v2 (multi-window, IPC, events), `serde` + `serde_json` (contratos tipados), `rusqlite` (`app_preferences`), `uuid` v4 (IDs de pasos y nonces de centinela), `ssh2` (`SshConnections` existente). | Preferir parsers deterministas sin dependencias pesadas para `strip_ansi`, `CommandRiskGuard` y `SecretRedactor` (o `regex` únicamente si la complejidad de patrones lo justifica en diseño). Manejo de subproceso `agy` con `std::process` / `tauri::async_runtime` + flag atómico de cancelación. *(Crates HTTP como `reqwest` NO se añadirán hasta un hito futuro de drivers API directos).* |
| **Frontend (`app/`)** | TypeScript estricto, Vite 6 (`rollupOptions.input`), `lucide` (iconos outline `currentColor`), `@tauri-apps/api`, `@tauri-apps/plugin-clipboard-manager`, CSS Vanilla con Tokens de `styles.css`. | **Cero frameworks UI adicionales** y **cero librerías externas de Markdown inseguras** (renderizador propio ligero y escapado contra XSS para respuestas y bloques de código del agente). Todo `<input>` / `<textarea>` nuevo cumplirá `autocomplete="off"`. |
| **Testing** | `cargo test` (Rust unit/integration en memoria) y `vitest` (módulos TS puros). | Cobertura TDD obligatoria sobre máquina de estados, validadores de seguridad, sanitizador de secretos, parser de centinela y helpers de UI. |

---

## 5. Desglose de Hitos y Alcances

### Hito 1: Núcleo del Arnés Determinista, Guardrails y Preferencias (Backend Core)
- **Estatus:** `✅ COMPLETADO`
- **Objetivo:** Construir los cimientos puros en Rust (`app/src-tauri/src/ai/`) que garantizan determinismo, seguridad y configuración en capas antes de conectar procesos externos o interfaz gráfica.
- **Alcance Incluido:**
  - Modelo de dominio y máquina de estados finita (`Idle`, `Reconnaissance`, `PlanAndDebate`, `GuidedExecution`, `HaltedOnError`, `Completed`).
  - Esquemas `serde` y JSON Schema de salida del agente con validación estricta de campos (incluyendo `backup_strategy` y `rollback_cmd`).
  - `ReadOnlyEnforcer` (candado duro de Fase 1) y `CommandRiskGuard` (elevación determinista de severidad).
  - `SecretRedactor` para enmascaramiento preventivo de credenciales y llaves privadas en capturas de consola.
  - `PromptAssembler` de 2 capas (Constitución fija + Directivas del operador) respaldado en SQLite (`app_preferences`) y comandos Tauri de lectura/escritura/restauración de preferencias del agente.
- **Fuera de Alcance del Hito:** Invocación del binario real `agy`, inyección en PTY y ventana frontend `neko-ai`.
- **Verificación de Cierre:** Suite exhaustiva en `cargo test` validando transiciones ilegales, bloqueo de comandos mutantes en Fase 1, detección de comandos destructivos, redacción de secretos y persistencia idempotente en SQLite en memoria.

---

### Hito 2: Puerto `AiProviderDriver` y Adaptador `AgyCliDriver` Aislado
- **Estatus:** `✅ COMPLETADO`
- **Objetivo:** Conectar la capacidad de razonamiento agéntico mediante un puerto desacoplado e implementar el primer driver productivo sobre `agy` CLI blindado para Windows y entornos POSIX.
- **Alcance Incluido:**
  - Definición de `trait AiProviderDriver` (`probe_health`, `list_models` para selector dinámico de modelo, y `execute_turn` con streaming y `TurnCancellationToken`) y sobre plano `AgentTurnWireEnvelope`.
  - Implementación de `AgyCliDriver` validado contra `agy.EXE v1.5.16`: descubrimiento de binario (`driver_options["agy_binary_path"]` / `PATH`), catálogo real `agy models`, invocación *headless* con `--output-format stream-json`, `--sandbox`, `--disable-slash-commands`, `--json-schema <archivo_RAII>`, `--model`, `--conversation`, prompt acotado vía `--print`, directorio aislado `{appData}/ai-sandbox/`, `CREATE_NO_WINDOW` en Windows y cancelación cooperativa.
  - Implementación de `FakeAiDriver` determinista y `HarnessOrchestrator` sin bloqueo de `Mutex` con remediaciones completas de auditoría en FSM, `CommandRiskGuard` y `SecretRedactor` (UTF-8 seguro).
  - Comandos IPC Tauri para consultar salud del driver, enviar turno de conversación/diagnóstico y cancelar turno activo.
- **Fuera de Alcance del Hito:** Drivers HTTP de terceros (OpenAI, Claude API, Gemini API, Ollama) e interfaz visual de la ventana secundaria.
- **Verificación de Cierre:** Pruebas unitarias y de integración con `FakeAiDriver` y pruebas de contrato de construcción de argumentos, manejo de timeouts, cancelación y parseo de respuestas/errores de `AgyCliDriver`.

---

### Hito 3: Ventana Nativa `neko-ai`, Ajustes y Flujo de Reconocimiento + Plan (Fases 1–2)
- **Estatus:** `✅ COMPLETADO` (`ai-copilot-window`)
- **Objetivo:** Materializar la ventana independiente multi-monitor en Tauri v2 con estética Cyber-Sakura y habilitar de extremo a extremo la lectura opt-in de consola, el diagnóstico de solo lectura y la negociación/aprobación del Plan de Ejecución.
- **Alcance Incluido:**
  - Configuración multi-entry en `app/vite.config.ts` (`ai-copilot.html`), *capability* aislada `ai-copilot.json` y gestión de ciclo de vida multi-ventana en Rust (`Always on Top`, cierre en cascada al destruir `"main"`).
  - Disparador `#btn-footer-ai` (Lucide outline `Sparkles`) + atajo `Ctrl+Shift+A` en la ventana principal y sincronización reactiva de tema activo (`nekossh-theme`) y `focusedTerminalId` (con selector e indicador de **Pin Shell**).
  - Interceptor opt-in `PtyContextTap` (solo cuando `Escucha = ON`) con limpieza ANSI/OSC y recorte *Head + Tail*.
  - UI completa para Estado Privacidad (`Escucha OFF`), Fase 1 (*Reconocimiento Solo Lectura*), Fase 2 (*Debate interactivo de ruta y Aprobación del Plan*) y modal/vista interna `⚙️ Ajustes del Agente` (Directivas de Capa 2 + verificación de estado de `agy`).
- **Fuera de Alcance del Hito:** Inyección automatizada paso a paso de Fase 3 con centinela y reversión asistida (cubiertos en Hito 4; en Hito 3 las lecturas aprobadas de Fase 1 ya validan el puente controlado o quedan listas para el corredor unificado según diseño del change).
- **Verificación de Cierre:** Compilación limpia `npm run build` (verificando ambos bundles HTML) + pruebas `vitest` + verificación real de comandos IPC y ventana nativa en runtime de escritorio.

---

### Hito 4: Ejecución Guiada en PTY con Centinela, Edición Inline, Freno de Error y Rollback (Fase 3)
- **Estatus:** `✅ COMPLETADO` (`ai-guided-execution`)
- **Objetivo:** Completar el ciclo productivo permitiendo ejecutar el plan aprobado paso a paso sobre el PTY real con detección exacta de término, código de salida y mecanismos de contingencia.
- **Alcance Incluido:**
  - Corredor `PtySentinelRunner`: inyección en el `terminal_id` vinculado tras autorización humana obligatoria en todos los pasos `[1..N]`, wrapper POSIX inmune a eco TTY (`%s_%s:%d__`) y comentarios, ocultamiento del marcador interno en el stream hacia `xterm.js` y commit atómico sobre `HarnessRegistry` en el hilo lector SSH.
  - Controles de Fase 3 en `neko-ai`: checklist de progreso en vivo, tarjetas con semáforo de severidad + detalle de `backup_strategy`/`rollback_cmd`, **edición inline** re-auditada en vivo del comando previo a ejecutar y botón de interrupción remota (`Ctrl+C` -> `exit_code = 130`).
  - Transición automática a **Freno de Emergencia (`HaltedOnError`)** cuando `exit_code != 0` (o desconexión SSH con paso en vuelo), permitiendo replantear el paso fallido junto al agente o autorizar la ejecución quirúrgica del `rollback_cmd` (tanto del paso detenido como de pasos previos completados en orden LIFO con bandera `is_rolled_back`).
  - Actualización final de documentación permanente SSOT (`docs/project_scope.md`, `docs/design/DESIGN.md`, `docs/design/ui-layout-contract.md`).
- **Fuera de Alcance del Hito:** Ejecución desatendida (*auto-run* sin confirmación humana) — prohibida por principio de seguridad del producto.
- **Verificación de Cierre:** Suite completa Rust + Frontend en verde, verificación E2E de escritorio cubriendo escenario completo (Reconocimiento → Debate → Plan → Ejecución exitosa → Simulación de fallo con freno automático y Rollback) y auditoría de cierre de sesiones.

---

## 6. Exclusiones Globales (Fuera del Alcance de esta Iniciativa)

1. **Modo Autónomo sin Confirmación ("YOLO Mode"):** Queda estrictamente descartado ejecutar secuencias de comandos remotos sin confirmación humana explícita paso a paso.
2. **Acoplamiento de Ventana Flotante Híbrida in-DOM (Opción B descartada):** Se adopta exclusivamente la **Ventana Nativa Independiente (`WebviewWindow`)** con soporte *Always on Top* para evitar duplicidad de UI y mantener el área de terminales libre de obstrucciones.
3. **Proveedores HTTP Adicionales en el MVP:** Los conectores directos para APIs REST (Claude, OpenAI, Gemini API, Ollama) quedan diferidos como extensiones futuras sobre el `trait AiProviderDriver` una vez consolidado el Hito 4 con `agy`.
4. **Escucha Pasiva Permanente en Segundo Plano:** El agente jamás leerá ni procesará streams de `ssh-stdout` mientras el interruptor de escucha permanezca en `OFF`.
