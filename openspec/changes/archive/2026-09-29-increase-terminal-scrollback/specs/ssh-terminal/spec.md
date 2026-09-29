## ADDED Requirements

### Requirement: Capacidad y Retención de Buffer de Scrollback
El sistema SHALL configurar cada instancia del emulador `Terminal` con una capacidad de retención de scrollback de al menos 10,000 líneas (`scrollback: 10000`). Todas las terminales creadas en el frontend (tanto el shell principal como cualquier shell secundario o hijo en el contexto de pestaña) MUST preservar hasta 10,000 líneas de historial acumulado en el buffer para permitir la inspección de salidas extensas de comandos y logs remotos sin truncamiento prematuro.

#### Scenario: Retención de salida extensa superior a 1,000 líneas
- **WHEN** un comando en el shell remoto emite una salida continua que excede las 1,000 líneas (por ejemplo `tail -n2000`)
- **THEN** el buffer del emulador retiene la totalidad de las 2,000 líneas y el usuario puede desplazarse hacia arriba hasta el inicio de dicha salida sin pérdida de datos

#### Scenario: Límite máximo de buffer fijado en 10,000 líneas
- **WHEN** la salida continua del shell remoto excede las 10,000 líneas en el buffer
- **THEN** el emulador xterm.js descarta de forma circular únicamente las líneas que exceden las 10,000 líneas retenidas en el historial previo, garantizando estabilidad y consumo de memoria controlado
