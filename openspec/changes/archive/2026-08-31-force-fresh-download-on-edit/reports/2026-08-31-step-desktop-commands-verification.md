# Step Desktop Commands Verification Report

- **Fecha:** 2026-08-31
- **Change:** force-fresh-download-on-edit
- **Superficie:** desktop-commands

## 1. Verificación de Commands y Backend

Se verificó el comando Tauri start_external_edit y el harness de prueba smoke_edit_session_local:

### Operación: start_external_edit
- **Comportamiento verificado:**
  1. Consulta de sesión previa para (terminal_id, remote_path).
  2. Limpieza y detención del watcher asociado a la sesión previa (stop_watcher(&watchers, &prev_id)).
  3. Descarga obligatoria y fresca del archivo remoto vía SFTP (sftp_download_file_blocking).
  4. Generación de nueva ruta temporal y cálculo de aseline_fingerprint.
  5. Registro de la sesión reemplazada en EditSessionRegistry (egister_or_replace).
  6. Apertura del editor externo o asociación del SO (open_local_in_editor).
  7. Inicio del nuevo watcher managed (spawn_file_watcher_managed).

### Harness: smoke_edit_session_local
- **Comportamiento verificado:**
  - Ejecución de cargo run --example smoke_edit_session_local.
  - Preferencias de editor, descarga/subida fake SFTP, rechazo por tamaño >10 MiB, heurística binaria y reemplazo de sesión con baseline fresco y coalesce de confirmación validados al 100% sin writes al host SSH de lab.

## 2. Resultado General
- **Estado:** PASS (56 unit tests + smoke harness exitoso)
- **Degradación PTY:** Ninguna.