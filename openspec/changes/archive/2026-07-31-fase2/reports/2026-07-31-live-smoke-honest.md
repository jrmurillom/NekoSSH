# Smoke SSH live — 2026-07-31 (honest)

Host: profile id=1 from local nekossh.db (`root@161.97.142.252`). Secrets not logged.

## Results

| Test | Stack | Result |
|------|--------|--------|
| TCP banner | socket | PASS |
| PTY + SFTP same transport + burst typing | paramiko | PASS |
| Single Session PTY + SFTP non-blocking + pump | ssh2 (app stack) | PASS `[A]` |
| 2º TCP login while PTY open | ssh2 | VPS tolera 2 logins (`[B]` no kill) |
| Threaded PTY reader + SFTP + typing | ssh2 like app | PASS `[C]` |

## Root cause found in ssh2 non-blocking

`session.sftp()` / `readdir` return `Session(-37) Would block …` unless the PTY channel is pumped between retries. Sleep-only retries are not enough.

## App fixes applied

1. Una Session (PTY + SFTP channels)
2. Never `set_blocking(true)`
3. SFTP retries + PTY pump + re-emit stdout
4. Release mutex while sleeping / writing WouldBlock
5. Apply SSH keepalive from profile (antes se ignoraba)

## User action

Rebuild/restart Tauri app. Yellow close line should include reason (`EOF…` / `error de lectura…`).
