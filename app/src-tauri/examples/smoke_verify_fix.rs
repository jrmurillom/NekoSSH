//! Verify fix: threaded reader + per-key write WITHOUT flush-each (app-like post-fix).
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
    // Prefer AES-CTR first (chacha has been flaky with some WinCNG paths).
    let _ = sess.method_pref(
        ssh2::MethodType::CryptCs,
        "aes256-ctr,aes192-ctr,aes128-ctr,chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com",
    );
    let _ = sess.method_pref(
        ssh2::MethodType::CryptSc,
        "aes256-ctr,aes192-ctr,aes128-ctr,chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com",
    );
    let _ = sess.method_pref(
        ssh2::MethodType::Kex,
        "curve25519-sha256,curve25519-sha256@libssh.org,ecdh-sha2-nistp256,ecdh-sha2-nistp384,ecdh-sha2-nistp521,diffie-hellman-group14-sha256,diffie-hellman-group14-sha1",
    );
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.set_keepalive(true, 30);
    sess.userauth_password(user, pass).unwrap();
    sess.set_timeout(0);
    sess
}

/// Mirrors write_ssh_input post-fix policy.
fn write_like_app(ch: &mut ssh2::Channel, data: &[u8]) {
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
    println!("verify flush policy → {user}@{host}:{port}");
    let mut fails = 0;
    for round in 1..=5 {
        print!("  round {round} typed line ... ");
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
        thread::sleep(Duration::from_millis(500));
        // Type like xterm onData: one char per call; flush only on \n
        for b in b"echo VERIFY_OK\n" {
            {
                let mut g = live.lock().unwrap();
                write_like_app(&mut g.ch, &[*b]);
            }
            thread::sleep(Duration::from_millis(20));
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut ok = false;
        while Instant::now() < deadline {
            if seen.lock().unwrap().contains("VERIFY_OK") {
                ok = true;
                break;
            }
            if live.lock().unwrap().ch.eof() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        *stop.lock().unwrap() = true;
        let _ = reader.join();
        if ok {
            println!("PASS");
        } else {
            fails += 1;
            println!("FAIL out={:?}", seen.lock().unwrap());
        }
        drop(live);
        drop(sess);
        thread::sleep(Duration::from_millis(300));
    }

    // Idle 25s then type (no bad timeouts)
    print!("  idle 25s + type ... ");
    let sess = connect(&host, port, &user, &pass);
    let mut ch = sess.channel_session().unwrap();
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);
    sess.set_timeout(0);
    let t0 = Instant::now();
    let mut buf = [0u8; 4096];
    let mut idle_err = None;
    while t0.elapsed() < Duration::from_secs(25) {
        let _ = sess.keepalive_send();
        match ch.read(&mut buf) {
            Ok(0) => {
                idle_err = Some("EOF".into());
                break;
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                idle_err = Some(e.to_string());
                break;
            }
        }
    }
    if let Some(e) = idle_err {
        println!("FAIL idle {e}");
        fails += 1;
    } else {
        write_like_app(&mut ch, b"D");
        thread::sleep(Duration::from_millis(200));
        match ch.read(&mut buf) {
            Err(e) if e.kind() != std::io::ErrorKind::WouldBlock => {
                println!("FAIL after type {e}");
                fails += 1;
            }
            Ok(0) => {
                println!("FAIL EOF after type");
                fails += 1;
            }
            _ => println!("PASS"),
        }
    }

    // SFTP + type
    print!("  SFTP then type ... ");
    let sess = connect(&host, port, &user, &pass);
    let mut ch = sess.channel_session().unwrap();
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);
    sess.set_timeout(0);
    thread::sleep(Duration::from_millis(400));
    let mut attempts = 0;
    let sftp = loop {
        let _ = ch.read(&mut buf);
        match sess.sftp() {
            Ok(s) => break s,
            Err(e)
                if attempts < 200
                    && e.to_string().to_lowercase().contains("would block") =>
            {
                attempts += 1;
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("{e}"),
        }
    };
    attempts = 0;
    let entries = loop {
        let _ = ch.read(&mut buf);
        match sftp.readdir(std::path::Path::new("/")) {
            Ok(e) => break e,
            Err(e)
                if attempts < 200
                    && e.to_string().to_lowercase().contains("would block") =>
            {
                attempts += 1;
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => panic!("{e}"),
        }
    };
    drop(sftp);
    for b in b"echo AFTER_SFTP\n" {
        write_like_app(&mut ch, &[*b]);
        thread::sleep(Duration::from_millis(15));
    }
    let mut out = String::new();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        match ch.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => out.push_str(&String::from_utf8_lossy(&buf[..n])),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                println!("FAIL {e} entries={}", entries.len());
                fails += 1;
                out.clear();
                break;
            }
        }
        if out.contains("AFTER_SFTP") {
            break;
        }
    }
    if out.contains("AFTER_SFTP") {
        println!("PASS ({} entries)", entries.len());
    } else if !out.is_empty() || fails == 0 {
        // only count if we didn't already fail
        if !out.contains("AFTER_SFTP") {
            println!("FAIL out={out:?}");
            fails += 1;
        }
    }

    // reconnect
    print!("  reconnect ... ");
    drop(ch);
    drop(sess);
    let sess = connect(&host, port, &user, &pass);
    let mut ch = sess.channel_session().unwrap();
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);
    sess.set_timeout(0);
    thread::sleep(Duration::from_millis(400));
    write_like_app(&mut ch, b"echo RECONNECT_OK\n");
    out = String::new();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        match ch.read(&mut buf) {
            Ok(n) if n > 0 => out.push_str(&String::from_utf8_lossy(&buf[..n])),
            Ok(0) => break,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                println!("FAIL {e}");
                fails += 1;
                break;
            }
            _ => {}
        }
        if out.contains("RECONNECT_OK") {
            break;
        }
    }
    if out.contains("RECONNECT_OK") {
        println!("PASS");
    } else {
        println!("FAIL out={out:?}");
        fails += 1;
    }

    if fails == 0 {
        println!("VERIFY: ALL PASS");
        std::process::exit(0);
    } else {
        println!("VERIFY: {fails} FAIL(S)");
        std::process::exit(1);
    }
}
