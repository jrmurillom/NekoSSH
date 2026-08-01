//! Auditoría / smoke matriz (ssh2, mismo stack que la app).
//! Uso:
//!   cargo run --example smoke_audit_matrix
//!   cargo run --example smoke_audit_matrix -- --bad-timeouts   # debe FALLAR idle
//!
//! No imprime secretos. Perfil id=1 en %APPDATA%/com.roberto.app/nekossh.db

use rusqlite::Connection;
use ssh2::{MethodType, Session};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct Creds {
    host: String,
    port: u16,
    user: String,
    password: String,
}

fn load_creds() -> Creds {
    let appdata = std::env::var("APPDATA").expect("APPDATA");
    let db = format!(r"{appdata}\com.roberto.app\nekossh.db");
    let conn = Connection::open(db).expect("open db");
    conn.query_row(
        "SELECT p.host, p.port, p.username, c.password
         FROM profiles p LEFT JOIN auth_credentials c ON c.profile_id = p.id
         WHERE p.id = 1",
        [],
        |row| {
            Ok(Creds {
                host: row.get(0)?,
                port: row.get::<_, i64>(1)? as u16,
                user: row.get(2)?,
                password: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            })
        },
    )
    .expect("profile 1")
}

/// Réplica del diseño coherente de authenticate_session_once (app).
fn auth_good(c: &Creds) -> Session {
    let addr = (c.host.as_str(), c.port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(15)).unwrap();
    let _ = tcp.set_nodelay(true);
    let _ = tcp.set_read_timeout(None);
    let _ = tcp.set_write_timeout(None);

    let mut sess = Session::new().unwrap();
    sess.set_timeout(20_000);
    let _ = sess.method_pref(
        MethodType::Kex,
        "curve25519-sha256,curve25519-sha256@libssh.org,ecdh-sha2-nistp256,ecdh-sha2-nistp384,ecdh-sha2-nistp521,diffie-hellman-group14-sha256,diffie-hellman-group16-sha512,diffie-hellman-group14-sha1",
    );
    let _ = sess.method_pref(
        MethodType::CryptCs,
        "chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com,aes256-ctr,aes192-ctr,aes128-ctr",
    );
    let _ = sess.method_pref(
        MethodType::CryptSc,
        "chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com,aes256-ctr,aes192-ctr,aes128-ctr",
    );
    sess.set_tcp_stream(tcp);
    sess.handshake().expect("handshake");
    sess.set_keepalive(true, 30);
    sess.userauth_password(&c.user, &c.password).expect("auth");
    assert!(sess.authenticated());
    sess.set_timeout(0);
    sess
}

/// Modelo roto: SO_RCVTIMEO + set_timeout(20s) dejados en la sesión interactiva.
fn auth_bad_timeouts(c: &Creds) -> Session {
    let tcp = TcpStream::connect(format!("{}:{}", c.host, c.port)).unwrap();
    let _ = tcp.set_nodelay(true);
    let _ = tcp.set_read_timeout(Some(Duration::from_secs(20)));
    let _ = tcp.set_write_timeout(Some(Duration::from_secs(20)));
    let mut sess = Session::new().unwrap();
    sess.set_timeout(20_000);
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.set_keepalive(true, 30);
    sess.userauth_password(&c.user, &c.password).unwrap();
    // NO set_timeout(0) — bug bajo auditoría
    sess
}

fn pump(ch: &mut ssh2::Channel, ms: u64) -> String {
    let mut buf = [0u8; 4096];
    let mut out = String::new();
    let deadline = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < deadline {
        match ch.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => out.push_str(&String::from_utf8_lossy(&buf[..n])),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => panic!("pump read: {e}"),
        }
    }
    out
}

fn open_shell(sess: &Session) -> ssh2::Channel {
    let mut ch = sess.channel_session().unwrap();
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);
    sess.set_timeout(0);
    thread::sleep(Duration::from_millis(400));
    let _ = pump(&mut ch, 500);
    ch
}

fn sftp_list_with_pump(sess: &Session, ch: &mut ssh2::Channel) -> usize {
    let mut buf = [0u8; 4096];
    let sftp = {
        let mut attempts = 0;
        loop {
            loop {
                match ch.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
            }
            match sess.sftp() {
                Ok(s) => break s,
                Err(e)
                    if attempts < 200
                        && (e.to_string().to_lowercase().contains("would block")
                            || e.code() == ssh2::ErrorCode::Session(-37)) =>
                {
                    attempts += 1;
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("sftp: {e}"),
            }
        }
    };
    let entries = {
        let mut attempts = 0;
        loop {
            loop {
                match ch.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
            }
            match sftp.readdir(Path::new("/")) {
                Ok(e) => break e,
                Err(e)
                    if attempts < 200
                        && (e.to_string().to_lowercase().contains("would block")
                            || e.code() == ssh2::ErrorCode::Session(-37)) =>
                {
                    attempts += 1;
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("readdir: {e}"),
            }
        }
    };
    let n = entries.len();
    drop(sftp);
    n
}

fn main() {
    let bad = std::env::args().any(|a| a == "--bad-timeouts");
    let c = load_creds();
    println!(
        "AUDIT MATRIX → {}@{}:{} bad_timeouts={}",
        c.user, c.host, c.port, bad
    );

    // --- 1: handshake + auth ---
    println!("[1] handshake+auth");
    let sess = if bad {
        auth_bad_timeouts(&c)
    } else {
        auth_good(&c)
    };
    println!("    PASS");

    // --- 2: PTY echo ---
    println!("[2] PTY echo");
    let mut ch = open_shell(&sess);
    ch.write_all(b"echo AUDIT_PTY\n").unwrap();
    let _ = ch.flush();
    let out = pump(&mut ch, 800);
    assert!(out.contains("AUDIT_PTY"), "missing echo: {out:?}");
    println!("    PASS");

    // --- 3: burst typing ---
    println!("[3] burst typing");
    for b in b"cd /tmp && pwd\n" {
        ch.write_all(&[*b]).unwrap();
        thread::sleep(Duration::from_millis(12));
    }
    let _ = ch.flush();
    let out = pump(&mut ch, 800);
    assert!(!ch.eof(), "EOF after burst; {out:?}");
    println!("    PASS (got {} bytes)", out.len());

    // --- 4: SFTP same session ---
    println!("[4] SFTP same session + pump");
    let n = sftp_list_with_pump(&sess, &mut ch);
    assert!(n > 0);
    println!("    PASS ({n} entries)");

    // --- 5: type after SFTP ---
    println!("[5] type after SFTP");
    ch.write_all(b"echo AFTER_SFTP\n").unwrap();
    let out = pump(&mut ch, 800);
    assert!(out.contains("AFTER_SFTP") || !ch.eof(), "{out:?}");
    println!("    PASS");

    let _ = ch.close();
    drop(sess);

    // --- 6: idle 25s then type (transport read repro) ---
    println!("[6] idle 25s then type 'D'");
    let sess = if bad {
        auth_bad_timeouts(&c)
    } else {
        auth_good(&c)
    };
    let mut ch = open_shell(&sess);
    let t0 = Instant::now();
    let mut buf = [0u8; 4096];
    let mut fail: Option<String> = None;
    while t0.elapsed() < Duration::from_secs(25) {
        let _ = sess.keepalive_send();
        match ch.read(&mut buf) {
            Ok(0) => {
                fail = Some(format!("EOF during idle at {:?}", t0.elapsed()));
                break;
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                fail = Some(format!("read during idle: {e} at {:?}", t0.elapsed()));
                break;
            }
        }
    }
    if let Some(msg) = fail {
        if bad {
            println!("    EXPECTED FAIL under --bad-timeouts: {msg}");
            println!("AUDIT: bad-timeouts path confirmed harmful");
            std::process::exit(0);
        }
        panic!("FAIL idle: {msg}");
    }
    ch.write_all(b"D").unwrap();
    let _ = ch.flush();
    thread::sleep(Duration::from_millis(300));
    match ch.read(&mut buf) {
        Ok(0) => panic!("FAIL: EOF after type"),
        Err(e) if e.kind() != std::io::ErrorKind::WouldBlock => {
            panic!("FAIL: read after type: {e}");
        }
        _ => println!("    PASS"),
    }
    let _ = ch.close();
    drop(sess);

    // --- 7: threaded like app ---
    println!("[7] threaded PTY reader + SFTP + typing");
    struct Live {
        session: Session,
        channel: ssh2::Channel,
    }
    let sess = auth_good(&c);
    let mut ch = sess.channel_session().unwrap();
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);
    sess.set_timeout(0);
    let live = Arc::new(Mutex::new(Live {
        session: sess,
        channel: ch,
    }));
    let stop = Arc::new(Mutex::new(false));
    let live_r = live.clone();
    let stop_r = stop.clone();
    let reader = thread::spawn(move || {
        let mut buf = [0u8; 4096];
        while !*stop_r.lock().unwrap() {
            let res = {
                let mut g = live_r.lock().unwrap();
                let _ = g.session.keepalive_send();
                g.channel.read(&mut buf)
            };
            match res {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });
    thread::sleep(Duration::from_millis(500));
    {
        let mut attempts = 0;
        let sftp = loop {
            let open = {
                let mut g = live.lock().unwrap();
                let mut buf = [0u8; 4096];
                loop {
                    match g.channel.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
                }
                g.session.sftp()
            };
            match open {
                Ok(s) => break s,
                Err(e)
                    if attempts < 200
                        && (e.to_string().to_lowercase().contains("would block")
                            || e.code() == ssh2::ErrorCode::Session(-37)) =>
                {
                    attempts += 1;
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("threaded sftp: {e}"),
            }
        };
        let mut attempts = 0;
        let _entries = loop {
            let rd = {
                let mut g = live.lock().unwrap();
                let mut buf = [0u8; 4096];
                loop {
                    match g.channel.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
                }
                sftp.readdir(Path::new("/"))
            };
            match rd {
                Ok(e) => break e,
                Err(e)
                    if attempts < 200
                        && (e.to_string().to_lowercase().contains("would block")
                            || e.code() == ssh2::ErrorCode::Session(-37)) =>
                {
                    attempts += 1;
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => panic!("threaded readdir: {e}"),
            }
        };
        drop(sftp);
    }
    for b in b"echo THREADED\n" {
        {
            let mut g = live.lock().unwrap();
            g.channel.write_all(&[*b]).unwrap();
        }
        thread::sleep(Duration::from_millis(10));
    }
    thread::sleep(Duration::from_millis(600));
    assert!(!live.lock().unwrap().channel.eof());
    *stop.lock().unwrap() = true;
    let _ = reader.join();
    println!("    PASS");

    // --- 8: reconnect ---
    println!("[8] reconnect (drop + new session)");
    let sess1 = auth_good(&c);
    let mut ch1 = open_shell(&sess1);
    ch1.write_all(b"echo FIRST\n").unwrap();
    let _ = pump(&mut ch1, 600);
    let _ = ch1.close();
    drop(sess1);
    thread::sleep(Duration::from_millis(300));
    let sess2 = auth_good(&c);
    let mut ch2 = open_shell(&sess2);
    ch2.write_all(b"echo SECOND\n").unwrap();
    let out = pump(&mut ch2, 800);
    assert!(out.contains("SECOND"), "{out:?}");
    println!("    PASS");

    println!("AUDIT MATRIX: ALL PASS (good path)");
}
