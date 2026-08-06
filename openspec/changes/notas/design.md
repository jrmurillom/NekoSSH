## Context
El sistema cuenta con una base de datos local SQLite y una interfaz modular dividida en pestañas laterales (Servidores, Archivos, Monitor). Implementaremos la sección de Notas de manera ligera e integrada en esta arquitectura usando tablas existentes y persistencia nativa en SQLite.

## Goals / Non-Goals

**Goals:**
- Añadir el listado y CRUD de notas en la base de datos local SQLite y su integración en el frontend.
- Proveer un modal de edición flotante Cyber-Sakura con título inline editable y editor textarea plano (Markdown nativo) con auto-guardado al cerrar o perder el foco.
- Integración adaptada automáticamente a los 8 temas de color.

**Non-Goals:**
- Renderizado interactivo HTML/Markdown enriquecido (WYSIWYG) para mantener la app ultra ligera.
- Integración de dependencias externas pesadas para edición de notas.

## Decisions

### 1. Persistencia de Datos (Base de Datos SQLite Local)
- **Decisión:** Crear una tabla `notes` en SQLite con los campos `id` (INTEGER PRIMARY KEY), `title` (TEXT), `content` (TEXT), `updated_at` (DATETIME).
- **Alternativa considerada:** Almacenamiento en archivos locales `.md` en disco. Se descartó temporalmente para simplificar la integración en SQLite mediante Tauri y evitar problemas de permisos de archivos o pérdidas accidentales.

### 2. Formato del Editor
- **Decisión:** Usar un elemento `<textarea>` nativo con tipografía monospace (`--font-mono`), bordes neón del color de acento y auto-guardado en eventos `blur` y `close`.
- **Alternativa considerada:** Integrar un editor WYSIWYG comercial. Se descartó por rendimiento y consistencia con las directrices de ligereza.

### 3. Arquitectura del Código (Modularización)
- **Decisión:** Crear un archivo dedicado `app/src/modules/notes-helper.ts` que centralice toda la lógica UI, estado y llamadas Tauri de la pestaña de Notas. `main.ts` únicamente importará e inicializará el módulo llamando a `initNotesTab()`.
- **Alternativa considerada:** Añadir el código directamente dentro de `main.ts` (descartado para evitar acoplamiento y crecimiento innecesario del archivo).

## Risks / Trade-offs

- **[Riesgo] Pérdida de cambios en edición concurrente o de-bounce ineficiente** → **Mitigación:** Asegurar que el guardado se invoque de forma robusta e síncrona en el evento de cierre del modal (clic en "X" o `Esc`) y al salir del foco (`blur`) del campo de texto, además de un de-bounce ligero mientras escribe.
