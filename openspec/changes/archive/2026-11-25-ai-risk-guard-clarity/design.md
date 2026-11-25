## Context

`CommandRiskGuard` y `ReadOnlyEnforcer` comparten `extract_effective_verb` y `args_after_verb` en `app/src-tauri/src/ai/risk_guard.rs`. Actualmente:
1. `extract_effective_verb` salta `"command"` incondicionalmente como wrapper (`sudo`, `env`, `nohup`, `time`, `exec`, `command`) y salta cualquier flag que empiece con `'-'`. En `command -v docker 2>&1`, devuelve `"docker"` como verbo efectivo y `"2>&1"` como único argumento posterior, disparando `is_service_or_package_mutating_call("docker", ...)` -> `true` (`ConfigChange`).
2. `ReadOnlyEnforcer::validate_recon_command` redacta rechazos como `"Fase de Reconocimiento prohíbe..."`, y `CommandRiskGuard::evaluate_proposal` reutiliza exactamente esa cadena como `audit_warning` en tarjetas HITL aunque el operador esté pidiendo una acción legítima o revisando una propuesta.
3. En frontend (`copilot-state.ts`), los badges muestran `"Solo lectura"` / `"Configuración"`, poco explícitos respecto al impacto real.

## Goals / Non-Goals

**Goals:**
- Garantizar cero falsos positivos de mutación sobre inspecciones compuestas POSIX con `command -v`/`-V`, `which`, `type`, `docker ps`, `docker compose ps`, `systemctl is-active`, `--version` y redirecciones `2>&1`/`2>/dev/null`.
- Mantener la arquitectura híbrida de seguridad (LLM propone `risk_level` + Rust valida deterministamente sin poder auto-ejecutar mutaciones en Modo Flexible).
- Emitir `audit_warning` claro y accionable en español latino sin jerga interna de fases.
- Clarificar las etiquetas de severidad en la tarjeta de autorización.

**Non-Goals:**
- No eliminar el auditor determinista en Rust ni delegar confianza ciega al LLM.
- No modificar el esquema JSON de salida del driver.

## Decisions

### 1. Manejo contextual del builtin `command` en `extract_effective_verb`
- **Decisión:** Cuando el primer token ejecutable sea `command` y entre sus flags inmediatos (antes del siguiente operando posicional) figure `-v` o `-V` (incluyendo grupos como `-pv` o `-pV`), devolver `"command"` como verbo efectivo (`ReadOnly`), ya que el builtin POSIX únicamente imprime la ruta o tipo del comando sin ejecutarlo. Si no incluye `-v`/`-V` (p. ej. `command rm -rf /` o `command -p systemctl restart nginx`), continuar desempaquetando el verbo real subyacente.
- **Alternativa descartada:** Eliminar `"command"` por completo de la lista de wrappers. Permitiría evadir la auditoría escribiendo `command rm ...`.

### 2. Filtrado de redirecciones inocuas y analizador compartido `inspect_readonly_safety`
- **Decisión:** Filtrar tokens de redirección estándar no entrecomillados (`2>&1`, `1>&2`, `>/dev/null`, `2>/dev/null`, `1>/dev/null`, `&>/dev/null`) al extraer subcomandos en `args_after_verb` / `is_service_or_package_mutating_call`, e incorporar una única función `inspect_readonly_safety` que devuelve descripciones claras de seguridad para `CommandRiskGuard::evaluate_proposal`, mientras `ReadOnlyEnforcer::validate_recon_command` conserva el prefijo `"Fase de Reconocimiento: "` para compatibilidad con guardias estrictos de Fase 1.

### 3. Etiquetas autoexplicativas en `formatRiskLabel`
- **Decisión:** Refinar las etiquetas en `copilot-state.ts` a `"Solo lectura · No modifica"`, `"Modifica archivos / config"`, `"Impacto en servicio"` y `"Crítico · Destructivo"`.

## Risks / Trade-offs

- **[Evasión mediante flags combinados con `command`]** → Evaluar cualquier token flag corto tras `command` antes del operando posicional comprobando `.contains('v') || .contains('V')` únicamente cuando empieza con `-` y no `--`, asegurando que `command -p rm` siga auditándose como `rm`.
- **[Incompatibilidad con pruebas previas que asertaban texto literal de advertencia]** → `commands.rs:validate_and_prepare_readonly_command` conserva el prefijo `"Fase de Reconocimiento: "` sobre el mensaje claro, manteniendo contratos previos verdes.
