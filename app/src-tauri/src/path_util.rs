/// Escapa un path POSIX para usarlo dentro de comillas simples en un shell Bourne-compatible.
pub fn shell_quote(path: &str) -> String {
    format!("'{}'", path.replace('\'', "'\"'\"'"))
}

/// Padre de un path POSIX remoto (`/` no tiene padre).
#[allow(dead_code)] // usado por tests; espejo de la lógica del frontend
pub fn parent_remote_path(path: &str) -> Option<String> {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() || trimmed == "/" || trimmed == "." {
        return None;
    }
    match trimmed.rfind('/') {
        None => None,
        Some(0) => Some("/".to_string()),
        Some(i) => Some(trimmed[..i].to_string()),
    }
}

/// Une un directorio base con un nombre de entrada, evitando dobles barras.
pub fn join_remote_path(base: &str, name: &str) -> String {
    if base.is_empty() || base == "/" {
        if name.starts_with('/') {
            name.to_string()
        } else {
            format!("/{}", name.trim_start_matches('/'))
        }
    } else if base.ends_with('/') {
        format!("{}{}", base, name.trim_start_matches('/'))
    } else {
        format!("{}/{}", base, name.trim_start_matches('/'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_simple() {
        assert_eq!(shell_quote("/var/log"), "'/var/log'");
    }

    #[test]
    fn shell_quote_con_comilla() {
        assert_eq!(shell_quote("it's"), "'it'\"'\"'s'");
    }

    #[test]
    fn join_desde_raiz() {
        assert_eq!(join_remote_path("/", "home"), "/home");
        assert_eq!(join_remote_path("", "etc"), "/etc");
    }

    #[test]
    fn join_subdir() {
        assert_eq!(join_remote_path("/home/neko", "docs"), "/home/neko/docs");
        assert_eq!(join_remote_path("/home/neko/", "docs"), "/home/neko/docs");
    }

    #[test]
    fn parent_de_subdir() {
        assert_eq!(parent_remote_path("/home/neko/docs"), Some("/home/neko".to_string()));
        assert_eq!(parent_remote_path("/home"), Some("/".to_string()));
        assert_eq!(parent_remote_path("/"), None);
    }
}
