## ADDED Requirements

### Requirement: Blindaje de Wrappers de Ejecución y Purgas de Bitácora en Auditoría Léxica
`CommandRiskGuard` y `ReadOnlyEnforcer` SHALL desenvolver de forma determinista wrappers encadenados de ejecución (`xargs`, `timeout`, `nice`, `ionice`, `stdbuf`, `setsid`, además de `sudo`, `env`, `nohup`, `time`, `exec` y `command`) para auditar siempre el binario y subcomandos efectivos, y clasificar operaciones destructivas sobre bitácoras (`journalctl --vacuum*`, `--flush`, `--rotate`, `--relinquish-var`) como `config_change`.

#### Scenario: Bloqueo de evasión mediante xargs, timeout, nice, ionice, stdbuf o setsid
- **WHEN** el modelo o el operador propone comandos envueltos como `ps aux | xargs kill -9`, `find /tmp -type f | xargs rm -f`, `timeout 10 systemctl restart nginx`, `nice -n 5 rm -rf /tmp/x`, `ionice -c3 cp a b` o `stdbuf -oL apt-get upgrade -y`
- **THEN** el extractor léxico SHALL resolver el binario subyacente (`kill`, `rm`, `systemctl`, `cp`, `apt-get`), clasificar el nivel real (`destructive`, `service_impact` o `config_change`), adjuntar `audit_warning` explicativo e impedir su auto-ejecución en *Modo Flexible*

#### Scenario: Uso benigno de wrappers y consulta normal de journalctl
- **WHEN** se propone `timeout 5s ping -c 1 8.8.8.8`, `echo a | xargs` o `journalctl -u nginx -n 30 --no-pager`
- **THEN** `CommandRiskGuard` SHALL mantener el riesgo en `read_only` (`Solo lectura · No modifica`), mientras que `journalctl --vacuum-time=1d` o `journalctl --rotate` SHALL elevarse a `config_change`
