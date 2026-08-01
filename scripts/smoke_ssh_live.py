"""
Live SSH smoke for NekoSSH (honest).
Reads credentials from local nekossh.db, never prints secrets.
Env override: NEKOSSH_SMOKE_PROFILE_ID
"""
from __future__ import annotations

import os
import socket
import sqlite3
import sys
import time

DB = r"C:\Users\Roberto\AppData\Roaming\com.roberto.app\nekossh.db"


def load_profile(profile_id: int | None):
    conn = sqlite3.connect(DB)
    conn.row_factory = sqlite3.Row
    if profile_id is None:
        row = conn.execute(
            """
            SELECT p.id, p.name, p.host, p.port, p.username, p.keepalive,
                   c.auth_type, c.password, c.key_path, c.passphrase
            FROM profiles p
            LEFT JOIN auth_credentials c ON c.profile_id = p.id
            ORDER BY p.id LIMIT 1
            """
        ).fetchone()
    else:
        row = conn.execute(
            """
            SELECT p.id, p.name, p.host, p.port, p.username, p.keepalive,
                   c.auth_type, c.password, c.key_path, c.passphrase
            FROM profiles p
            LEFT JOIN auth_credentials c ON c.profile_id = p.id
            WHERE p.id = ?
            """,
            (profile_id,),
        ).fetchone()
    if row is None:
        raise SystemExit("No profile found in nekossh.db")
    return row


def smoke_tcp(host: str, port: int) -> None:
    print(f"[1] TCP connect {host}:{port} ...", flush=True)
    t0 = time.time()
    with socket.create_connection((host, port), timeout=10) as s:
        s.settimeout(5)
        # SSH banner
        banner = s.recv(256)
    print(f"    OK in {time.time()-t0:.2f}s banner={banner[:40]!r}")


def smoke_paramiko(row) -> None:
    try:
        import paramiko
    except ImportError:
        print("[2] paramiko not installed — pip install paramiko for PTY+SFTP smoke")
        print("    TCP-only smoke done. Install paramiko to continue.")
        return

    host = row["host"]
    port = int(row["port"])
    user = row["username"]
    auth = row["auth_type"] or "password"
    print(f"[2] SSH auth as {user}@{host}:{port} auth={auth} ...", flush=True)

    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    kwargs = {"hostname": host, "port": port, "username": user, "timeout": 15, "allow_agent": False, "look_for_keys": False}
    if auth == "password":
        kwargs["password"] = row["password"] or ""
    else:
        kwargs["key_filename"] = row["key_path"]
        if row["passphrase"]:
            kwargs["passphrase"] = row["passphrase"]

    client.connect(**kwargs)
    print("    auth OK")

    # Single session: PTY channel
    print("[3] Open PTY shell, type 'echo NEKOSSH_SMOKE' ...", flush=True)
    chan = client.invoke_shell(term="xterm-256color", width=80, height=24)
    chan.settimeout(8)
    time.sleep(0.5)
    while chan.recv_ready():
        chan.recv(4096)
    chan.send("echo NEKOSSH_SMOKE\n")
    time.sleep(0.8)
    out = b""
    while chan.recv_ready():
        out += chan.recv(4096)
    text = out.decode("utf-8", "replace")
    if "NEKOSSH_SMOKE" not in text:
        print("    FAIL: echo not seen in PTY output:")
        print(repr(text[:500]))
        sys.exit(2)
    print("    PTY echo OK")

    # Same TCP client: open SFTP (second channel, not second TCP in paramiko — one transport)
    print("[4] SFTP readdir / on SAME transport ...", flush=True)
    sftp = client.open_sftp()
    names = sftp.listdir("/")
    sftp.close()
    print(f"    SFTP OK ({len(names)} entries)")

    # Keep typing after SFTP
    print("[5] Type after SFTP (cd / && pwd) ...", flush=True)
    while chan.recv_ready():
        chan.recv(4096)
    chan.send("cd / && pwd\n")
    time.sleep(0.8)
    out = b""
    while chan.recv_ready():
        out += chan.recv(4096)
    text = out.decode("utf-8", "replace")
    if "/" not in text:
        print("    WARN: unexpected output:", repr(text[:300]))
    else:
        print("    PTY still alive after SFTP")

    # Burst typing like the user bug (cd /o...)
    print("[6] Burst typing 'cd /tmp' ...", flush=True)
    for ch in "cd /tmp\n":
        chan.send(ch)
        time.sleep(0.02)
    time.sleep(0.8)
    if chan.closed:
        print("    FAIL: channel closed during typing")
        sys.exit(3)
    out = b""
    while chan.recv_ready():
        out += chan.recv(4096)
    print("    channel still open; got", len(out), "bytes")

    chan.close()
    client.close()
    print("PASS: live SSH smoke (paramiko single-connection PTY+SFTP)")


def main() -> None:
    pid = os.environ.get("NEKOSSH_SMOKE_PROFILE_ID")
    profile_id = int(pid) if pid else None
    row = load_profile(profile_id)
    print(
        f"profile id={row['id']} name={row['name']!r} host={row['host']} "
        f"user={row['username']} (secrets not printed)"
    )
    smoke_tcp(row["host"], int(row["port"]))
    smoke_paramiko(row)


if __name__ == "__main__":
    main()
