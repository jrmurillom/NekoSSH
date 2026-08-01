# Desktop Commands Verification (fix v2)

- Date: 2026-07-30
- Change: fase2
- Agent: Cursor Auto (opsx-apply)

## Surface
- Commands registrados en `app/src-tauri/src/lib.rs`: `sftp_list_dir`, `ssh_cd`, `write_ssh_input`, `get_remote_cwd` (legado; **no** usado para sync Explorer←Terminal), `close_ssh_session`.
- Evento nuevo: `ssh-cwd` (path desde OSC 7 en el hilo de lectura PTY).
- Estado: una sola `LiveSsh` (Session + channel PTY); SFTP vía `session.sftp()` sobre la misma Session. Sin `SftpSessions` / 2º login.

## Checks ejecutados
| Check | Resultado |
|-------|-----------|
| Compilación + unit tests (incl. `osc7`) | PASS (13) |
| `ssh_cd` escribe `cd` al PTY y devuelve path | Código verificado |
| `sftp_list_dir` usa Session compartida + `set_blocking` acotado | Código verificado |
| Sync ya no invoca `get_remote_cwd` desde frontend | Grep: solo queda el command Rust, sin callers TS |

## Live SSH IPC
- No hay host/credenciales en el entorno del agente para invoke real contra un servidor.
- Cobertura de garantía: parser OSC 7 unitario + revisión estática del wiring IPC.

## Outcome
- Status: PASS con limitación documentada (sin smoke SSH remoto en CI/agente)
- Blocking issues: none for code path; live smoke pendiente de entorno con servidor
