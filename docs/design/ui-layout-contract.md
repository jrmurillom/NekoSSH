---
description: SSOT estructural — shell de layout, zonas y patrones de página
alwaysApply: false
---

# UI Layout Contract (SSOT estructural)

Fuente de verdad de **estructura**: shell, zonas, patrones de página y navegación entre regiones.  
El **look** (tokens, tipografía, acentos planos) vive en `DESIGN.md` — no duplicar colores aquí.

Usar términos portables: *layout shell*, *zonas*, *patrones de página*, *paneles*, *áreas de trabajo*.

---

## 1. Layout shell (ventana principal)

Composición por defecto:

```
┌──────────────────────────────────────────────────────────┐
│                     APP CHROME (opcional)                │
├────────────────┬─────────────────────────────────────────┤
│                │                                         │
│  SIDEBAR       │           MAIN WORKSPACE                │
│  (conexiones / │   (pestañas: terminales / editor)       │
│   archivos)    │                                         │
│                │                                         │
└────────────────┴─────────────────────────────────────────┘
```

| Zona | Rol | Contenido típico |
|------|-----|------------------|
| **Sidebar** | Navegación y listados | Perfiles/grupos de conexión; más adelante árbol de archivos remoto |
| **Main workspace** | Trabajo primario | Sesiones de terminal y, en fases posteriores, editor |
| **App chrome** | Controles de ventana/app | Solo si el runtime lo exige; no meter marketing ni clutter |

Reglas:

- Una composición primaria: sidebar + workspace — no dashboard de widgets en el primer viewport de trabajo.
- El sidebar no compite visualmente con el stream de la terminal: densidad de lista, no tarjetas decorativas.
- El workspace es el ancla funcional; paneles secundarios no deben tapar la zona de entrada de la terminal sin acción explícita del usuario.

---

## 2. Patrones de página / vista

| Patrón | Cuándo | Comportamiento |
|--------|--------|----------------|
| **Connection list** | CRUD de carpetas + conexiones | Árbol en sidebar (carpeta → conexiones); modal de conexión; rename inline (carpeta/conexión) desde menú contextual |
| **Active session** | Conexión establecida | Pestaña (o equivalente) en workspace con viewport de terminal a pantalla útil |
| **Empty workspace** | Sin sesión | Estado vacío claro + CTA hacia conectar/crear perfil — sin relleno decorativo |
| **Settings / appearance** | Preferencias visuales | Fondo, opacidad, estilo de cursor — modal o vista dedicada; no ensuciar el shell |

---

## 3. Zonas reservadas por fase (contrato evolutivo)

| Fase | Sidebar | Main workspace |
|------|---------|----------------|
| Fase 1 | Carpetas (un nivel) + conexiones SSH anidadas; rename inline de carpeta; add-folder | Terminal(es) |
| Fase 2 | Perfiles + **explorador remoto** (árbol SFTP lazy; path editable + Ir + Actualizar) | Terminal + sync cwd vía **OSC 7**; “Abrir en Terminal” |
| Fase 3 | + menú/doble clic **Editar** en archivos; settings de editor externo | Edición **fuera** del proceso (editor OS); A1 “¿Subir al servidor?”; tras fallo de permisos A1 opcional “Subir con sudo” (sin password UI); sin pestaña Monaco |
| Fase 3b (futuro) | (igual) | + editor Monaco en pestaña/área (diferido) |
| Fase 4a | Botón **Snippets** temático + engrane en `sidebar-footer` (prefs editor/fondo/opacidad en popover) → modal | Modal glass: chips + lista plana + búsqueda + CRUD; copy clipboard; sin tercera columna |
| Fase 4b (futuro) | (definir Petdex) | Mascotas diferidas; sin romper shell |

No inventar una tercera columna permanente sin actualizar este contrato.

---

## 4. Interacción entre zonas

- Cerrar una pestaña de sesión debe liberar la sesión asociada (PTY + SFTP en la misma Session SSH). Cerrar la ventana / salir de la app debe cerrar **todas** las Sessions SSH activas (contrato de ciclo de vida).
- Acciones de sidebar que abren sesión actualizan el workspace (nueva pestaña o foco), no navegan a una “página” fuera del shell.
- **Sesión SSH caída:** el viewport de la pestaña muestra aviso de desconexión + **Ctrl+R para reconectar** (manual, sin auto-reconnect). El indicador bajo la pestaña (dot + texto) refleja `connecting` / `connected` / `disconnected` / `error`. Ctrl+R solo aplica con pestaña desconectada.
- **Árbol de conexiones:** carpetas con chevron Lucide + icono carpeta + nombre + `+` (nueva conexión); sin basurero en la fila. Conexiones indentadas. Acción add-folder (FolderPlus). Rename de carpeta: menú contextual → input inline (Enter/Escape/blur); no doble clic. Eliminar carpeta: menú contextual → dialog glass A1 con conteo (cascade). **Click primario en toda la fila** de carpeta alterna expand/collapse; el botón `+` usa `stopPropagation` para no disparar el toggle.
- **Tarjeta de conexión:** compacta (nombre + `user@host:port` en cyan); sin label de auth/túnel. Sin lápiz ni basurero inline. Icono Copy copia `user@host`. **Doble clic** abre la sesión SSH; click simple solo resalta. Editar (modal), renombrar (inline) y eliminar (confirm A1) viven en el menú contextual B3.
- Menús contextuales anclan a la zona de origen (lista vs terminal vs explorador); no flotar badges sueltos sobre el viewport de terminal.
- **Confirmaciones de producto:** dialog glass **centrado** (ver `DESIGN.md` § Confirmaciones). Usar para eliminar carpeta/conexión, cerrar todas las terminales y cualquier acción destructiva con impacto. Escape / Cancelar descarta; no usar `window.confirm` del OS en flujos nuevos.
- **Menú contextual:** patrón de ítems con icono Lucide + label; hover con `--color-sakura-neon` (mismo rosa que “Nueva conexión” / `.btn-primary`), no pastel ni cian. Anclado al punto de invocación (clic derecho); clic fuera o Escape cierra. Look en `DESIGN.md` § Menús contextuales.
- **Sync Fase 2:** el explorador **no** sigue el `cd` tipado automáticamente. Navegación = **SFTP** (chevron Lucide expand/collapse, click = abrir, iconos Subir/Ir/Actualizar). “Abrir en Terminal” = `cd` al PTY.
- Iconografía de chrome: Lucide outline + color por tema (`currentColor`); ver `DESIGN.md` § Iconografía.
---

## 5. Qué actualizar aquí

Cambios de shell, columnas, zonas, patrones de página, o reglas de navegación entre regiones → este archivo.  
Cambios de tokens o look de un control → `DESIGN.md`.
