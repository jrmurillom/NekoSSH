# Auditoría SSH/PTY/SFTP — 2026-07-31 (honesta)

## Alcance

Revisión línea a línea de:
- `app/src-tauri/src/lib.rs` (ciclo de vida SSH)
- `app/src/main.ts` (xterm → invoke, listeners, explorador)

## Causas raíz (con evidencia)

### RC1 — `transport read` al tipear (principal)

**Síntoma usuario:** `[Conexión Cerrada…] (error de lectura PTY: transport read)` tras teclear.

**Evidencia (`smoke_isolate_burst`, `smoke_repro_transport`):**
| Patrón | Resultado |
|--------|-----------|
| write de string completa | PASS estable |
| write 1 byte + `flush()` por tecla + lector concurrente | FAIL `transport read` (reproducible) |
| write 1 byte sin flush-por-tecla + flush al final | PASS |

**Causa:** `write_ssh_input` hacía `channel.flush()` en **cada** `onData` (1 tecla = 1 invoke = 1 flush) mientras el hilo PTY lee la misma `Session` no bloqueante (libssh2/WinCNG). Eso tumba el transport.

**Archivos:**
- Antes: `lib.rs` `write_ssh_input` → flush incondicional
- Antes: `main.ts` `term.onData` → invoke inmediato por tecla

### RC2 — timeouts dejados en sesión interactiva

**Evidencia / diseño:** se había puesto `TcpStream::set_read_timeout(20s)` + `sess.set_timeout(20_000)` y **no se apagaban** tras auth. En Windows eso se reporta como error de socket → libssh2 `"transport read"`.

**Fix coherente:** `connect_timeout` solo para conectar; `set_read_timeout(None)`; tras auth `set_timeout(0)`.

### RC3 — `Session(-8) Unable to exchange encryption keys`

**Evidencia:** error en UI; smoke ssh2 a veces OK. WinCNG + KEX `diffie-hellman-group-exchange-*` es causa conocida (issues libssh2/ssh2-rs).

**Mitigación:** `method_pref` (AES-CTR primero, KEX moderno sin dh-gex preferido) + reintento ×3 solo en KEX fail.

### RC4 — SFTP disparado al cambiar de terminal

**Evidencia de código:** `switchActiveTerminal` llamaba siempre `refreshExplorerForActiveTerminal()` → `sftp_list_dir` sobre la **misma** Session del PTY, aunque Archivos no estuviera visible. Compite con tipeo.

**Fix:** SFTP solo si `#panel-files` está `.active`.

### RC5 (secundario) — SFTP non-blocking sin bombear PTY

**Evidencia:** `session.sftp()` / `readdir` → `Session(-37) Would block` sin retries+pump. Ya mitigado con reintentos + pump + reemit stdout.

## Cambios aplicados (justificados)

1. **Flush selectivo** en `write_ssh_input`: solo si hay `\n`/`\r` o `len > 1` (paste).
2. **Coalesce 16ms** en `term.onData` (`main.ts`).
3. **Timeouts**: connect_timeout; sin SO_RCVTIMEO; `set_timeout(0)` post-auth.
4. **KEX prefs + retry** para Session(-8).
5. **Explorador**: no SFTP en switch de terminal salvo pestaña Archivos.
6. **Keepalive**: enviar periódico; **no** matar sesión por error soft de keepalive (revert half-patch malo).
7. **SFTP**: misma Session, sin `set_blocking(true)`, retries+pump.

## Qué NO está “cerrado al 100%”

- Sigue habiendo flaky residual si se fuerza burst+flush_each (el anti-patrón). Con política nueva: 8/8 PASS.
- OpenSSL vendored no compiló en este entorno (perl roto); se sigue en WinCNG.
- No se probó la UI Tauri embebida end-to-end (solo ejemplos ssh2 + unit tests). Usuario debe rebuild.

## Matriz de pruebas (esta corrida)

| Prueba | Resultado |
|--------|-----------|
| `cargo test` (14) | PASS |
| `smoke_coalesce_verify` 8× tipeo + idle 25s | **ALL PASS** |
| `smoke_isolate_burst` flush_each+threaded | FAIL (confirma RC1) |
| `smoke_isolate_burst` flush_end+threaded | PASS |
| `smoke_verify_fix` (política flush, sin coalesce) | 4/5 tipeo + idle + SFTP + reconnect PASS; 1 FAIL flaky |
| `smoke_pty_sftp` A/C | PASS (corrida previa en sesión) |
| `smoke_audit_matrix` (burst antiguo) | FAIL transport read (pre-fix path) |

## Qué debe hacer el usuario

1. Cerrar NekoSSH por completo.
2. Rebuild / `tauri dev` con este código.
3. Conectar, tipear una línea completa, idle >25s, abrir Archivos y listar.
4. Si cae, copiar la línea amarilla completa (incluye motivo).
