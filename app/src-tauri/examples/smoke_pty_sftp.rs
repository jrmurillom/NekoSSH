//! Smoke live con ssh2 (mismo stack que NekoSSH).
//! Uso: cargo run --example smoke_pty_sftp --manifest-path app/src-tauri/Cargo.toml
//! Lee perfil id=1 de %APPDATA%/com.nekossh.app/nekossh.db (no imprime secretos).

use rusqlite::Connection;
use ssh2::Session;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use std::time::Duration;

fn appdata_db() -> String {
    let appdata = std::env::var("APPDATA").expect("APPDATA");
    format!(r"{appdata}\com.nekossh.app\nekossh.db")
}

struct Creds {
    host: String,
    port: u16,
    user: String,
    password: String,
}

fn load_creds() -> Creds {
    let conn = Connection::open(appdata_db()).expect("open db");
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

fn auth(c: &Creds) -> Session {
    let tcp = TcpStream::connect(format!("{}:{}", c.host, c.port)).expect("tcp");
    let _ = tcp.set_nodelay(true);
    let mut sess = Session::new().unwrap();
    sess.set_timeout(20_000);
    let _ = sess.method_pref(
        ssh2::MethodType::Kex,
        "curve25519-sha256,curve25519-sha256@libssh.org,ecdh-sha2-nistp256,ecdh-sha2-nistp384,ecdh-sha2-nistp521,diffie-hellman-group14-sha256,diffie-hellman-group16-sha512,diffie-hellman-group14-sha1",
    );
    let _ = sess.method_pref(
        ssh2::MethodType::CryptCs,
        "chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com,aes256-ctr,aes192-ctr,aes128-ctr",
    );
    let _ = sess.method_pref(
        ssh2::MethodType::CryptSc,
        "chacha20-poly1305@openssh.com,aes256-gcm@openssh.com,aes128-gcm@openssh.com,aes256-ctr,aes192-ctr,aes128-ctr",
    );
    sess.set_tcp_stream(tcp);
    sess.handshake().expect("handshake");
    sess.set_keepalive(true, 30);
    sess.userauth_password(&c.user, &c.password)
        .expect("auth");
    assert!(sess.authenticated());
    sess
}

fn drain(ch: &mut ssh2::Channel) -> String {
    let mut buf = [0u8; 4096];
    let mut out = String::new();
    for _ in 0..50 {
        match ch.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => out.push_str(&String::from_utf8_lossy(&buf[..n])),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(e) => panic!("read: {e}"),
        }
    }
    out
}

fn main() {
    let c = load_creds();
    println!(
        "smoke ssh2 → {}@{}:{} (password redacted)",
        c.user, c.host, c.port
    );

    // --- A: single session PTY + SFTP (app model) ---
    println!("[A] single Session: PTY + SFTP channels");
    let sess = auth(&c);
    let mut ch = sess.channel_session().expect("channel");
    ch.request_pty("xterm-256color", None, None).expect("pty");
    ch.shell().expect("shell");
    sess.set_blocking(false);
    thread::sleep(Duration::from_millis(400));
    let _ = drain(&mut ch);

    ch.write_all(b"echo NEKOSSH_SSH2\n").expect("write");
    let _ = ch.flush();
    thread::sleep(Duration::from_millis(600));
    let out = drain(&mut ch);
    assert!(
        out.contains("NEKOSSH_SSH2"),
        "PTY echo missing: {out:?}"
    );
    println!("  PTY echo OK");

    // SFTP without set_blocking(true) — must pump PTY while waiting
    let sftp = {
        let mut attempts = 0;
        let mut buf = [0u8; 4096];
        loop {
            loop {
                match ch.read(&mut buf) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => break,
                }
            }
            match sess.sftp() {
                Ok(s) => break s,
                Err(e) => {
                    let msg = e.to_string().to_lowercase();
                    if attempts < 200 && (msg.contains("would block") || e.code() == ssh2::ErrorCode::Session(-37))
                    {
                        attempts += 1;
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    panic!("sftp open after {attempts}: {e}");
                }
            }
        }
    };
    let entries = {
        let mut attempts = 0;
        let mut buf = [0u8; 4096];
        loop {
            loop {
                match ch.read(&mut buf) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => break,
                }
            }
            match sftp.readdir(Path::new("/")) {
                Ok(e) => break e,
                Err(e) => {
                    let msg = e.to_string().to_lowercase();
                    if attempts < 200
                        && (msg.contains("would block") || e.code() == ssh2::ErrorCode::Session(-37))
                    {
                        attempts += 1;
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    panic!("readdir after {attempts}: {e}");
                }
            }
        }
    };
    println!("  SFTP OK ({} entries)", entries.len());
    drop(sftp);

    for b in b"cd /tmp\n" {
        ch.write_all(&[*b]).expect("burst");
        thread::sleep(Duration::from_millis(15));
    }
    let _ = ch.flush();
    thread::sleep(Duration::from_millis(500));
    let out = drain(&mut ch);
    assert!(!ch.eof(), "PTY EOF after typing+SFTP; out={out:?}");
    println!("  PTY alive after SFTP+typing OK");
    let _ = ch.close();
    drop(sess);
    println!("[A] PASS");

    // --- B: second TCP login while first PTY open (old broken model) ---
    println!("[B] second TCP login while PTY open (expect possible kill)");
    let sess1 = auth(&c);
    let mut ch1 = sess1.channel_session().expect("ch1");
    ch1.request_pty("xterm-256color", None, None).expect("pty1");
    ch1.shell().expect("shell1");
    sess1.set_blocking(false);
    thread::sleep(Duration::from_millis(300));
    let _ = drain(&mut ch1);

    let dual_ok = match std::panic::catch_unwind(|| {
        let sess2 = auth(&c);
        let sftp2 = sess2.sftp().expect("sftp2");
        let _ = sftp2.readdir(Path::new("/")).expect("readdir2");
        drop(sftp2);
        drop(sess2);
    }) {
        Ok(()) => true,
        Err(_) => false,
    };

    thread::sleep(Duration::from_millis(400));
    ch1.write_all(b"echo AFTER_SECOND_LOGIN\n").ok();
    let _ = ch1.flush();
    thread::sleep(Duration::from_millis(600));
    let out = drain(&mut ch1);
    let pty_dead = ch1.eof() || !out.contains("AFTER_SECOND_LOGIN");
    println!(
        "  second_login_ok={dual_ok} first_pty_dead={pty_dead} out_snip={:?}",
        out.chars().take(120).collect::<String>()
    );
    if pty_dead {
        println!("[B] CONFIRMED: 2º login tumba/rompe el PTY (modelo viejo malo)");
    } else {
        println!("[B] este VPS tolera 2 logins; el bug puede ser otro (set_blocking, etc.)");
    }

    println!("[C] threaded like app: PTY reader + SFTP + typing");
    let sess = auth(&c);
    let mut ch = sess.channel_session().expect("ch");
    ch.request_pty("xterm-256color", None, None).unwrap();
    ch.shell().unwrap();
    sess.set_blocking(false);

    use std::sync::{Arc, Mutex};
    struct Live {
        session: Session,
        channel: ssh2::Channel,
    }
    // Session/Channel aren't Sync in a way we can easily share across - actually
    // ssh2 types are Send. Wrap like the app.
    let live = Arc::new(Mutex::new(Live { session: sess, channel: ch }));
    let live_r = live.clone();
    let stop = Arc::new(Mutex::new(false));
    let stop_r = stop.clone();
    let reader = thread::spawn(move || {
        let mut buf = [0u8; 4096];
        while !*stop_r.lock().unwrap() {
            let res = {
                let mut g = live_r.lock().unwrap();
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
        let mut g = live.lock().unwrap();
        g.channel.write_all(b"echo THREADED_OK\n").unwrap();
        let _ = g.channel.flush();
    }
    thread::sleep(Duration::from_millis(500));

    // SFTP from "UI thread" while reader runs
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
        let entries = loop {
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
        println!("  threaded SFTP OK ({} entries)", entries.len());
        drop(sftp);
    }

    {
        for b in b"cd /var\n" {
            {
                let mut g = live.lock().unwrap();
                g.channel.write_all(&[*b]).unwrap();
                let _ = g.channel.flush();
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    thread::sleep(Duration::from_millis(800));
    let eof = {
        let g = live.lock().unwrap();
        g.channel.eof()
    };
    assert!(!eof, "PTY EOF in threaded smoke");
    println!("  threaded PTY still alive");
    *stop.lock().unwrap() = true;
    let _ = reader.join();
    println!("[C] PASS");

    println!("DONE");
}
