# Desktop Commands Verification

- Date: 2026-07-31
- Change: `external-edit-sudo-retry`
- Branch: `feature/external-edit-sudo-retry`
- Agent: Cursor Auto (opsx-apply)

## Postura

Harness **mock/fake only**. No se invocó SSH live ni se escribió en el lab compartido.

## Comando ejecutado

```text
cargo run --manifest-path app/src-tauri/Cargo.toml --example smoke_elevated_upload_local
```

**Result:** PASS

### Casos cubiertos (6.1)

| Caso | Resultado |
|------|-----------|
| Clasificación permission → elevable | PASS |
| Clasificación disconnect → no elevable | PASS |
| Builder `sudo -n cp` + rechazo NUL | PASS |
| Fake upload denegado a `/etc` | PASS |
| Path elevado mock éxito + cleanup temp remoto en store | PASS |
| `sudo_password_required` | PASS |
| `sudo_failed` | PASS |
| Un solo intento elevado por aceptación | PASS |

### Aislamiento / fase (6.2)

| Caso | Resultado |
|------|-----------|
| Tras `fail_upload` fase vuelve a `Watching` | PASS |
| Temp local dirty conservado | PASS |
| Cleanup temps locales del harness | PASS |
| Modelo PTY/Session roto | **N/A** (sin Session SSH real; exec es closure mock) |

También se re-ejecutó `smoke_edit_session_local` → PASS (base fase3 intacta).

Commands Tauri registrados en `lib.rs`:
- `confirm_edit_upload` → error estructurado `EditUploadError`
- `edit_session_upload_with_sudo` → path elevado

## Lab SSH: cero mutaciones

- **Declaración explícita:** cero writes/exec/delete al host SSH de pruebas compartido.
- Qué se mockeó: `FakeSftpStore` + exec in-process en `run_elevated_upload`.
- Write remoto live: **N/A**.
- Sandbox disposable: **no provisionado**; no ejecutar uploads/sudo live contra el lab hasta que exista.

## Outcome

- Desktop-commands verification (mock/local): **PASS**
