# Desktop Commands Verification

- Date: 2026-07-31
- Change: `fase3-external-edit-sync`
- Branch: `feature/fase3-external-edit-sync`
- Agent: Cursor Auto (opsx-apply)

## Commands Executed

```text
cargo run --manifest-path app/src-tauri/Cargo.toml --example smoke_edit_session_local
cargo test --manifest-path app/src-tauri/Cargo.toml --lib
```

## Harness: `smoke_edit_session_local`

Salida (resumen):

- OK preferencias get/set round-trip (SQLite in-memory)
- OK fake SFTP download/upload round-trip (temp local)
- OK rechazo por tamaño >10 MiB
- OK heurística binaria
- OK edit session reuse + coalesce confirm
- OK cleanup temps locales
- `=== PASS (sin mutaciones al lab SSH) ===`

## Commands de producto cubiertos (vía mock/helpers)

| Área | Evidencia |
|------|-----------|
| Preferencias editor | `get/set_preferred_external_editor` + schema `app_preferences` |
| Download/upload | `FakeSftpStore` (equivalente local a `sftp_download_file` / `sftp_upload_file`) |
| Edit lifecycle | `EditSessionRegistry` reuse / confirm / dismiss |
| Errores tamaño/path | rechazo download >10 MiB; probe binario |

## PTY / transfer model

- El modelo de transfer real reutiliza pump PTY no bloqueante (mismo patrón que `sftp_list_dir`); unit/harness **no** abre Session live.
- PTY live tras transfer: **N/A** en esta verificación (sin SSH live); no se dejó transport inutilizable porque no hubo conexión.

## Cleanup

- Directorio temp del harness eliminado al final (`remove_dir_all`).

## Lab SSH: cero mutaciones

- **Declaración explícita:** esta verificación **no** conectó al host SSH de pruebas compartido y **no** realizó upload, replace ni delete remoto.
- Mockeado: `FakeSftpStore` + SQLite in-memory + registry en proceso.
- Write remoto: **N/A**.
- Sandbox remoto desechable: **no provisionado por el usuario**; no se ejecutaron uploads live.

## Outcome

- Desktop-commands (mock/local): **PASS**
