## Context

En start_external_edit de NekoSSH, cuando el usuario solicita editar un archivo remoto, el sistema consultaba si existía un registro en SharedEditSessions. Si existía y no estaba en fase Closed, la función omitía la descarga SFTP y abría directamente la ruta local temporal ya existente. Dado que los editores externos se ejecutan como procesos independientes del sistema operativo, las sesiones no cambian a Closed mientras la terminal permanezca activa, causando que aperturas posteriores sobre el mismo archivo reabran una copia local obsoleta en lugar de descargar la versión actualizada del servidor.

## Goals / Non-Goals

**Goals:**
- Asegurar que cada invocación de start_external_edit descargue siempre y de forma obligatoria la versión actual del archivo desde el servidor remoto vía SFTP.
- Si existe una sesión previa para el mismo (terminal_id, remote_path), cancelar y remover su watcher anterior antes de iniciar el nuevo ciclo.
- Registrar el nuevo baseline (aseline_fingerprint) del archivo descargado y levantar un nuevo watcher managed.
- Mantener intacto el flujo de vigilancia local y el diálogo A1 de confirmación de subida cuando el usuario guarda cambios en su editor externo.

**Non-Goals:**
- No alterar la configuración de editor preferido ni la asociación del sistema operativo (prompt de Windows).
- No modificar el límite de tamaño de 10 MiB ni la heurística de archivos binarios.

## Decisions

### Decisión 1: Reemplazo explícito de sesión y limpieza de watcher en start_external_edit
- **Razón**: Al solicitar  Editar, el usuario espera explícitamente ver el contenido actual del servidor. Si existe una sesión previa, se invoca stop_watcher sobre el edit_id anterior y se remueve de SharedEditSessions, procediendo a crear un nuevo edit_id con descarga SFTP fresca.
- **Alternativas consideradas**:
  - *Mantener la reutilización y solo recargar si el hash remoto cambia*: Descartado por añadir complejidad y llamadas adicionales de sondeo previo cuando el usuario ya solicitó expresamente abrir el archivo.

### Decisión 2: Actualización de egister_or_reuse en EditSessionRegistry
- **Razón**: EditSessionRegistry debe soportar el reemplazo forzado o actualización del registro (egister_or_replace), actualizando el index y y_id de modo que la clave (terminal_id, remote_path) siempre apunte a la sesión activa más reciente.

## Risks / Trade-offs

- **[Riesgo] Pérdida de cambios locales no subidos si se reabre el archivo**: Si el usuario editó el archivo local, no lo subió y vuelve a hacer clic en Editar, la nueva descarga reemplazará el contenido del archivo temporal.
  - *Mitigación*: Al hacer clic en Editar, la intención del usuario es obtener y editar el archivo del servidor. El nuevo baseline se establece sobre el contenido descargado.