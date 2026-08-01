#![allow(dead_code)] // módulo retenido para tests unitarios; sync OSC 7 fuera de alcance de producto

//! Parser de secuencias OSC 7 (cwd del shell) en salida PTY.

/// Extrae paths de cwd desde chunks que pueden contener OSC 7.
/// Devuelve `(paths, leftover)` donde leftover es un prefijo incompleto de secuencia.
pub fn extract_cwd_paths(input: &str) -> (Vec<String>, String) {
    let mut paths = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // ESC ]
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b']' {
            let seq_start = i;
            let start = i + 2;
            let mut j = start;
            let mut found_term = false;
            let mut bel = true;
            while j < bytes.len() {
                if bytes[j] == 0x07 {
                    found_term = true;
                    bel = true;
                    break;
                }
                if bytes[j] == 0x1b && j + 1 < bytes.len() && bytes[j + 1] == b'\\' {
                    found_term = true;
                    bel = false;
                    break;
                }
                j += 1;
            }
            if !found_term {
                return (
                    paths,
                    String::from_utf8_lossy(&bytes[seq_start..]).into_owned(),
                );
            }
            let body = String::from_utf8_lossy(&bytes[start..j]);
            if let Some(path) = parse_osc7_body(&body) {
                paths.push(path);
            }
            i = if bel { j + 1 } else { j + 2 };
            continue;
        }
        i += 1;
    }

    (paths, String::new())
}

fn parse_osc7_body(body: &str) -> Option<String> {
    // Esperado: 7;file://host/path  o  7;file:///path
    let rest = body.strip_prefix("7;")?;
    let uri = rest.strip_prefix("file://")?;
    // uri = host/path | /path | host
    let path = if let Some(slash) = uri.find('/') {
        let after_host = &uri[slash..];
        if after_host.is_empty() {
            "/".to_string()
        } else {
            after_host.to_string()
        }
    } else if uri.is_empty() {
        "/".to_string()
    } else {
        // solo hostname sin path → no usable
        return None;
    };
    // Decodificar %XX básico
    Some(percent_decode(&path))
}

fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4 | l) as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// One-liner Bourne/bash para emitir OSC 7 del $PWD y engancharlo al prompt.
#[allow(dead_code)]
pub fn bash_osc7_enable_command() -> &'static str {
    "printf '\\033]7;file://localhost%s\\007' \"$PWD\"; PROMPT_COMMAND='printf \"\\033]7;file://localhost$PWD\\007\"'\r"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osc7_bel_extrae_path() {
        let (paths, left) =
            extract_cwd_paths("antes\x1b]7;file://host/tmp/demo\x07despues");
        assert_eq!(paths, vec!["/tmp/demo".to_string()]);
        assert!(left.is_empty());
    }

    #[test]
    fn osc7_st_terminator() {
        let (paths, _) = extract_cwd_paths("\x1b]7;file://localhost/var/log\x1b\\");
        assert_eq!(paths, vec!["/var/log".to_string()]);
    }

    #[test]
    fn osc7_file_triple_slash() {
        let (paths, _) = extract_cwd_paths("\x1b]7;file:///home/neko\x07");
        assert_eq!(paths, vec!["/home/neko".to_string()]);
    }

    #[test]
    fn osc7_incompleto_queda_en_leftover() {
        let (paths, left) = extract_cwd_paths("xyz\x1b]7;file://host/par");
        assert!(paths.is_empty());
        assert!(left.starts_with('\u{1b}'));
    }

    #[test]
    fn osc7_percent_decode() {
        let (paths, _) = extract_cwd_paths("\x1b]7;file://h/tmp/my%20dir\x07");
        assert_eq!(paths, vec!["/tmp/my dir".to_string()]);
    }

    #[test]
    fn osc7_sin_secuencia() {
        let (paths, left) = extract_cwd_paths("hello world");
        assert!(paths.is_empty());
        assert!(left.is_empty());
    }
}
