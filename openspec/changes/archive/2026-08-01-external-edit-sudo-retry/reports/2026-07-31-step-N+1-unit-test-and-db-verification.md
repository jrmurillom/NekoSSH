# Step N+1 — Unit tests + DB verification

- Date: 2026-07-31
- Change: `external-edit-sudo-retry`
- Branch: `feature/external-edit-sudo-retry`
- Agent: Cursor Auto (opsx-apply)

## Unit tests

```text
cargo test --manifest-path app/src-tauri/Cargo.toml
```

**Result:** PASS — 44 tests (incluye 9 de `elevated_upload` + denegación fake SFTP).

Cobertura delta relevante:
- Clasificación `permission_denied` elevable vs disconnect/not_found/other
- Builder `sudo -n cp` con quoting + rechazo NUL
- Orquestación mock: éxito, `sudo_password_required`, `sudo_failed`, permission en temp
- Fake SFTP: deny por prefijo, copy/unlink

Frontend: `npm run build --prefix app` → PASS (`tsc && vite build`).

## DB / SQLite

**N/A** — este change no añade migración ni muta esquema SQLite. El path elevado no toca `app_preferences` ni perfiles.

## Lab SSH: cero mutaciones

- **Declaración explícita:** cero uploads/exec/delete contra el host SSH de pruebas compartido.
- Qué se mockeó: `FakeSftpStore`, closures de exec in-process, temps bajo `%TEMP%` local.
- Write remoto live: **N/A**.
- Sandbox disposable: **no provisionado**; no se ejecutó sudo/upload live.

## Outcome

- N+1 verification: **PASS**
