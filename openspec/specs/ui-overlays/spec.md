# ui-overlays

## Purpose

Overlays de chrome de producto: diálogos glass de confirmación y menús contextuales con iconos, alineados a DESIGN.md.

## Requirements

### Requirement: Dialog de confirmación glass (A1)
El chrome de producto SHALL usar un dialog glass centrado para confirmaciones destructivas y avisos que hoy dependen de diálogos nativos del OS. El dialog MUST seguir el look definido en `DESIGN.md` (overlay, panel glass, acciones Cancelar + primaria/destructiva). Escape o Cancelar MUST descartar la acción. El producto NO MUST usar `window.confirm` ni `window.alert` para estos flujos una vez migrados.

#### Scenario: Confirmar eliminación
- **WHEN** el usuario elige eliminar una carpeta o conexión y aparece el dialog
- **THEN** el dialog muestra título, cuerpo, impacto relevante si aplica, y solo ejecuta el borrado si confirma

#### Scenario: Cancelar con Escape
- **WHEN** el dialog de confirmación está abierto y el usuario pulsa Escape
- **THEN** el dialog se cierra y no se ejecuta la acción destructiva

### Requirement: Menú contextual con iconos (B3)
El sistema SHALL mostrar un menú contextual de chrome con ítems que incluyen icono Lucide outline + etiqueta. El hover/focus de ítems no destructivos habilitados MUST usar acento sakura (alineado al botón “Nueva conexión”), no cian. El componente `showContextMenu` SHALL soportar la propiedad opcional `disabled?: boolean` en cada `ContextMenuItem`, aplicando la clase `.chrome-context-item.is-disabled` y `aria-disabled="true"` e impidiendo la ejecución de `onSelect` o el cierre del menú al hacer clic sobre un ítem deshabilitado. El menú MUST anclarse al origen del `contextmenu`, cerrarse con Escape o clic fuera, y no bloquear el viewport de terminal con badges sueltos.

#### Scenario: Hover sakura
- **WHEN** el usuario desplaza el puntero sobre un ítem no destructivo habilitado del menú
- **THEN** el ítem refleja hover sakura (fondo/texto) según tokens de diseño

#### Scenario: Ítem de menú contextual deshabilitado
- **WHEN** un `ContextMenuItem` se configura con `disabled: true` (ej. acción de pestaña en extremo izquierdo/derecho)
- **THEN** el ítem se renderiza con clase `.is-disabled` (`aria-disabled="true"`, opacidad atenuada), no presenta efecto hover y hacer clic sobre él no dispara `onSelect` ni cierra el menú

#### Scenario: Cerrar menú
- **WHEN** el menú está abierto y el usuario pulsa Escape o hace clic fuera
- **THEN** el menú se cierra sin ejecutar una acción

### Requirement: Inhabilitación del Menú Contextual Nativo del Navegador
El cliente NekoSSH SHALL bloquear la aparición del menú contextual predeterminado del motor WebView2/browser (`contextmenu` nativo del sistema operativo) en todas las áreas de la aplicación donde no existan componentes con menús contextuales de producto.

#### Scenario: Clic derecho en zona neutra de la aplicación
- **WHEN** el usuario realiza clic derecho en un área sin menú contextual personalizado (barra de título, headers, fondo de ventana, modales, etc.)
- **THEN** el sistema previene el evento nativo y no despliega el menú contextual del navegador
