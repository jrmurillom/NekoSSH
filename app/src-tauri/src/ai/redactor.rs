//! Sanitizador preventivo de información sensible y credenciales (`SecretRedactor`).
//! Implementación segura sobre bytes ASCII para evitar desalineaciones UTF-8 con caracteres multibyte.

/// Sanitiza cadenas de texto para evitar la fuga de secretos hacia logs o modelos de IA.
pub struct SecretRedactor;

impl SecretRedactor {
    /// Devuelve una copia de la cadena con llaves privadas, tokens y credenciales enmascaradas.
    pub fn redact(text: &str) -> String {
        let mut result = text.to_string();
        result = redact_pem_blocks(&result);
        result = redact_bearer_tokens(&result);
        result = redact_url_passwords(&result);
        result = redact_key_value_secrets(&result);
        result = redact_structured_kv_secrets(&result);
        result = redact_known_token_prefixes(&result);
        result
    }
}

/// Enmascara bloques completos de llaves privadas PEM.
fn redact_pem_blocks(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;

    while let Some(start_idx) = text[cursor..].find("-----BEGIN ") {
        let actual_start = cursor + start_idx;
        if let Some(end_marker_rel) = text[actual_start..].find("-----END ") {
            let end_marker_start = actual_start + end_marker_rel;
            if let Some(dashes_end_rel) = text[end_marker_start..].find("-----") {
                let second_dashes = end_marker_start + dashes_end_rel + 5;
                if let Some(final_dashes_rel) = text[second_dashes..].find("-----") {
                    let block_end = second_dashes + final_dashes_rel + 5;
                    output.push_str(&text[cursor..actual_start]);
                    output.push_str("[REDACTED_PRIVATE_KEY]");
                    cursor = block_end;
                    continue;
                }
            }
        }
        break;
    }

    output.push_str(&text[cursor..]);
    output
}

/// Enmascara encabezados o tokens tipo Bearer.
fn redact_bearer_tokens(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;

    while let Some(pos) = find_ascii_case_insensitive(&text[cursor..], b"BEARER ") {
        let start = cursor + pos;
        output.push_str(&text[cursor..start + 7]);
        cursor = start + 7;

        let token_len = text[cursor..]
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
            .unwrap_or(text.len() - cursor);

        if token_len > 0 {
            output.push_str("[REDACTED_TOKEN]");
            cursor += token_len;
        }
    }

    output.push_str(&text[cursor..]);
    output
}

/// Busca una aguja ASCII en mayúsculas de manera insensible a mayúsculas sobre los bytes del texto,
/// garantizando que el índice retornado caiga siempre en una frontera válida UTF-8.
fn find_ascii_case_insensitive(haystack: &str, needle_upper_ascii: &[u8]) -> Option<usize> {
    let hay_bytes = haystack.as_bytes();
    if needle_upper_ascii.is_empty() || hay_bytes.len() < needle_upper_ascii.len() {
        return None;
    }
    hay_bytes
        .windows(needle_upper_ascii.len())
        .position(|window| window.eq_ignore_ascii_case(needle_upper_ascii))
}

fn is_sensitive_key_name(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "password"
            | "passwd"
            | "secret"
            | "token"
            | "api_key"
            | "apikey"
            | "access_key"
            | "secret_key"
            | "client_secret"
            | "auth_token"
            | "private_key"
    ) || lower.ends_with("_password")
        || lower.ends_with("_passwd")
        || lower.ends_with("_secret")
        || lower.ends_with("_token")
        || lower.ends_with("_api_key")
        || lower.ends_with("_access_key")
        || lower.ends_with("_secret_key")
        || lower.ends_with("_private_key")
        || lower.ends_with("_auth_key")
}

/// Enmascara asignaciones sensibles como PASSWORD=..., AWS_SECRET_ACCESS_KEY=..., SECRET=... sin riesgo UTF-8.
fn redact_key_value_secrets(text: &str) -> String {
    let mut lines = Vec::new();
    for line in text.lines() {
        let mut processed_line = line.to_string();
        let mut search_from = 0usize;

        while search_from < processed_line.len() {
            let Some(rel_eq) = processed_line[search_from..].find('=') else {
                break;
            };
            let eq_idx = search_from + rel_eq;
            let bytes = processed_line.as_bytes();

            let mut key_start = eq_idx;
            while key_start > 0 {
                let b = bytes[key_start - 1];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    key_start -= 1;
                } else {
                    break;
                }
            }
            let key_name = &processed_line[key_start..eq_idx];
            if key_name.is_empty() || !is_sensitive_key_name(key_name) {
                search_from = eq_idx + 1;
                continue;
            }

            let val_start = eq_idx + 1;
            let remainder = &processed_line[val_start..];
            let (prefix, rest, quote_char) = if remainder.starts_with('"') {
                ("\"", &remainder[1..], Some('"'))
            } else if remainder.starts_with('\'') {
                ("'", &remainder[1..], Some('\''))
            } else {
                ("", remainder, None)
            };

            let val_len = match quote_char {
                Some(q) => rest.find(q).unwrap_or(rest.len()),
                None => rest
                    .find(|c: char| c.is_whitespace() || c == ';' || c == '&')
                    .unwrap_or(rest.len()),
            };

            if val_len == 0 {
                search_from = val_start + prefix.len();
                continue;
            }

            let val_end = val_start + prefix.len() + val_len;
            let mut new_line = String::with_capacity(processed_line.len());
            new_line.push_str(&processed_line[..val_start]);
            new_line.push_str(prefix);
            new_line.push_str("[REDACTED_SECRET]");
            new_line.push_str(&processed_line[val_end..]);

            search_from = val_start + prefix.len() + "[REDACTED_SECRET]".len();
            processed_line = new_line;
        }
        lines.push(processed_line);
    }

    let mut res = lines.join("\n");
    if text.ends_with('\n') {
        res.push('\n');
    }
    res
}

/// Enmascara claves sensibles en JSON o YAML (`"api_key": "..."`, `token: ...`) respetando fronteras UTF-8.
fn redact_structured_kv_secrets(text: &str) -> String {
    let mut lines = Vec::new();
    for line in text.lines() {
        let mut processed = line.to_string();
        let mut search_from = 0usize;

        while search_from < processed.len() {
            let Some(rel_colon) = processed[search_from..].find(':') else {
                break;
            };
            let colon_idx = search_from + rel_colon;
            let before_colon = processed[..colon_idx].trim_end_matches([' ', '\t']);

            let key_token = if let Some(unquoted_end) = before_colon.strip_suffix('"') {
                unquoted_end
                    .rsplit('"')
                    .next()
                    .unwrap_or("")
            } else if let Some(unquoted_end) = before_colon.strip_suffix('\'') {
                unquoted_end
                    .rsplit('\'')
                    .next()
                    .unwrap_or("")
            } else {
                let bytes = before_colon.as_bytes();
                let mut start = bytes.len();
                while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
                    start -= 1;
                }
                &before_colon[start..]
            };

            if key_token.is_empty()
                || !key_token.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                || !is_sensitive_key_name(key_token)
            {
                search_from = colon_idx + 1;
                continue;
            }

            let after_colon = &processed[colon_idx + 1..];
            let ws_len = after_colon
                .find(|c: char| c != ' ' && c != '\t')
                .unwrap_or(after_colon.len());
            let val_start = colon_idx + 1 + ws_len;
            let rem = &processed[val_start..];
            if rem.is_empty() {
                search_from = colon_idx + 1;
                continue;
            }

            let (prefix, rest, quote_char) = if rem.starts_with('"') {
                ("\"", &rem[1..], Some('"'))
            } else if rem.starts_with('\'') {
                ("'", &rem[1..], Some('\''))
            } else {
                ("", rem, None)
            };

            let val_len = match quote_char {
                Some(q) => rest.find(q).unwrap_or(rest.len()),
                None => rest
                    .find(|c: char| {
                        c.is_whitespace() || matches!(c, ',' | '}' | ']' | ';' | '#')
                    })
                    .unwrap_or(rest.len()),
            };

            let raw_val = &rest[..val_len];
            if val_len == 0
                || raw_val == "[REDACTED_SECRET]"
                || matches!(raw_val, "null" | "true" | "false")
            {
                search_from = val_start + prefix.len() + val_len;
                continue;
            }

            let val_end = val_start + prefix.len() + val_len;
            let mut new_line = String::with_capacity(processed.len());
            new_line.push_str(&processed[..val_start]);
            new_line.push_str(prefix);
            new_line.push_str("[REDACTED_SECRET]");
            new_line.push_str(&processed[val_end..]);

            search_from = val_start + prefix.len() + "[REDACTED_SECRET]".len();
            processed = new_line;
        }
        lines.push(processed);
    }

    let mut res = lines.join("\n");
    if text.ends_with('\n') {
        res.push('\n');
    }
    res
}

/// Enmascara tokens con prefijos conocidos (`sk-`, `ghp_`, `gho_`, `github_pat_`, `glpat-`, `AKIA`).
fn redact_known_token_prefixes(text: &str) -> String {
    const PREFIXES: [&str; 6] = ["github_pat_", "glpat-", "ghp_", "gho_", "sk-", "AKIA"];
    let mut output = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut idx = 0usize;

    while idx < bytes.len() {
        let matched = PREFIXES.iter().copied().find(|p| bytes[idx..].starts_with(p.as_bytes()));
        if let Some(prefix) = matched {
            let valid_prev = idx == 0 || {
                let prev = bytes[idx - 1];
                !prev.is_ascii_alphanumeric() && prev != b'_' && prev != b'-'
            };
            if valid_prev {
                let mut end = idx + prefix.len();
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_' || bytes[end] == b'-')
                {
                    end += 1;
                }
                let token_len = end - idx;
                let suffix_len = end - (idx + prefix.len());
                let is_real_secret = if prefix == "AKIA" {
                    token_len == 20
                        && bytes[idx..end]
                            .iter()
                            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
                } else {
                    suffix_len >= 16
                };

                if is_real_secret {
                    output.push_str("[REDACTED_TOKEN]");
                    idx = end;
                    continue;
                }
            }
        }

        let Some(ch) = text[idx..].chars().next() else {
            break;
        };
        output.push(ch);
        idx += ch.len_utf8();
    }
    output
}

/// Enmascara contraseñas incrustadas en URLs: https://user:password@host -> https://user:[REDACTED_SECRET]@host
fn redact_url_passwords(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;

    let schemes = ["http://", "https://", "postgres://", "mysql://", "redis://", "mongodb://"];

    while let Some((scheme_start, scheme)) = find_earliest_scheme(&text[cursor..], &schemes) {
        let actual_start = cursor + scheme_start;
        output.push_str(&text[cursor..actual_start + scheme.len()]);
        cursor = actual_start + scheme.len();

        let url_tail = &text[cursor..];
        let url_end_rel = url_tail
            .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
            .unwrap_or(url_tail.len());

        let url_authority = &url_tail[..url_end_rel];
        if let Some(at_idx) = url_authority.find('@') {
            let userinfo = &url_authority[..at_idx];
            if let Some(colon_idx) = userinfo.find(':') {
                output.push_str(&userinfo[..colon_idx + 1]);
                output.push_str("[REDACTED_SECRET]");
                output.push_str(&url_authority[at_idx..]);
                cursor += url_end_rel;
                continue;
            }
        }

        output.push_str(&url_tail[..url_end_rel]);
        cursor += url_end_rel;
    }

    output.push_str(&text[cursor..]);
    output
}

fn find_earliest_scheme<'a>(text: &'a str, schemes: &[&'a str]) -> Option<(usize, &'a str)> {
    let mut earliest: Option<(usize, &'a str)> = None;
    for scheme in schemes {
        if let Some(pos) = text.find(scheme) {
            match earliest {
                Some((min_pos, _)) if pos < min_pos => {
                    earliest = Some((pos, scheme));
                }
                None => {
                    earliest = Some((pos, scheme));
                }
                _ => {}
            }
        }
    }
    earliest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enmascara_bloque_pem() {
        let input = "Antes\n-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAA\n-----END OPENSSH PRIVATE KEY-----\nDespues";
        let redacted = SecretRedactor::redact(input);
        assert!(!redacted.contains("b3BlbnNzaC"));
        assert!(redacted.contains("[REDACTED_PRIVATE_KEY]"));
        assert!(redacted.contains("Antes"));
        assert!(redacted.contains("Despues"));
    }

    #[test]
    fn enmascara_bearer_token() {
        let input = "curl -H 'Authorization: Bearer my-secret-jwt-token-12345' https://api.com";
        let redacted = SecretRedactor::redact(input);
        assert!(!redacted.contains("my-secret-jwt-token-12345"));
        assert!(redacted.contains("Bearer [REDACTED_TOKEN]"));
    }

    #[test]
    fn enmascara_asignaciones_sensibles_y_unicode_multibyte_sin_panic() {
        // ß y İ cambian de tamaño en bytes al pasar a mayúsculas; no debe provocar panic UTF-8.
        let input = "Straße İSTANBUL export password=\"super_secret_123\" API_KEY=xyz987654321; echo ñandú";
        let redacted = SecretRedactor::redact(input);
        assert!(!redacted.contains("super_secret_123"));
        assert!(!redacted.contains("xyz987654321"));
        assert!(redacted.contains("Straße İSTANBUL"));
        assert!(redacted.contains("password=\"[REDACTED_SECRET]\""));
        assert!(redacted.contains("API_KEY=[REDACTED_SECRET]"));
        assert!(redacted.contains("ñandú"));
    }

    #[test]
    fn enmascara_credenciales_en_url() {
        let input = "DATABASE_URL=postgres://app_user:ultra_secure_pass@db.local:5432/production";
        let redacted = SecretRedactor::redact(input);
        assert!(!redacted.contains("ultra_secure_pass"));
        assert!(redacted.contains("postgres://app_user:[REDACTED_SECRET]@db.local:5432/production"));
    }

    #[test]
    fn enmascara_claves_compuestas_json_yaml_y_prefijos_cloud_sin_falsos_positivos() {
        let input = r#"
AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
DATABASE_URL_PASSWORD="mi_pass_prod_987"
{"api_key": "super_json_key_123456", "password": "json_pw_value", "ok": "normal"}
token: yaml_secret_token_999
openai=sk-proj-abc1234567890XYZ987654
gh=ghp_1234567890abcdefghijKLMNOPQRST
gl=glpat-abcdef1234567890XYZ12
aws_id=AKIAIOSFODNN7EXAMPLE
nota=sk-test
"#;
        let redacted = SecretRedactor::redact(input);
        assert!(!redacted.contains("wJalrXUtnFEMI"));
        assert!(!redacted.contains("mi_pass_prod_987"));
        assert!(!redacted.contains("super_json_key_123456"));
        assert!(!redacted.contains("json_pw_value"));
        assert!(!redacted.contains("yaml_secret_token_999"));
        assert!(!redacted.contains("sk-proj-abc1234567890XYZ987654"));
        assert!(!redacted.contains("ghp_1234567890abcdefghijKLMNOPQRST"));
        assert!(!redacted.contains("glpat-abcdef1234567890XYZ12"));
        assert!(!redacted.contains("AKIAIOSFODNN7EXAMPLE"));
        // Etiquetas preservadas y texto corto benigno intacto
        assert!(redacted.contains("AWS_SECRET_ACCESS_KEY=[REDACTED_SECRET]"));
        assert!(redacted.contains("\"ok\": \"normal\""));
        assert!(redacted.contains("nota=sk-test"));
    }
}
