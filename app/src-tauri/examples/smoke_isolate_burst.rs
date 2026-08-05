//! Root-cause isolation: burst write ± flush ± interleaved read.
use rusqlite::Connection;
use ssh2::Session;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
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

fn connect(host: &str, port: u16, user: &str, pass: &str) -> Session {
    let addr = (host, port).to_socket_addrs().unwrap().next().unwrap();
    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(15)).unwrap();
    let _ = tcp.set_nodelay(true);
    let _ = tcp.set_read_timeout(None);
    let _ = tcp.set_write_timeout(None);
    let mut sess = Session::new().unwrap();
    sess.set_timeout(20_000);
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.set_keepalive(true, 30);
    sess.userauth_password(user, pass).unwrap();
    sess.set_timeout(0);
    sess
}

fn write_nb(ch: &mut ssh2::Channel, data: &[u8], flush_each: bool) -> Result<(), String> {
    let mut off = 0;
    while off < data.len() {
        match ch.write(&data[off..]) {
            Ok(n) => {
                off += n.max(0);
                if n == 0 {
                    thread::sleep(Duration::from_millis(5));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(e) => return Err(format!("write: {e}")),
        }
        if flush_each {
            let _ = ch.flush();
        }
    }
    if !flush_each {
        let _ = ch.flush();
    }
    Ok(())
}

fn expect(ch: &mut ssh2::Channel, needle: &str, ms: u64) -> Result<(), String> {
    let mut buf = [0u8; 4096];
    let mut out = String::new();
    let deadline = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < deadline {
        match ch.read(&mut buf) {
            Ok(0) => return Err(format!("EOF got={out:?}")),
            Ok(n) => {
                out.push_str(&String::from_utf8_lossy(&buf[..n]));
                if out.contains(needle) {
                    return Ok(());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(15));
            }
            Err(e) => return Err(format!("{e} got={out:?}")),
        }
    }
    Err(format!("timeout got={out:?}"))
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

fn one_shot(label: &str, flush_each: bool, interleave_read: bool) {
    let (host, port, user, pass) = creds();
    print!("  {label} ... ");
    let sess = connect(&host, port, &user, &pass);
    let mut ch = sess.channel_session().unwrap();
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);
    sess.set_timeout(0);
    thread::sleep(Duration::from_millis(400));
    drain(&mut ch);

    let payload = b"echo ISO_OK\n";
    let mut buf = [0u8; 256];
    for b in payload {
        if let Err(e) = write_nb(&mut ch, &[*b], flush_each) {
            println!("FAIL {e}");
            return;
        }
        if interleave_read {
            let _ = ch.read(&mut buf); // ignore WouldBlock
        }
        thread::sleep(Duration::from_millis(15));
    }
    match expect(&mut ch, "ISO_OK", 2500) {
        Ok(()) => println!("PASS"),
        Err(e) => println!("FAIL {e}"),
    }
}

fn threaded_like_app(label: &str, flush_each: bool) {
    let (host, port, user, pass) = creds();
    print!("  {label} ... ");
    struct Live {
        sess: Session,
        ch: ssh2::Channel,
    }
    let sess = connect(&host, port, &user, &pass);
    let mut ch = sess.channel_session().unwrap();
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);
    sess.set_timeout(0);
    let live = Arc::new(Mutex::new(Live { sess, ch }));
    let stop = Arc::new(Mutex::new(false));
    let live_r = live.clone();
    let stop_r = stop.clone();
    let reader = thread::spawn(move || {
        let mut buf = [0u8; 4096];
        let mut out = String::new();
        while !*stop_r.lock().unwrap() {
            let res = {
                let mut g = live_r.lock().unwrap();
                g.ch.read(&mut buf)
            };
            match res {
                Ok(0) => break,
                Ok(n) => out.push_str(&String::from_utf8_lossy(&buf[..n])),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    println!("FAIL reader: {e}");
                    break;
                }
            }
        }
        out
    });
    thread::sleep(Duration::from_millis(500));
    for b in b"echo THR_OK\n" {
        {
            let mut g = live.lock().unwrap();
            let mut off = 0;
            let data = [*b];
            while off < 1 {
                match g.ch.write(&data[off..]) {
                    Ok(n) => off += n.max(0),
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => {
                        println!("FAIL write: {e}");
                        *stop.lock().unwrap() = true;
                        let _ = reader.join();
                        return;
                    }
                }
            }
            if flush_each {
                let _ = g.ch.flush();
            }
        }
        thread::sleep(Duration::from_millis(15));
    }
    if !flush_each {
        let mut g = live.lock().unwrap();
        let _ = g.ch.flush();
    }
    thread::sleep(Duration::from_millis(800));
    let eof = live.lock().unwrap().ch.eof();
    *stop.lock().unwrap() = true;
    let out = reader.join().unwrap_or_default();
    if eof || !out.contains("THR_OK") {
        println!("FAIL eof={eof} out={out:?}");
    } else {
        println!("PASS");
    }
}

fn main() {
    println!("isolation (2 rounds)");
    for r in 1..=2 {
        println!("round {r}");
        one_shot("burst+flush_each+no_read", true, false);
        one_shot("burst+flush_end+no_read", false, false);
        one_shot("burst+flush_each+interleave", true, true);
        one_shot("burst+flush_end+interleave", false, true);
        threaded_like_app("threaded+flush_each (app-like)", true);
        threaded_like_app("threaded+flush_end", false);
        thread::sleep(Duration::from_millis(400));
    }
}
