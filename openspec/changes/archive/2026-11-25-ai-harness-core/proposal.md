## Why

Actualmente los flujos de optimización, resolución de incidentes y despliegues en servidores remotos dependen del copiado y pegado manual entre la terminal y herramientas externas de IA, lo que genera fricción y riesgo operativo. Para integrar un asistente agéntico seguro y de nivel productivo dentro de NekoSSH, se requiere un arnés determinista en el backend Rust que gobierne el ciclo de vida del agente, fuerce contratos estrictos de solo lectura, audite riesgos con un analizador léxico de 4 etapas, soporte modos de ejecución configurables (Estricto y Flexible sin turbo) y mantenga resiliencia de sesión ante desconexiones de red.

## What Changes

- Creación del submódulo desacoplado de inteligencia artificial en Rust (`app/src-tauri/src/ai/`).
- Implementación de la máquina de estados finita del arnés (`AgentHarnessFsm`) con soporte para las 3 fases operativas (Reconocimiento, Plan y Debate, Ejecución guiada) y estados de resiliencia ante contingencias (`HaltedOnError`, `PausedOnDisconnect`).
- Soporte para dos modos operativos configurables en SQLite:
  - **Modo Estricto (Default):** Autorización manual humana obligatoria para absolutamente todo, incluyendo cada lectura de diagnóstico.
  - **Modo Flexible (Semi-Supervisado):** Auto-autorización de lecturas puras verificadas (🟢) para acelerar diagnósticos, con freno manual obligatorio ante cualquier cambio de configuración, servicio o riesgo (prohibido modo turbo desatendido).
- Implementación de `ReadOnlyEnforcer` para asegurar que en la fase de reconocimiento no se presenten comandos con mutaciones o redirecciones.
- Implementación del analizador léxico de riesgo `CommandRiskGuard` estructurado en 4 etapas (Lexer/Tokenizador, Detección de tuberías a shell, Análisis de verbos/flags y Auditor de operadores de redirección) que clasifica o eleva deterministamente en 4 niveles de severidad (`read_only`, `config_change`, `service_impact`, `destructive`).
- Implementación del sanitizador `SecretRedactor` para enmascarar automáticamente llaves privadas PEM, tokens y contraseñas antes del procesamiento agéntico.
- Implementación del ensamblador de prompt en 2 capas (`PromptAssembler`), fusionando una constitución fija e inmutable en Rust con directivas del operador persistidas en la tabla SQLite `app_preferences`.
- Arquitectura multi-sesión mediante `HarnessRegistry(Arc<Mutex<HashMap<String, AgentHarnessFsm>>>)`, aislando el estado de cada sesión/pestaña SSH.
- Exposición de comandos IPC Tauri seguros para consultar el estado del arnés, auditar comandos y gestionar preferencias.

## Capabilities

### New Capabilities
- `ai-harness-core`: Especificación del núcleo del arnés determinista, máquina de estados resiliente ante desconexión, contratos de respuesta estructurada, guardrails de seguridad (modos estricto/flexible, auditor léxico de riesgo, redacción de secretos) y configuración en dos capas.

### Modified Capabilities
<!-- Sin modificaciones a requerimientos de specs previas -->

## Impact

- **Backend Rust (`app/src-tauri`)**: Nuevo submódulo `src/ai/` con pruebas unitarias TDD exhaustivas en memoria. Integración mínima en `lib.rs` (3 líneas) para registrar comandos IPC de arnés sin alterar el motor SSH existente.
- **Base de Datos Local (`SQLite`)**: Reutilización de la tabla `app_preferences` para almacenar claves de directivas del operador y modo de ejecución (`ai_execution_mode`).
- **Dependencias**: Reutilización estricta de crates existentes (`serde`, `serde_json`, `rusqlite`, `uuid`, `tauri`). Cero crates adicionales pesados en este hito.
