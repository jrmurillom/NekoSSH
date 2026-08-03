## MODIFIED Requirements

### Requirement: Desconexión durante edición
Si la sesión SSH se desconecta o cierra mientras hay ediciones activas (es decir, en fase de confirmación de subida, subida en curso, o con cambios locales detectados pendientes de procesar), el sistema SHALL detener watchers, no completar una subida pendiente, cerrar cualquier dialog A1 de subida sin aplicar cambios remotos, e informar al usuario. El archivo local temporal de dichas ediciones activas SHOULD conservarse de forma temporal para no perder trabajo del usuario. Si no hay ediciones activas (todas las sesiones de edición asociadas están limpias y en estado de vigilancia pasiva sin cambios pendientes), el sistema SHALL cerrar las sesiones de edición de manera silenciosa, detener los watchers y eliminar los archivos temporales sin mostrar ninguna advertencia al usuario.

#### Scenario: Disconnect con confirm abierto
- **WHEN** se desconecta la sesión SSH con un dialog “¿Subir al servidor?” abierto
- **THEN** el dialog se cierra sin subir y el usuario recibe aviso de que no se pudo subir únicamente si el archivo temporal tiene cambios locales reales pendientes de guardar; de lo contrario se cierra en silencio sin emitir advertencia

#### Scenario: Trabajo local tras disconnect
- **WHEN** ocurre desconexión mid-edit
- **THEN** el sistema deja de intentar subir automáticamente y conserva el temp local únicamente si está sucio/dirty, limpiándolo silenciosamente si ya estaba sincronizado con el servidor
