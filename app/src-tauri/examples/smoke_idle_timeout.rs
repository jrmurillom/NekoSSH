//! Reproduce: connect, idle >25s, type. Fails if TCP/session timeout kills transport.
use rusqlite::Connection;
use ssh2::Session;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

fn creds() -> (String, u16, String, String) {
    let appdata = std::env::var("APPDATA").unwrap();
    let db = format!(r"{appdata}\com.roberto.app\nekossh.db");
    let conn = Connection::open(db).unwrap();
    conn.query_row(
        "SELECT p.host, p.port, p.username, c.password FROM profiles p
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

fn main() {
    let bad_timeouts = std::env::args().any(|a| a == "--bad-timeouts");
    let (host, port, user, pass) = creds();
    println!("idle-smoke → {user}@{host}:{port} bad_timeouts={bad_timeouts}");

    let tcp = TcpStream::connect(format!("{host}:{port}")).unwrap();
    let _ = tcp.set_nodelay(true);
    if bad_timeouts {
        let _ = tcp.set_read_timeout(Some(Duration::from_secs(20)));
        let _ = tcp.set_write_timeout(Some(Duration::from_secs(20)));
    }

    let mut sess = Session::new().unwrap();
    if bad_timeouts {
        sess.set_timeout(20_000);
    }
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.set_keepalive(true, 15);
    sess.userauth_password(&user, &pass).unwrap();
    let mut ch = sess.channel_session().unwrap();
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);
    // After connect, interactive sessions must NOT keep a hard socket/session timeout.
    if !bad_timeouts {
        sess.set_timeout(0);
    }

    let t0 = Instant::now();
    let mut buf = [0u8; 4096];
    let idle_for = Duration::from_secs(25);
    println!("idling {idle_for:?} while pumping PTY...");
    while t0.elapsed() < idle_for {
        match ch.read(&mut buf) {
            Ok(0) => {
                println!("FAIL: EOF during idle at {:?}", t0.elapsed());
                std::process::exit(2);
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                let _ = sess.keepalive_send();
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                println!("FAIL: read during idle: {e} at {:?}", t0.elapsed());
                std::process::exit(3);
            }
        }
    }
    println!("idle OK; typing 'D'");
    ch.write_all(b"D").unwrap();
    let _ = ch.flush();
    thread::sleep(Duration::from_millis(300));
    match ch.read(&mut buf) {
        Ok(0) => {
            println!("FAIL: EOF after type");
            std::process::exit(4);
        }
        Err(e) if e.kind() != std::io::ErrorKind::WouldBlock => {
            println!("FAIL: read after type: {e}");
            std::process::exit(5);
        }
        _ => println!("PASS: still alive after 25s idle + type"),
    }
}
