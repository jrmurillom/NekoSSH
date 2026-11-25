## Why

Al proponer comandos legítimos de inspección como `command -v docker 2>&1; systemctl is-active docker containerd 2>&1; docker ps 2>&1`, el auditor léxico `CommandRiskGuard` en `risk_guard.rs` interpreta `command -v docker` como si invocara el binario `docker` directamente (saltando `-v` como wrapper junto a `sudo`) y detecta `2>&1` sin subcomando de lectura, inflando falsamente la severidad a `ConfigChange` ("Configuración") y mostrando la alerta confusa `"Fase de Reconocimiento prohíbe operaciones de modificación/servicio con 'docker'"`.

Esto impide al operador distinguir con claridad si un comando propuesto es realmente de solo lectura o implica mutaciones, y rompe la confianza en las etiquetas y advertencias de seguridad de la tarjeta de autorización.

## What Changes

- **Corrección léxica en `CommandRiskGuard` y `ReadOnlyEnforcer` (`app/src-tauri/src/ai/risk_guard.rs`)**:
  - Tratar `command -v` y `command -V` como búsqueda de binario en `$PATH` de solo lectura (sin extraer el argumento buscado como verbo ejecutado), manteniendo el unwrap únicamente cuando `command` ejecuta un binario sin `-v`/`-V`.
  - Filtrar tokens de redirección inocua (`2>&1`, `1>&2`, `2>/dev/null`, `>/dev/null`, `1>/dev/null`, `&>/dev/null`) al analizar argumentos posicionales y reconocer flags informativos (`--version`, `--help`) y subcomandos compuestos seguros (`docker compose ps|logs|ls`, `docker container ls|inspect`, `docker network ls|inspect`, `docker volume ls|inspect`).
  - Separar la redacción de advertencias para propuestas conversacionales (`audit_warning` en lenguaje claro de SysAdmin sin jerga interna de `"Fase de Reconocimiento prohíbe..."`), conservando compatibilidad de mensaje en `ReadOnlyEnforcer::validate_recon_command`.
- **Claridad visual en etiquetas de riesgo (`app/src/ai-copilot/copilot-state.ts`)**:
  - Actualizar `formatRiskLabel` con etiquetas autoexplicativas (`"Solo lectura · No modifica"`, `"Modifica archivos / config"`, `"Impacto en servicio"`, `"Crítico · Destructivo"`).

## Capabilities

### New Capabilities
- *(Ninguna)*

### Modified Capabilities
- `ai-harness-core`: Precisión en tokenización de `command -v`/`-V`, redirecciones inocuas entre argumentos y mensajes humanos claros en `ProposalSafetyReport::audit_warning`.

## Impact

- **Backend**: `app/src-tauri/src/ai/risk_guard.rs`.
- **Frontend**: `app/src/ai-copilot/copilot-state.ts`, `app/src/ai-copilot/copilot-state.test.ts`.
