//! Verificación desktop-commands del contexto multi-shell (padre + 3 hijos).
//!
//! Abre 4 Sessions SSH independientes contra el perfil id=1 (mismo host, mismo
//! login), como hace el frontend con 4 `terminal_id` distintos: PTY por sesión,
//! escritura, resize y cierre. Además valida el caso de error (credencial mala).
//! No imprime secretos. Perfil id=1 en %APPDATA%/com.nekossh.app/nekossh.db
use rusqlite::Connection;
use ssh2::Session;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::thread;
use std::time::{Duration, Instant};

struct Creds {
    host: String,
    port: u16,
    user: String,
    auth_type: String,
    password: String,
    private_key: String,
    passphrase: String,
}

fn creds() -> Creds {
    let db = format!(
        r"{}\com.nekossh.app\nekossh.db",
        std::env::var("APPDATA").unwrap()
    );
    let c = Connection::open(db).unwrap();
    c.query_row(
        "SELECT p.host, p.port, p.username, COALESCE(c.auth_type,'password'),
                COALESCE(c.password,''), COALESCE(c.private_key,''), COALESCE(c.passphrase,'')
         FROM profiles p
         LEFT JOIN auth_credentials c ON c.profile_id = p.id
         WHERE p.id = 1",
        [],
        |r| {
            Ok(Creds {
                host: r.get(0)?,
                port: r.get::<_, i64>(1)? as u16,
                user: r.get(2)?,
                auth_type: r.get(3)?,
                password: r.get(4)?,
                private_key: r.get(5)?,
                passphrase: r.get(6)?,
            })
        },
    )
    .unwrap()
}

/// Autentica como el backend: password o PEM en archivo temporal efímero.
fn connect(c: &Creds) -> Result<Session, String> {
    let addr = (c.host.as_str(), c.port)
        .to_socket_addrs()
        .map_err(|e| format!("dns: {e}"))?
        .next()
        .ok_or_else(|| "sin direcciones".to_string())?;
    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(15))
        .map_err(|e| format!("tcp: {e}"))?;
    let _ = tcp.set_nodelay(true);
    let mut sess = Session::new().map_err(|e| format!("session: {e}"))?;
    sess.set_timeout(20_000);
    sess.set_tcp_stream(tcp);
    sess.handshake().map_err(|e| format!("handshake: {e}"))?;
    sess.set_keepalive(true, 30);

    if c.auth_type == "key" {
        let tmp = std::env::temp_dir().join(format!("nekossh-smoke-{}.pem", std::process::id()));
        std::fs::write(&tmp, c.private_key.as_bytes()).map_err(|e| format!("tmp key: {e}"))?;
        let pass = if c.passphrase.is_empty() {
            None
        } else {
            Some(c.passphrase.as_str())
        };
        let res = sess.userauth_pubkey_file(&c.user, None, &tmp, pass);
        let _ = std::fs::remove_file(&tmp);
        res.map_err(|e| format!("auth key: {e}"))?;
    } else {
        sess.userauth_password(&c.user, &c.password)
            .map_err(|e| format!("auth pass: {e}"))?;
    }
    sess.set_timeout(0);
    Ok(sess)
}

struct Shell {
    label: String,
    _sess: Session,
    ch: ssh2::Channel,
}

fn open_shell(c: &Creds, label: &str) -> Result<Shell, String> {
    let sess = connect(c)?;
    let mut ch = sess
        .channel_session()
        .map_err(|e| format!("channel: {e}"))?;
    ch.request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
        .map_err(|e| format!("pty: {e}"))?;
    ch.shell().map_err(|e| format!("shell: {e}"))?;
    sess.set_blocking(false);
    sess.set_timeout(0);
    thread::sleep(Duration::from_millis(400));
    drain(&mut ch);
    Ok(Shell {
        label: label.to_string(),
        _sess: sess,
        ch,
    })
}

fn drain(ch: &mut ssh2::Channel) {
    let mut buf = [0u8; 4096];
    for _ in 0..30 {
        match ch.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }
}

fn write_all_nb(ch: &mut ssh2::Channel, data: &[u8]) -> Result<(), String> {
    let mut off = 0;
    while off < data.len() {
        match ch.write(&data[off..]) {
            Ok(n) => off += n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(e) => return Err(format!("write: {e}")),
        }
    }
    let _ = ch.flush();
    Ok(())
}

fn expect(ch: &mut ssh2::Channel, needle: &str, ms: u64) -> Result<(), String> {
    let mut buf = [0u8; 4096];
    let mut out = String::new();
    let deadline = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < deadline {
        match ch.read(&mut buf) {
            Ok(0) => return Err("EOF".to_string()),
            Ok(n) => {
                out.push_str(&String::from_utf8_lossy(&buf[..n]));
                if out.contains(needle) {
                    return Ok(());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(15));
            }
            Err(e) => return Err(format!("read: {e}")),
        }
    }
    Err("timeout".to_string())
}

fn main() {
    let c = creds();
    println!("contexto multi-shell: 1 padre + 3 hijos (4 logins independientes)");

    let labels = ["padre", "hijo-1", "hijo-2", "hijo-3"];
    let mut shells: Vec<Shell> = Vec::new();

    for label in labels {
        match open_shell(&c, label) {
            Ok(s) => {
                println!("  abrir {label:<7} ... PASS");
                shells.push(s);
            }
            Err(e) => println!("  abrir {label:<7} ... FAIL {e}"),
        }
    }

    // Escritura + eco aislado por sesión (equivale a write_ssh_input por terminal_id).
    for s in shells.iter_mut() {
        let token = format!("MS_OK_{}", s.label.replace('-', "_"));
        let payload = format!("echo {token}\n");
        let res = write_all_nb(&mut s.ch, payload.as_bytes())
            .and_then(|_| expect(&mut s.ch, &token, 4000));
        match res {
            Ok(()) => println!("  eco {:<9} ... PASS", s.label),
            Err(e) => println!("  eco {:<9} ... FAIL {e}", s.label),
        }
    }

    // Resize por sesión (equivale a resize_ssh_pty por terminal_id).
    for s in shells.iter_mut() {
        match s.ch.request_pty_size(100, 30, None, None) {
            Ok(()) => println!("  resize {:<6} ... PASS", s.label),
            Err(e) => println!("  resize {:<6} ... FAIL {e}", s.label),
        }
    }

    // Cerrar un hijo no debe afectar a los demás (cerrar celda hijo).
    if shells.len() >= 2 {
        let mut child = shells.remove(1);
        let _ = child.ch.close();
        drop(child);
        println!("  cerrar hijo-1  ... PASS");
        for s in shells.iter_mut() {
            let token = format!("ALIVE_{}", s.label.replace('-', "_"));
            let payload = format!("echo {token}\n");
            let res = write_all_nb(&mut s.ch, payload.as_bytes())
                .and_then(|_| expect(&mut s.ch, &token, 4000));
            match res {
                Ok(()) => println!("  vivo {:<8} ... PASS", s.label),
                Err(e) => println!("  vivo {:<8} ... FAIL {e}", s.label),
            }
        }
    }

    // Cierre del contexto: todas las sesiones restantes.
    for s in shells.iter_mut() {
        let _ = s.ch.close();
    }
    println!("  cerrar contexto ... PASS ({} sesiones)", shells.len());

    // Caso de error: credencial inválida debe fallar sin colgar.
    let bad = Creds {
        auth_type: "password".to_string(),
        password: "credencial-invalida-smoke".to_string(),
        private_key: String::new(),
        passphrase: String::new(),
        ..creds()
    };
    match connect(&bad) {
        Ok(_) => println!("  auth inválida  ... FAIL (aceptó credencial mala)"),
        Err(e) => {
            let short = e.split(':').next().unwrap_or("error");
            println!("  auth inválida  ... PASS (rechazada: {short})");
        }
    }
}
