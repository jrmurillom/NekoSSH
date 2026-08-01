## ADDED Requirements

### Requirement: Dialog de confirmación glass (A1)
El chrome de producto SHALL usar un dialog glass centrado para confirmaciones destructivas y avisos que hoy dependen de diálogos nativos del OS. El dialog MUST seguir el look definido en `DESIGN.md` (overlay, panel glass, acciones Cancelar + primaria/destructiva). Escape o Cancelar MUST descartar la acción. El producto NO MUST usar `window.confirm` ni `window.alert` para estos flujos una vez migrados.

#### Scenario: Confirmar eliminación
- **WHEN** el usuario elige eliminar una carpeta o conexión y aparece el dialog
- **THEN** el dialog muestra título, cuerpo, impacto relevante si aplica, y solo ejecuta el borrado si confirma

#### Scenario: Cancelar con Escape
- **WHEN** el dialog de confirmación está abierto y el usuario pulsa Escape
- **THEN** el dialog se cierra y no se ejecuta la acción destructiva

### Requirement: Menú contextual con iconos (B3)
El sistema SHALL mostrar un menú contextual de chrome con ítems que incluyen icono Lucide outline + etiqueta. El hover/focus de ítems no destructivos MUST usar acento sakura (alineado al botón “Nueva conexión”), no cian. El menú MUST anclarse al origen del `contextmenu`, cerrarse con Escape o clic fuera, y no bloquear el viewport de terminal con badges sueltos.

#### Scenario: Hover sakura
- **WHEN** el usuario desplaza el puntero sobre un ítem no destructivo del menú
- **THEN** el ítem refleja hover sakura (fondo/texto) según tokens de diseño

#### Scenario: Cerrar menú
- **WHEN** el menú está abierto y el usuario pulsa Escape o hace clic fuera
- **THEN** el menú se cierra sin ejecutar una acción
