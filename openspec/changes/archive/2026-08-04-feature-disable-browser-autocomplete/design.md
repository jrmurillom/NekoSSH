## Context

En el desarrollo de aplicaciones basadas en Tauri, los motores de WebView nativos heredan comportamientos predeterminados del navegador, como el guardado de historial de autocompletado en campos `<input>`. Este comportamiento interfiere con la estética de aplicación nativa cyberpunk de NekoSSH.

## Goals / Non-Goals

**Goals:**
- Desactivar completamente el menú desplegable de autocompletado nativo del navegador en todos los campos de entrada de la aplicación.
- Documentar en el archivo de estándares principales (`docs/base-standards.md`) la obligatoriedad de este comportamiento para todos los futuros desarrollos.

**Non-Goals:**
- Deshabilitar el filtrado reactivo del lado del frontend en buscadores (como el filtro de snippets o historial).

## Decisions

### 1. Inserción de `autocomplete="off"` en Inputs de HTML
Se modificará cada etiqueta `<input>` en `app/index.html` para incorporar de manera explícita el atributo `autocomplete="off"`. 
* *Alternativa considerada*: Desactivar el autocompletado a nivel de formulario general (`<form autocomplete="off">`).
  * *Razón de rechazo*: Varios inputs en `index.html` están sueltos (fuera de etiquetas `<form>`), por lo que la especificación individual en cada `<input>` garantiza cobertura total.

### 2. Registro en el SSOT Técnico
Se modificará `docs/base-standards.md` agregando una regla bajo la sección de desarrollo frontend / estándares de interfaz:
> "Todos los elementos `<input>` de texto, números o búsquedas en la aplicación SHALL incluir el atributo `autocomplete="off"` para evitar que la WebView nativa guarde historial o muestre desplegables de autocompletado propios del navegador."

## Risks / Trade-offs

- **[Riesgo] Pérdida de comodidad de relleno automático**:
  * El usuario perderá la capacidad de que el sistema recuerde contraseñas o nombres de servidor a través de la base de datos de WebView.
  * *Mitigación*: Esto es deseable, ya que NekoSSH ya cuenta con un sistema de perfiles de conexión (`ConnectionProfile`) guardados de forma segura en su base de datos local SQLite para gestionar los servidores y credenciales, por lo que el relleno automático del navegador es redundante y menos seguro.
