## ADDED Requirements

### Requirement: Oferta de reintento con sudo tras fallo de upload por permisos
Tras un upload SFTP normal fallido que el sistema clasifique como error de permisos o escritura denegada sobre el path remoto de una sesión de edición externa, el sistema SHALL mostrar un dialog glass A1 que ofrezca **«Subir con sudo»** (aceptar) y cancelar. El sistema MUST NOT ofrecer sudo cuando el fallo no sea candidato a elevación (p. ej. desconexión, sesión cerrada, path inexistente). El sistema MUST NOT usar sudo en el primer intento de subida tras el confirm «¿Subir al servidor?».

#### Scenario: Fallo por permisos ofrece sudo
- **WHEN** el usuario confirma subir y el upload SFTP normal falla por permisos/escritura denegada
- **THEN** el sistema muestra un dialog A1 con la acción «Subir con sudo» y la opción de cancelar

#### Scenario: Fallo no elevable no ofrece sudo
- **WHEN** el upload SFTP normal falla por desconexión u otro error no clasificado como permisos
- **THEN** el sistema informa el error y no muestra la oferta «Subir con sudo»

#### Scenario: Upload normal exitoso no menciona sudo
- **WHEN** el usuario confirma subir y el upload SFTP normal tiene éxito
- **THEN** el sistema actualiza la baseline y no muestra dialog de sudo

#### Scenario: Cancelar oferta de sudo
- **WHEN** el dialog «Subir con sudo» está abierto y el usuario cancela o pulsa Escape
- **THEN** el sistema no ejecuta el path elevado, el remoto no se modifica por ese intento, y el archivo temporal local dirty se conserva

### Requirement: Path elevado mínimo con sudo no interactivo
Si el usuario acepta «Subir con sudo», el sistema SHALL intentar un único path elevado mínimo sobre la misma sesión SSH: subir el contenido a un temporal remoto writable por el usuario y copiarlo al destino con un comando `sudo` **no interactivo** (p. ej. `sudo -n cp` o equivalente documentado en design). El sistema MUST NOT solicitar ni capturar la contraseña de sudo en la UI. Si el intento elevado tiene éxito, el sistema SHALL actualizar la baseline de vigilancia como en un upload normal.

#### Scenario: Aceptar Subir con sudo con éxito
- **WHEN** el usuario acepta «Subir con sudo» y el host permite el comando no interactivo
- **THEN** el sistema aplica el contenido local al path remoto de origen vía el path elevado y actualiza la baseline

#### Scenario: Un solo intento elevado por aceptación
- **WHEN** el usuario acepta «Subir con sudo»
- **THEN** el sistema realiza un único intento elevado para esa aceptación (sin bucles automáticos de reintento)

### Requirement: Fallo limpio si sudo pide password o no está disponible
Si el path elevado requiere contraseña, no dispone de TTY interactivo usable, o el comando sudo/exec falla, el sistema SHALL abortar el intento, informar al usuario con un mensaje de error claro en español, y MUST NOT abrir un prompt de password. El archivo temporal local dirty MUST conservarse. El sistema MUST NOT reintentar en silencio.

#### Scenario: sudo requiere password
- **WHEN** el comando elevado falla porque sudo pediría contraseña (p. ej. `sudo -n` rechaza)
- **THEN** el sistema muestra error, no pide password en UI, y conserva el temp local dirty

#### Scenario: exec o sudo falla por otra causa
- **WHEN** el path elevado falla (comando, permisos de sudoers, timeout, etc.)
- **THEN** el sistema detiene el flujo de elevación con alert de error y conserva el temp local dirty

### Requirement: Verificación del path sudo sin mutar el lab SSH
La verificación automatizada y las corridas de agente que cubran el reintento con sudo MUST NOT escribir, sobrescribir ni borrar archivos en hosts SSH de prueba compartidos. Los tests del path elevado SHALL usar mocks, fixtures locales, fake SFTP y/o harness de exec in-process. El producto MAY usar sudo en uso real cuando el usuario acepta «Subir con sudo»; esta restricción aplica a desarrollo/verificación contra el lab, no al comportamiento de producto.

#### Scenario: Tests del path elevado sin writes al lab
- **WHEN** se ejecutan unit/integration tests del reintento con sudo
- **THEN** no se realizan upload/exec/delete contra el host SSH de pruebas compartido; la evidencia usa mock o fixture local

#### Scenario: Verificación de agente del sudo sin mutar el lab
- **WHEN** un agente ejecuta desktop-commands o desktop-ui verification del flujo sudo sin sandbox remoto disposable documentado
- **THEN** no escribe en paths remotos del lab; valida con mocks/fixtures locales y documenta N/A para writes remotos live
