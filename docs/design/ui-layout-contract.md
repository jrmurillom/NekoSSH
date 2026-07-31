---
description: SSOT estructural — shell de layout, zonas y patrones de página
alwaysApply: false
---

# UI Layout Contract (SSOT estructural)

Fuente de verdad de **estructura**: shell, zonas, patrones de página y navegación entre regiones.  
El **look** (tokens, tipografía, glow) vive en `DESIGN.md` — no duplicar colores aquí.

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
| **Connection list** | CRUD de perfiles | Lista/grupos en sidebar; formulario o detalle sin romper el shell |
| **Active session** | Conexión establecida | Pestaña (o equivalente) en workspace con viewport de terminal a pantalla útil |
| **Empty workspace** | Sin sesión | Estado vacío claro + CTA hacia conectar/crear perfil — sin relleno decorativo |
| **Settings / appearance** | Preferencias visuales | Fondo, opacidad, estilo de cursor — modal o vista dedicada; no ensuciar el shell |

---

## 3. Zonas reservadas por fase (contrato evolutivo)

| Fase | Sidebar | Main workspace |
|------|---------|----------------|
| Fase 1 | Perfiles / grupos | Terminal(es) |
| Fase 2 | + explorador remoto | Terminal + sync de ruta |
| Fase 3 | (igual) | + editor en pestaña/área |
| Fase 4 | (definir) | Snippets / extensiones sin romper shell |

No inventar una tercera columna permanente sin actualizar este contrato.

---

## 4. Interacción entre zonas

- Cerrar una pestaña de sesión debe liberar la sesión asociada (contrato de ciclo de vida; detalle técnico en design del change).
- Acciones de sidebar que abren sesión actualizan el workspace (nueva pestaña o foco), no navegan a una “página” fuera del shell.
- Menús contextuales anclan a la zona de origen (lista vs terminal); no flotar badges sueltos sobre el viewport de terminal.

---

## 5. Qué actualizar aquí

Cambios de shell, columnas, zonas, patrones de página, o reglas de navegación entre regiones → este archivo.  
Cambios de tokens o look de un control → `DESIGN.md`.
