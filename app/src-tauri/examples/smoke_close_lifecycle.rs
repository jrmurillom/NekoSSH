//! Smoke: conectar → close_ssh path (remove+disconnect) → mapa vacío / canal muerto.
//! Uso: cargo run --example smoke_close_lifecycle --manifest-path app/src-tauri/Cargo.toml

use rusqlite::Connection;
use ssh2::Session;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct LiveSsh {
    session: Session,
    channel: ssh2::Channel,
}

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

fn shutdown(live: &mut LiveSsh) {
    let _ = live.channel.close();
    let _ = live
        .session
        .disconnect(None, "NekoSSH session closed", None);
}

fn remove_and_shutdown(
    map: &mut HashMap<String, Arc<Mutex<LiveSsh>>>,
    id: &str,
) -> bool {
    let Some(arc) = map.remove(id) else {
        return false;
    };
    if let Ok(mut live) = arc.lock() {
        shutdown(&mut live);
    }
    true
}

fn main() {
    let (host, port, user, pass) = creds();
    println!("close-lifecycle → {user}@{host}:{port}");

    let tcp = TcpStream::connect(format!("{host}:{port}")).unwrap();
    let _ = tcp.set_nodelay(true);
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password(&user, &pass).unwrap();
    let mut ch = sess.channel_session().unwrap();
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);
    sess.set_timeout(0);

    let mut map: HashMap<String, Arc<Mutex<LiveSsh>>> = HashMap::new();
    map.insert(
        "term-1".into(),
        Arc::new(Mutex::new(LiveSsh {
            session: sess,
            channel: ch,
        })),
    );
    assert_eq!(map.len(), 1);

    // Pump a bit then close like the app
    {
        let arc = map.get("term-1").unwrap().clone();
        let mut live = arc.lock().unwrap();
        let mut buf = [0u8; 1024];
        for _ in 0..20 {
            match live.channel.read(&mut buf) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20));
                }
                Err(e) => panic!("read: {e}"),
            }
        }
        live.channel.write_all(b"echo BEFORE_CLOSE\n").unwrap();
        let _ = live.channel.flush();
    }
    thread::sleep(Duration::from_millis(400));

    assert!(remove_and_shutdown(&mut map, "term-1"));
    assert!(map.is_empty());
    assert!(!remove_and_shutdown(&mut map, "term-1"), "idempotent");

    println!("PASS: session removed + disconnect; map empty; second close idempotent");
}
