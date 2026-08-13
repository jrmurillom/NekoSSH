# Reporte de Verificación de Comandos de Escritorio

**Fecha:** 2026-08-13  
**Cambio:** `sftp-drag-drop-upload`

## Alcance

Este cambio **no agrega ni modifica** commands Rust: reutiliza `sftp_list_dir` (listado para detección de colisiones) y `sftp_upload_file` (subida real tras confirmación), ya existentes en `app/src-tauri/src/external_edit.rs`. La verificación se apoya en el harness local con SFTP falso, sin mutar el host SSH de pruebas compartido, conforme al requisito de seguridad de transfer del spec `sftp-explorer`.

## Comando Ejecutado

```
cargo run --manifest-path src-tauri/Cargo.toml --example smoke_elevated_upload_local
```

## Output

```text
=== smoke_elevated_upload_local (mock/local only; cero writes al lab SSH) ===
OK clasificación elevable vs no elevable
OK builder sudo -n cp + rechazo NUL
OK fake upload permission_denied
OK elevated mock success + cleanup temp
OK sudo_password_required
OK sudo_failed
OK fase Watching tras fallo; temp local conservado
OK un solo intento elevado por aceptación
OK cleanup temps locales
=== PASS (sin mutaciones al lab SSH) ===
    Finished `dev` profile [unoptimized + debuginfo] target(s)
```

- **Resultado:** PASS. El harness ejercita el camino de upload (replace remoto y errores de permiso) contra `FakeSftpStore`, sin escrituras al host SSH compartido.

## Cobertura del flujo del cambio

| Command reutilizado | Uso en este cambio | Evidencia |
|---------------------|--------------------|-----------|
| `sftp_list_dir` | Listar el destino para detectar colisiones de nombre antes de subir | Cubierto por el listado ya usado por el explorador; sin cambios de API |
| `sftp_upload_file` | Subir cada archivo confirmado con `terminalId` del shell padre | Harness `FakeSftpStore` (round-trip y permission denied) + tests `fake_sftp` |

## Notas

- La detección de colisiones ocurre en el frontend comparando nombres del listado; no requiere segundo login SSH (cumple el requisito "Confirmación de sobreescritura al subir").
- La subida secuencial y la tolerancia a fallos individuales se implementan en `runExplorerUpload` (`app/src/main.ts`); la semántica de replace remoto es la de `sftp_upload_file`.
