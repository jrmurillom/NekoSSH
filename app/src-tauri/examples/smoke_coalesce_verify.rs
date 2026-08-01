//! Simula coalescing 16ms del frontend + política de flush del backend.
use rusqlite::Connection;
use ssh2::Session;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn creds() -> (String, u16, String, String) {
    let db = format!(
        r"{}\com.roberto.app\nekossh.db",
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
    let _ = sess.method_pref(
        ssh2::MethodType::CryptCs,
        "aes256-ctr,aes192-ctr,aes128-ctr,chacha20-poly1305@openssh.com",
    );
    let _ = sess.method_pref(
        ssh2::MethodType::CryptSc,
        "aes256-ctr,aes192-ctr,aes128-ctr,chacha20-poly1305@openssh.com",
    );
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.set_keepalive(true, 30);
    sess.userauth_password(user, pass).unwrap();
    sess.set_timeout(0);
    sess
}

fn write_policy(ch: &mut ssh2::Channel, data: &[u8]) {
    let mut written = 0;
    while written < data.len() {
        match ch.write(&data[written..]) {
            Ok(n) => written += n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(e) => panic!("write: {e}"),
        }
    }
    let should_flush = data.iter().any(|b| *b == b'\n' || *b == b'\r') || data.len() > 1;
    if should_flush {
        let _ = ch.flush();
    }
}

fn main() {
    let (host, port, user, pass) = creds();
    println!("coalesce+flush policy verify → {user}@{host}");
    let mut fails = 0;

    for round in 1..=8 {
        print!("  round {round} ... ");
        struct Live {
            ch: ssh2::Channel,
        }
        let sess = connect(&host, port, &user, &pass);
        let mut ch = sess.channel_session().unwrap();
        ch.request_pty("xterm-256color", None, None).unwrap();
        ch.shell().unwrap();
        sess.set_blocking(false);
        sess.set_timeout(0);
        let live = Arc::new(Mutex::new(Live { ch }));
        let stop = Arc::new(Mutex::new(false));
        let seen = Arc::new(Mutex::new(String::new()));
        let live_r = live.clone();
        let stop_r = stop.clone();
        let seen_r = seen.clone();
        let reader = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while !*stop_r.lock().unwrap() {
                let res = {
                    let mut g = live_r.lock().unwrap();
                    g.ch.read(&mut buf)
                };
                match res {
                    Ok(0) => break,
                    Ok(n) => seen_r
                        .lock()
                        .unwrap()
                        .push_str(&String::from_utf8_lossy(&buf[..n])),
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        eprintln!("READER_ERR {e}");
                        break;
                    }
                }
            }
        });

        thread::sleep(Duration::from_millis(600));

        // Tecleo humano ~40ms/tecla; coalescing 16ms → batches cortos + flush en \n.
        let line = b"echo COALESCE_OK\n";
        let mut batch = Vec::new();
        let mut last_flush = Instant::now();
        for &b in line {
            batch.push(b);
            thread::sleep(Duration::from_millis(40));
            if last_flush.elapsed() >= Duration::from_millis(16) || b == b'\n' {
                {
                    let mut g = live.lock().unwrap();
                    write_policy(&mut g.ch, &batch);
                }
                batch.clear();
                last_flush = Instant::now();
            }
        }
        if !batch.is_empty() {
            let mut g = live.lock().unwrap();
            write_policy(&mut g.ch, &batch);
        }

        let deadline = Instant::now() + Duration::from_secs(4);
        let mut ok = false;
        while Instant::now() < deadline {
            if seen.lock().unwrap().contains("COALESCE_OK") {
                ok = true;
                break;
            }
            if live.lock().unwrap().ch.eof() {
                break;
            }
            thread::sleep(Duration::from_millis(40));
        }
        *stop.lock().unwrap() = true;
        let _ = reader.join();
        if ok {
            println!("PASS");
        } else {
            fails += 1;
            let s = seen.lock().unwrap().clone();
            let tail: String = s.chars().rev().take(100).collect::<String>().chars().rev().collect();
            println!("FAIL tail={tail:?}");
        }
        drop(live);
        drop(sess);
        thread::sleep(Duration::from_millis(250));
    }

    print!("  idle25 ... ");
    let sess = connect(&host, port, &user, &pass);
    let mut ch = sess.channel_session().unwrap();
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);
    sess.set_timeout(0);
    let t0 = Instant::now();
    let mut buf = [0u8; 4096];
    let mut err = None;
    while t0.elapsed() < Duration::from_secs(25) {
        let _ = sess.keepalive_send();
        match ch.read(&mut buf) {
            Ok(0) => {
                err = Some("EOF".into());
                break;
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                err = Some(e.to_string());
                break;
            }
        }
    }
    if let Some(e) = err {
        println!("FAIL {e}");
        fails += 1;
    } else {
        write_policy(&mut ch, b"D");
        thread::sleep(Duration::from_millis(200));
        match ch.read(&mut buf) {
            Err(e) if e.kind() != std::io::ErrorKind::WouldBlock => {
                println!("FAIL {e}");
                fails += 1;
            }
            Ok(0) => {
                println!("FAIL EOF");
                fails += 1;
            }
            _ => println!("PASS"),
        }
    }

    if fails == 0 {
        println!("ALL PASS");
        std::process::exit(0);
    }
    println!("{fails} FAIL(S)");
    std::process::exit(1);
}
