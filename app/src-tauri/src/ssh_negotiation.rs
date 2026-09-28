//! Módulo de negociación criptográfica SSH para NekoSSH.
//!
//! Desacopla y centraliza las preferencias de algoritmos (KEX, Ciphers, HostKey y MAC)
//! para garantizar una compatibilidad amplia con servidores modernos y legacy (cPanel, CentOS, appliances),
//! manteniendo una estricta jerarquía de seguridad donde los métodos modernos siempre tienen precedencia.

use ssh2::{MethodType, Session};

/// Algoritmos de Intercambio de Claves (Key Exchange) ordenados por seguridad decreciente.
///
/// Prioriza algoritmos de curvas elípticas modernas (Curve25519, NIST ECDH), seguidos de Diffie-Hellman
/// Group 16/14 con SHA-2, y finaliza con fallback para servidores empresariales/cPanel que requieren
/// Diffie-Hellman Group Exchange y Group 14 con SHA-1.
pub const KEX_PREFERENCES: &str = "\
    curve25519-sha256,\
    curve25519-sha256@libssh.org,\
    ecdh-sha2-nistp256,\
    ecdh-sha2-nistp384,\
    ecdh-sha2-nistp521,\
    diffie-hellman-group16-sha512,\
    diffie-hellman-group14-sha256,\
    diffie-hellman-group-exchange-sha256,\
    diffie-hellman-group14-sha1,\
    diffie-hellman-group-exchange-sha1";

/// Algoritmos de Cifrado simétrico ordenados por seguridad decreciente.
///
/// Prioriza modos AEAD y CTR (AES-GCM, ChaCha20-Poly1305, AES-CTR), seguidos por modos CBC
/// (AES-CBC, 3DES-CBC) estrictamente como fallback para servidores antiguos que no soportan CTR/GCM.
pub const CIPHER_PREFERENCES: &str = "\
    aes256-gcm@openssh.com,\
    chacha20-poly1305@openssh.com,\
    aes256-ctr,\
    aes192-ctr,\
    aes128-ctr,\
    aes128-gcm@openssh.com,\
    aes256-cbc,\
    aes192-cbc,\
    aes128-cbc,\
    3des-cbc";

/// Algoritmos de Clave de Host (HostKey) ordenados por seguridad decreciente.
pub const HOSTKEY_PREFERENCES: &str = "\
    ssh-ed25519,\
    ecdsa-sha2-nistp256,\
    ecdsa-sha2-nistp384,\
    ecdsa-sha2-nistp521,\
    rsa-sha2-512,\
    rsa-sha2-256,\
    ssh-rsa";

/// Algoritmos de Integridad de Mensaje (MAC) ordenados por seguridad decreciente.
///
/// Prioriza variantes Encrypt-then-MAC (ETM) con SHA-2, seguidas de HMAC estándar y fallback a SHA-1.
pub const MAC_PREFERENCES: &str = "\
    hmac-sha2-256-etm@openssh.com,\
    hmac-sha2-512-etm@openssh.com,\
    hmac-sha2-256,\
    hmac-sha2-512,\
    hmac-sha1";

/// Aplica todas las preferencias de algoritmos a una sesión de libssh2 antes del handshake.
///
/// Configura KEX, Ciphers (Cliente->Servidor y Servidor->Cliente), HostKey y MAC.
/// Retorna error descriptivo si libssh2 rechaza alguna de las configuraciones.
pub fn configure_session_methods(sess: &mut Session) -> Result<(), String> {
    sess.method_pref(MethodType::Kex, KEX_PREFERENCES)
        .map_err(|e| format!("Error configurando preferencia de KEX: {}", e))?;

    sess.method_pref(MethodType::CryptCs, CIPHER_PREFERENCES)
        .map_err(|e| format!("Error configurando cifrado Cliente->Servidor: {}", e))?;

    sess.method_pref(MethodType::CryptSc, CIPHER_PREFERENCES)
        .map_err(|e| format!("Error configurando cifrado Servidor->Cliente: {}", e))?;

    sess.method_pref(MethodType::HostKey, HOSTKEY_PREFERENCES)
        .map_err(|e| format!("Error configurando preferencia de HostKey: {}", e))?;

    sess.method_pref(MethodType::MacCs, MAC_PREFERENCES)
        .map_err(|e| format!("Error configurando MAC Cliente->Servidor: {}", e))?;

    sess.method_pref(MethodType::MacSc, MAC_PREFERENCES)
        .map_err(|e| format!("Error configurando MAC Servidor->Cliente: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_methods(prefs: &str) -> Vec<&str> {
        prefs.split(',').map(str::trim).filter(|s| !s.is_empty()).collect()
    }

    #[test]
    fn test_kex_precedence_and_security() {
        let methods = parse_methods(KEX_PREFERENCES);
        assert!(!methods.is_empty(), "KEX_PREFERENCES no debe estar vacío");

        // Curve25519 debe ser la máxima prioridad (primeros elementos)
        assert!(
            methods[0].contains("curve25519"),
            "El primer algoritmo KEX debe ser Curve25519"
        );

        let pos_curve25519 = methods.iter().position(|&m| m == "curve25519-sha256").unwrap();
        let pos_dh_gex = methods.iter().position(|&m| m == "diffie-hellman-group-exchange-sha256").unwrap();
        let pos_dh_sha1 = methods.iter().position(|&m| m == "diffie-hellman-group14-sha1").unwrap();

        assert!(
            pos_curve25519 < pos_dh_gex,
            "Curve25519 ({}) debe preceder a DH-GEX ({})",
            pos_curve25519,
            pos_dh_gex
        );
        assert!(
            pos_dh_gex < pos_dh_sha1,
            "DH-GEX SHA-256 ({}) debe preceder a DH SHA-1 ({})",
            pos_dh_gex,
            pos_dh_sha1
        );
    }

    #[test]
    fn test_cipher_precedence_and_security() {
        let methods = parse_methods(CIPHER_PREFERENCES);
        assert!(!methods.is_empty(), "CIPHER_PREFERENCES no debe estar vacío");

        let pos_gcm = methods.iter().position(|&m| m == "aes256-gcm@openssh.com").unwrap();
        let pos_ctr = methods.iter().position(|&m| m == "aes256-ctr").unwrap();
        let pos_cbc = methods.iter().position(|&m| m == "aes256-cbc").unwrap();
        let pos_3des = methods.iter().position(|&m| m == "3des-cbc").unwrap();

        // GCM y CTR deben preceder estrictamente a CBC y 3DES
        assert!(pos_gcm < pos_cbc, "AES-GCM ({}) debe preceder a AES-CBC ({})", pos_gcm, pos_cbc);
        assert!(pos_ctr < pos_cbc, "AES-CTR ({}) debe preceder a AES-CBC ({})", pos_ctr, pos_cbc);
        assert!(pos_cbc < pos_3des, "AES-CBC ({}) debe preceder a 3DES ({})", pos_cbc, pos_3des);
    }

    #[test]
    fn test_cpanel_centos_mandatory_algorithms_present() {
        let kex = parse_methods(KEX_PREFERENCES);
        let ciphers = parse_methods(CIPHER_PREFERENCES);

        // KEX requerido por cPanel y OpenSSH corporativo
        assert!(kex.contains(&"diffie-hellman-group-exchange-sha256"), "Falta diffie-hellman-group-exchange-sha256");
        assert!(kex.contains(&"diffie-hellman-group-exchange-sha1"), "Falta diffie-hellman-group-exchange-sha1");
        assert!(kex.contains(&"diffie-hellman-group14-sha1"), "Falta diffie-hellman-group14-sha1");

        // Cifrados requeridos por configuraciones antiguas
        assert!(ciphers.contains(&"aes256-cbc"), "Falta aes256-cbc");
        assert!(ciphers.contains(&"aes128-cbc"), "Falta aes128-cbc");
        assert!(ciphers.contains(&"3des-cbc"), "Falta 3des-cbc");
    }

    #[test]
    fn test_no_duplicate_methods() {
        for (name, list_str) in [
            ("KEX", KEX_PREFERENCES),
            ("CIPHER", CIPHER_PREFERENCES),
            ("HOSTKEY", HOSTKEY_PREFERENCES),
            ("MAC", MAC_PREFERENCES),
        ] {
            let methods = parse_methods(list_str);
            let mut seen = std::collections::HashSet::new();
            for m in methods {
                assert!(
                    seen.insert(m),
                    "Algoritmo duplicado '{}' encontrado en {}",
                    m,
                    name
                );
            }
        }
    }

    #[test]
    fn test_configure_session_methods_on_live_session() {
        let mut sess = Session::new().expect("Debe crear sesión SSH en memoria");
        let result = configure_session_methods(&mut sess);
        assert!(
            result.is_ok(),
            "configure_session_methods falló en libssh2: {:?}",
            result.err()
        );
    }
}
