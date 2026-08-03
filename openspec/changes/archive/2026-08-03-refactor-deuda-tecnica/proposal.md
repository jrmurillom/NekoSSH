## Why

Se requiere atender exhaustivamente todos los hallazgos de severidad **Crítica** y **Alta** identificados en la auditoría técnica del sistema NekoSSH:
1. `RUST-CRIT-1`: Riesgo de panics en cadena por Mutex Lock Poisoning mediante `.lock().unwrap()` en Rust.
2. `RUST-CRIT-2`: Advertencia de visibilidad pública en `SshConnections` ([lib.rs:40](file:///c:/Users/Roberto/Documents/antigravity/NekoSSH/app/src-tauri/src/lib.rs#L40)).
3. `ARCH-HIGH-1`: Monolito frontend en `main.ts` (2,110+ líneas).
4. `ARCH-HIGH-2`: Monolito backend en `lib.rs` (1,410+ líneas).

## What Changes

1. **Protección contra Mutex Lock Poisoning (`RUST-CRIT-1`)**:
   - Reemplazar las llamadas vulnerables a `.lock().unwrap()` por manejo seguro de locks (`unwrap_or_else` / `match`) en `external_edit.rs`, `lib.rs` y `edit_session.rs` para prevenir colapsos irrecuperables del proceso backend.
2. **Corrección de Visibilidad en Rust (`RUST-CRIT-2`)**:
   - Ajustar la visibilidad de `LiveSsh` a pública (`pub struct LiveSsh`) en `lib.rs`, resolviendo el warning de la tupla `SshConnections`.
3. **Modularización del Monolito Frontend (`ARCH-HIGH-1`)**:
   - Desacoplar `main.ts` en submódulos especializados en `app/src/modules/`:
     - `connection-tree-helper.ts`: Agrupación y ordenamiento del árbol de carpetas/perfiles.
     - `sftp-path-helper.ts`: Normalización y navegación de rutas SFTP.
4. **Modularización del Monolito Backend (`ARCH-HIGH-2`)**:
   - Desacoplar responsabilidades en `lib.rs` organizando helpers de base de datos y comandos PTY.
5. **Suite de Pruebas Unitarias de Alto Valor**:
   - Pruebas unitarias de jerarquía de carpetas, ordenamiento y perfiles huérfanos (`connection-tree-helper.test.ts`).
   - Pruebas unitarias de navegación SFTP y sanitización de barras (`sftp-path-helper.test.ts`).
   - Pruebas unitarias de Rust (`cargo test`) para la máquina de estados de edición externa (`edit_session.rs`) y escapado de `sudo cp` (`elevated_upload.rs`).

## Capabilities

### New Capabilities
- Ninguna.

### Modified Capabilities
- `app-branding`: Estabilidad, concurrencia segura y mantenibilidad del sistema sin alterar la interfaz gráfica.

## Impact

- **Archivos afectados**:
  - `app/src-tauri/src/lib.rs`, `edit_session.rs`, `external_edit.rs`, `elevated_upload.rs`.
  - `app/src/main.ts` y nuevos submódulos en `app/src/modules/`.
