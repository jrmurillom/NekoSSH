## MODIFIED Requirements

### Requirement: Validador Estricto de Solo Lectura (ReadOnlyEnforcer)
Durante la fase de `Reconnaissance`, el backend SHALL validar mediante análisis léxico basado en tokens conscientes de comillas que todo comando propuesto sea estrictamente de inspección y no cause mutaciones en el sistema remoto, reconociendo búsquedas POSIX inocuas (`command -v`, `command -V`, `which`, `type`), redirecciones estándar inocuas (`2>&1`, `2>/dev/null`), flags `--version` y subcomandos compuestos de solo lectura en herramientas de contenedores y servicios.

#### Scenario: Aprobación de comandos puros de diagnóstico
- **WHEN** el agente propone comandos como `uname -a`, `cat /etc/os-release`, `free -h`, `ss -tulpn`, `df -h`, `grep -i "error" /var/log/syslog` o diagnósticos compuestos como `command -v docker 2>&1; systemctl is-active docker containerd 2>&1; docker ps 2>&1; ps -ef | grep -iE 'dockerd|containerd' | grep -v grep 2>&1; true`
- **THEN** el validador SHALL clasificar el comando como permitido (`read_only`) sin falsos positivos ni advertencias espurias de modificación

#### Scenario: Bloqueo de redirecciones de escritura y mutaciones
- **WHEN** un comando en reconocimiento contiene redirecciones fuera de comillas hacia archivos reales (`>`, `>>`), `tee` directo o en tubería, tuberías hacia intérpretes con o sin espacios/prefijos (`|bash`, `|  sh`, `| sudo bash`), `find` con `-delete`/`-exec`, `crontab`, gestores de paquetes/contenedores con acciones mutantes (incluyendo `command rm` o `command -p systemctl restart`), creación o borrado de archivos (`rm`, `touch`, `mkdir`, `mv`, `cp`, `chmod`, `chown`) o control de servicios (`systemctl`, `service`, `kill`)
- **THEN** el validador SHALL rechazar el comando marcándolo como no conforme para la fase de reconocimiento, impidiendo su presentación como lectura segura

#### Scenario: Reconocimiento de supresión estándar POSIX 2>/dev/null y 2>&1 como lectura pura
- **WHEN** el agente propone comandos diagnósticos como `cat /etc/os-release 2>/dev/null || uname -a` o `nginx -t 2>&1`
- **THEN** `ReadOnlyEnforcer` y `CommandRiskGuard` SHALL clasificarlos como `read_only` (`🟢`) sin emitir falsos positivos de redirección de escritura

#### Scenario: Degradación segura a autorización humana sin bloquear el turno
- **WHEN** el agente propone un comando que utiliza un binario propio no catalogado, sustitución `$()`, redirección real a archivo (`> /tmp/out`) o modificación de sistema
- **THEN** el backend **NO** abortará el turno con error; desactivará `auto_approved_read_only`, asignará el nivel de riesgo auditado junto con `audit_warning` explicativo en español claro y presentará la tarjeta al operador para que decida si autoriza, edita u omite

---

### Requirement: Auditor Estático de Riesgo en Cuatro Etapas (CommandRiskGuard)
El sistema SHALL auditar y clasificar el riesgo operativo de cada comando propuesto mediante un pipeline léxico determinista de 4 etapas evaluando tokens reales por segmento: Tokenizador/Segmentador por operadores consciente de comillas, Detector de tuberías hacia intérpretes de shell, Análisis de verbos y subcomandos/flags por token exacto (distinguiendo `command -v`/`-V` de ejecuciones `command <bin>`), y Auditor de operadores de redirección. La salida SHALL categorizarse en cuatro niveles (`read_only`, `config_change`, `service_impact`, `destructive`) y proveer en `evaluate_proposal` advertencias explicativas en español claro sin jerga interna de fases.

#### Scenario: Elevación forzosa ante tuberías a intérpretes de shell
- **WHEN** un comando incluye tuberías hacia intérpretes con cualquier espaciado o prefijo como `| bash`, `|bash`, `| sudo sh` o `|/usr/bin/python3`
- **THEN** `CommandRiskGuard` SHALL clasificar el comando automáticamente como `destructive` (🔴) sin importar la etiqueta propuesta por la IA

#### Scenario: Elevación por operadores de redirección sobre comandos de lectura
- **WHEN** un comando con verbo de lectura (p. ej. `cat` o `dmesg`) incluye un operador de redirección hacia archivo fuera de comillas (`>`, `>>` o `tee`)
- **THEN** el auditor SHALL elevar inmediatamente la severidad de `read_only` a `config_change` (🟡) por implicar escritura en el sistema de archivos, mientras mantiene en `read_only` las redirecciones inocuas (`2>&1`, `2>/dev/null`)

#### Scenario: Identificación de impacto en servicios del sistema
- **WHEN** un comando manipula el ciclo de vida de demonios o servicios en vivo (incluyendo nombres que contengan la subcadena `status`, p. ej. `systemctl restart status-api`) o reglas de red activa (`ufw enable|disable`, `iptables`)
- **THEN** el auditor SHALL evaluar el subcomando real (`restart`), clasificar el riesgo en `service_impact` (🟠), emitir en `audit_warning` una explicación clara para el operador (sin usar frases internas como `"Fase de Reconocimiento prohíbe..."`), y reservar `read_only` cuando el subcomando sea `status`, `is-active`, `is-enabled` o `--version`
