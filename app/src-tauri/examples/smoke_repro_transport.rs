//! Minimal repro: auth variants × write styles → catch transport read.
use rusqlite::Connection;
use ssh2::{MethodType, Session};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::thread;
use std::time::{Duration, Instant};

fn creds() -> (String, u16, String, String) {
    let db = format!(
        r"{}\com.nekossh.app\nekossh.db",
        std::env::var("APPDATA").unwrap()
    );
    let c = Connection::open(db).unwrap();
    c.query_row(
        "SELECT p.host,p.port,p.username,c.password FROM profiles p
         LEFT JOIN auth_credentials c ON c.profile_id=p.id WHERE p.id=1",
        [],
        |r| {
            Ok((
                r.get(0)?,
                r.get::<_, i64>(1)? as u16,
                r.get(2)?,
                r.get::<_, Option<String>>(3)?.unwrap_or_default(),
            ))
        },
    )
    .unwrap()
}

fn auth(host: &str, port: u16, user: &str, pass: &str, with_prefs: bool) -> Session {
    let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();
    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(15)).unwrap();
    let _ = tcp.set_nodelay(true);
    let _ = tcp.set_read_timeout(None);
    let _ = tcp.set_write_timeout(None);
    let mut sess = Session::new().unwrap();
    sess.set_timeout(20_000);
    if with_prefs {
        let _ = sess.method_pref(
            MethodType::Kex,
            "curve25519-sha256,curve25519-sha256@libssh.org,ecdh-sha2-nistp256,diffie-hellman-group14-sha256,diffie-hellman-group14-sha1",
        );
        let _ = sess.method_pref(
            MethodType::CryptCs,
            "aes256-ctr,aes192-ctr,aes128-ctr,chacha20-poly1305@openssh.com",
        );
        let _ = sess.method_pref(
            MethodType::CryptSc,
            "aes256-ctr,aes192-ctr,aes128-ctr,chacha20-poly1305@openssh.com",
        );
    }
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.set_keepalive(true, 30);
    sess.userauth_password(user, pass).unwrap();
    sess.set_timeout(0);
    sess
}

fn nb_write(ch: &mut ssh2::Channel, data: &[u8]) {
    let mut off = 0;
    let mut attempts = 0;
    while off < data.len() {
        match ch.write(&data[off..]) {
            Ok(0) => thread::sleep(Duration::from_millis(5)),
            Ok(n) => {
                off += n;
                attempts = 0;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                attempts += 1;
                if attempts > 200 {
                    panic!("write WouldBlock timeout");
                }
                thread::sleep(Duration::from_millis(5));
            }
            Err(e) => panic!("write: {e}"),
        }
    }
    let _ = ch.flush();
}

fn read_until(ch: &mut ssh2::Channel, needle: &str, ms: u64) -> Result<String, String> {
    let mut buf = [0u8; 4096];
    let mut out = String::new();
    let deadline = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < deadline {
        match ch.read(&mut buf) {
            Ok(0) => return Err(format!("EOF; got={out:?}")),
            Ok(n) => {
                out.push_str(&String::from_utf8_lossy(&buf[..n]));
                if out.contains(needle) {
                    return Ok(out);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => return Err(format!("{e}; got={out:?}")),
        }
    }
    Err(format!("timeout; got={out:?}"))
}

fn drain(ch: &mut ssh2::Channel, ms: u64) {
    let mut buf = [0u8; 4096];
    let deadline = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < deadline {
        match ch.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }
}

fn trial(label: &str, with_prefs: bool, burst: bool) {
    let (host, port, user, pass) = creds();
    print!("  {label} ... ");
    let sess = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        auth(&host, port, &user, &pass, with_prefs)
    })) {
        Ok(s) => s,
        Err(_) => {
            println!("FAIL auth panic");
            return;
        }
    };
    let mut ch = sess.channel_session().unwrap();
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);
    sess.set_timeout(0);
    thread::sleep(Duration::from_millis(500));
    drain(&mut ch, 400);

    let payload = b"echo REPRO_OK\n";
    if burst {
        for b in payload {
            nb_write(&mut ch, &[*b]);
            thread::sleep(Duration::from_millis(15));
        }
    } else {
        nb_write(&mut ch, payload);
    }

    match read_until(&mut ch, "REPRO_OK", 2000) {
        Ok(_) => println!("PASS"),
        Err(e) => println!("FAIL {e}"),
    }
}

fn main() {
    println!("repro matrix (3 rounds)");
    for round in 1..=3 {
        println!("round {round}");
        trial("prefs=no  write=whole", false, false);
        trial("prefs=no  write=burst", false, true);
        trial("prefs=yes write=whole", true, false);
        trial("prefs=yes write=burst", true, true);
        // Prefer AES over chacha (WinCNG suspicion)
        thread::sleep(Duration::from_millis(500));
    }
}
