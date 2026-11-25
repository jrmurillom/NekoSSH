## ADDED Requirements

### Requirement: Eco TTY Resiliente a ANSI, Detección de Espera Interactiva y Estado HITL por Sesión
El colector `PtyContextTap` y la ventana `"neko-ai"` SHALL limpiar el eco del wrapper centinela incluso cuando el shell remoto intercale secuencias ANSI de resaltado (p. ej. `zsh-syntax-highlighting`), advertir al operador cuando un comando supervisado quede esperando entrada interactiva o encuentre un shell no-POSIX (`fish`), y conservar por `terminal_id` toda tarjeta pendiente, fallo reciente y rollback preparado entre cambios de pestaña o reapertura de ventana.

#### Scenario: Supresión limpia de eco en zsh con resaltado ANSI intercalado
- **WHEN** el shell remoto devuelve el eco de `;__neko_rc=$?;printf '\n%s_%s:%d__\n' '__NEKO_END' '<nonce>' "$__neko_rc"` intercalado con códigos de color ANSI CSI (`\x1b[32m...`)
- **THEN** `strip_echo_plumbing` SHALL ignorar secuencias ANSI durante la detección de fronteras y ocultar íntegro el sufijo de eco antes de enviarlo a `xterm.js`

#### Scenario: Aviso en vivo cuando el comando espera entrada interactiva o encuentra shell fish
- **WHEN** mientras hay un centinela activo llegan patrones de espera interactiva (`[sudo] password`, `password:`, `passphrase`, `[y/N]`) o errores de sintaxis `fish`, o transcurre el umbral de espera en tarjeta activa
- **THEN** el backend SHALL emitir `ai-pty-waiting-input` una vez por centinela y `"neko-ai"` SHALL mostrar un aviso claro indicando que la terminal espera respuesta del operador (o pulsación de `Ctrl+C`)

#### Scenario: Preservación de tarjeta pendiente y rollback por terminal_id
- **WHEN** el operador tiene una tarjeta pendiente de autorización o un contexto de fallo/rollback preparado en el shell `A`, cambia momentáneamente al shell `B` (o cierra y reabre `"neko-ai"` en la misma ejecución de NekoSSH) y regresa al shell `A`
- **THEN** `"neko-ai"` SHALL restaurar exactamente la tarjeta pendiente, comando editado en borrador, contexto de error y rollback del shell `A` sin perder estado
