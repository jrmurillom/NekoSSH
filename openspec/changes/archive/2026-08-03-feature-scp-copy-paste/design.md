## Context

Para mejorar la productividad al trabajar con múltiples servidores remotos activos, diseñamos un portapapeles en memoria del cliente para copiar archivos usando "copiar scp" y "pegar scp".

## Goals / Non-Goals

**Goals:**
- Implementar un búfer global en TypeScript para registrar el `terminal_id` origen y el `remote_path` del archivo copiado.
- Exponer las opciones "copiar scp" y "pegar scp" en el menú contextual de los archivos y del contenedor del explorador de archivos.
- Crear un comando en el backend Rust `sftp_copy_between_sessions` que multiplexe SFTP y realice lectura/escritura mediante un streaming de chunks (64 KiB) directamente en la memoria del cliente local (sin tocar disco duro).
- Refrescar la carpeta de destino automáticamente al finalizar la copia.

**Non-Goals:**
- Copiar directorios de manera recursiva (se restringe a archivos individuales en la primera fase del diseño).
- Realizar transferencias directas de servidor a servidor (scp ssh bypass) debido a dependencias de claves e IPs públicas.

## Decisions

### Comando de Streaming Inter-sesión en Rust
- **Decisión**: Añadir `sftp_copy_between_sessions` en `app/src-tauri/src/external_edit.rs` (o `lib.rs`).
- **Detalle**: El comando recibe `source_terminal_id`, `source_path`, `target_terminal_id` y `target_path`. Abre un SFTP reader y un SFTP writer concurrentemente, copiando los bytes en un loop con un buffer local en Rust de 64 KiB:
  ```rust
  let mut buffer = [0u8; 65536];
  loop {
      let n = file_src.read(&mut buffer)?;
      if n == 0 { break; }
      file_tgt.write_all(&buffer[..n])?;
  }
  ```

### Portapapeles en Frontend
- **Decisión**: Declarar variables globales en `app/src/main.ts` para retener la información del archivo origen copiado:
  ```typescript
  let scpClipboard: {
    terminalId: string;
    path: string;
    name: string;
  } | null = null;
  ```

## Risks / Trade-offs

- **Riesgo**: Que la velocidad de transferencia sea limitada por la conexión a internet del cliente (tasa de subida/bajada).
- **Mitigación**: Es un trade-off necesario para garantizar la seguridad de las claves privadas en el host local y saltear restricciones de red entre servidores. Al usar chunks de 64 KiB y buffers en memoria, la transferencia opera al máximo de la capacidad del canal TCP.

### Corrección de Ruta (Fix): Conexión Explorer On-Focus

- **Decisión**: Para evitar que todas las pestañas de SSH compartan la misma ruta remota y árbol de archivos, vinculamos el estado del explorador (`explorerCwd` y `explorerRoot`) individualmente a cada objeto `ActiveTerminal`.
- **Implementación**:
  - En la interfaz `ActiveTerminal`, agregamos `explorerCwd?: string` y `explorerRoot?: ExplorerNodeState | null`.
  - En `switchActiveTerminal`, guardamos las variables globales actuales en la terminal saliente y cargamos/restauramos los estados de la terminal entrante.
  - Se restringe el renderizado y listado automático de SFTP si la vista de archivos no está activa (`panelFiles.classList.contains("active") == false`), optimizando el rendimiento y evitando colisiones de sesión.
