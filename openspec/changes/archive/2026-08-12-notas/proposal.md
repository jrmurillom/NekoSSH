## Why

Los desarrolladores necesitan un espacio integrado para tomar notas rápidas, apuntar comandos de uso frecuente o guardar fragmentos de configuración durante sus sesiones SSH, sin tener que recurrir a herramientas externas de notas.

## What Changes

- Añadir una nueva pestaña "Notas" en el menú de navegación de la barra lateral (cuarto tab).
- Mostrar un listado de notas creadas bajo la cabecera "Notas".
- Añadir un botón `(+)` para la creación rápida de notas vacías.
- Mostrar una ventana modal flotante (overlay glassmorphic) al hacer click en una nota para visualizar y editar su contenido.
- Permitir renombrar la nota mediante edición inline en el título dentro del modal flotante.
- Añadir un botón de eliminar con confirmación rápida dentro del modal flotante.
- Implementar guardado automático de notas en la base de datos SQLite (al cerrar el modal o perder el foco del editor).

## Capabilities

### New Capabilities
- `notes-manager`: Permite gestionar y persistir notas rápidas del usuario vinculadas a la base de datos SQLite local de la aplicación, mostrándolas y editándolas a través de una interfaz interactiva.

### Modified Capabilities
