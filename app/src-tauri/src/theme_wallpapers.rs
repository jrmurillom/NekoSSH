//! Theme wallpaper persistence: SQLite metadata + files under app_data/wallpapers/.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_OPACITY: f64 = 0.3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WallpaperSourceKind {
    File,
    Url,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeWallpaperDto {
    pub theme_id: String,
    pub label: String,
    pub opacity: f64,
    pub source_kind: WallpaperSourceKind,
    /// Absolute path (file) or http(s) URL, or empty when none.
    pub display_url: String,
}

pub fn ensure_theme_wallpapers_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(include_str!("../migrations/006_theme_wallpapers.sql"))
        .map_err(|e| format!("Error al crear schema theme_wallpapers: {}", e))?;
    Ok(())
}

pub fn wallpapers_dir(app_data: &Path) -> PathBuf {
    app_data.join("wallpapers")
}

pub fn ensure_wallpapers_dir(app_data: &Path) -> Result<PathBuf, String> {
    let dir = wallpapers_dir(app_data);
    fs::create_dir_all(&dir).map_err(|e| format!("No se pudo crear carpeta wallpapers: {}", e))?;
    Ok(dir)
}

fn clamp_opacity(value: f64) -> f64 {
    if value.is_nan() {
        return DEFAULT_OPACITY;
    }
    value.clamp(0.0, 1.0)
}

fn sanitize_theme_id(theme_id: &str) -> Result<String, String> {
    let t = theme_id.trim();
    if t.is_empty() {
        return Err("theme_id vacío".into());
    }
    if t.contains('/') || t.contains('\\') || t.contains("..") || t.contains(':') {
        return Err("theme_id inválido".into());
    }
    Ok(t.to_string())
}

fn extension_from_path(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| matches!(e.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg"))
        .unwrap_or_else(|| "png".to_string())
}

fn extension_from_data_url(data_url: &str) -> String {
    let lower = data_url.to_ascii_lowercase();
    if lower.starts_with("data:image/jpeg") || lower.starts_with("data:image/jpg") {
        "jpg".into()
    } else if lower.starts_with("data:image/webp") {
        "webp".into()
    } else if lower.starts_with("data:image/gif") {
        "gif".into()
    } else if lower.starts_with("data:image/svg") {
        "svg".into()
    } else {
        "png".into()
    }
}

fn dest_file_name(theme_id: &str, ext: &str) -> String {
    format!("{}.{}", theme_id, ext)
}

fn absolute_wallpaper_path(app_data: &Path, file_name: &str) -> PathBuf {
    wallpapers_dir(app_data).join(file_name)
}

fn delete_wallpaper_file(app_data: &Path, file_name: Option<&str>) {
    let Some(name) = file_name.filter(|s| !s.is_empty()) else {
        return;
    };
    let path = absolute_wallpaper_path(app_data, name);
    let _ = fs::remove_file(path);
}

fn row_to_dto(
    app_data: &Path,
    theme_id: String,
    label: String,
    opacity: f64,
    source_kind: String,
    file_name: Option<String>,
    remote_url: Option<String>,
) -> ThemeWallpaperDto {
    let opacity = clamp_opacity(opacity);
    match source_kind.as_str() {
        "file" => {
            let name = file_name.unwrap_or_default();
            let display_url = if name.is_empty() {
                String::new()
            } else {
                absolute_wallpaper_path(app_data, &name)
                    .to_string_lossy()
                    .replace('\\', "/")
            };
            ThemeWallpaperDto {
                theme_id,
                label,
                opacity,
                source_kind: WallpaperSourceKind::File,
                display_url,
            }
        }
        "url" => ThemeWallpaperDto {
            theme_id,
            label,
            opacity,
            source_kind: WallpaperSourceKind::Url,
            display_url: remote_url.unwrap_or_default(),
        },
        _ => ThemeWallpaperDto {
            theme_id,
            label: String::new(),
            opacity: DEFAULT_OPACITY,
            source_kind: WallpaperSourceKind::None,
            display_url: String::new(),
        },
    }
}

pub fn get_theme_wallpaper(
    conn: &Connection,
    app_data: &Path,
    theme_id: &str,
) -> Result<ThemeWallpaperDto, String> {
    let theme_id = sanitize_theme_id(theme_id)?;
    let row: Option<(String, f64, String, Option<String>, Option<String>)> = conn
        .query_row(
            "SELECT label, opacity, source_kind, file_name, remote_url
             FROM theme_wallpapers WHERE theme_id = ?1",
            params![theme_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(match row {
        Some((label, opacity, kind, file_name, remote_url)) => {
            row_to_dto(app_data, theme_id, label, opacity, kind, file_name, remote_url)
        }
        None => ThemeWallpaperDto {
            theme_id,
            label: String::new(),
            opacity: DEFAULT_OPACITY,
            source_kind: WallpaperSourceKind::None,
            display_url: String::new(),
        },
    })
}

fn upsert_file_row(
    conn: &Connection,
    theme_id: &str,
    label: &str,
    opacity: f64,
    file_name: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO theme_wallpapers (theme_id, label, opacity, source_kind, file_name, remote_url, updated_at)
         VALUES (?1, ?2, ?3, 'file', ?4, NULL, datetime('now'))
         ON CONFLICT(theme_id) DO UPDATE SET
           label = excluded.label,
           opacity = excluded.opacity,
           source_kind = 'file',
           file_name = excluded.file_name,
           remote_url = NULL,
           updated_at = datetime('now')",
        params![theme_id, label, clamp_opacity(opacity), file_name],
    )
    .map_err(|e| format!("Error al guardar wallpaper: {}", e))?;
    Ok(())
}

fn existing_file_name(conn: &Connection, theme_id: &str) -> Option<String> {
    conn.query_row(
        "SELECT file_name FROM theme_wallpapers WHERE theme_id = ?1 AND source_kind = 'file'",
        params![theme_id],
        |r| r.get::<_, Option<String>>(0),
    )
    .ok()
    .flatten()
}

pub fn set_theme_wallpaper_file(
    conn: &Connection,
    app_data: &Path,
    theme_id: &str,
    source_path: &str,
    label: &str,
    opacity: f64,
) -> Result<ThemeWallpaperDto, String> {
    let theme_id = sanitize_theme_id(theme_id)?;
    let source = PathBuf::from(source_path);
    if !source.is_file() {
        return Err(format!("Archivo no encontrado: {}", source_path));
    }

    let dir = ensure_wallpapers_dir(app_data)?;
    let ext = extension_from_path(&source);
    let file_name = dest_file_name(&theme_id, &ext);
    let dest = dir.join(&file_name);

    let previous = existing_file_name(conn, &theme_id);

    fs::copy(&source, &dest).map_err(|e| format!("No se pudo copiar la imagen: {}", e))?;

    if let Err(e) = upsert_file_row(conn, &theme_id, label.trim(), opacity, &file_name) {
        let _ = fs::remove_file(&dest);
        return Err(e);
    }

    if let Some(prev) = previous {
        if prev != file_name {
            delete_wallpaper_file(app_data, Some(&prev));
        }
    }

    get_theme_wallpaper(conn, app_data, &theme_id)
}

pub fn set_theme_wallpaper_bytes(
    conn: &Connection,
    app_data: &Path,
    theme_id: &str,
    bytes: &[u8],
    ext: &str,
    label: &str,
    opacity: f64,
) -> Result<ThemeWallpaperDto, String> {
    let theme_id = sanitize_theme_id(theme_id)?;
    if bytes.is_empty() {
        return Err("Imagen vacía".into());
    }
    let ext = {
        let e = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        if e.is_empty() {
            "png".to_string()
        } else {
            e
        }
    };

    let dir = ensure_wallpapers_dir(app_data)?;
    let file_name = dest_file_name(&theme_id, &ext);
    let dest = dir.join(&file_name);
    let previous = existing_file_name(conn, &theme_id);

    fs::write(&dest, bytes).map_err(|e| format!("No se pudo escribir la imagen: {}", e))?;

    if let Err(e) = upsert_file_row(conn, &theme_id, label.trim(), opacity, &file_name) {
        let _ = fs::remove_file(&dest);
        return Err(e);
    }

    if let Some(prev) = previous {
        if prev != file_name {
            delete_wallpaper_file(app_data, Some(&prev));
        }
    }

    get_theme_wallpaper(conn, app_data, &theme_id)
}

/// Decode a `data:image/...;base64,...` URL and persist as file.
pub fn set_theme_wallpaper_data_url(
    conn: &Connection,
    app_data: &Path,
    theme_id: &str,
    data_url: &str,
    label: &str,
    opacity: f64,
) -> Result<ThemeWallpaperDto, String> {
    let bytes = decode_data_url_bytes(data_url)?;
    let ext = extension_from_data_url(data_url);
    set_theme_wallpaper_bytes(conn, app_data, theme_id, &bytes, &ext, label, opacity)
}

fn decode_data_url_bytes(data_url: &str) -> Result<Vec<u8>, String> {
    let trimmed = data_url.trim();
    if !trimmed.starts_with("data:") {
        return Err("Se esperaba una data URL".into());
    }
    let (_, payload) = trimmed
        .split_once(',')
        .ok_or_else(|| "data URL inválida".to_string())?;
    decode_base64(payload)
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    // Minimal base64 decoder (standard alphabet) to avoid new dependency.
    const TABLE: &[u8; 128] = &{
        let mut t = [0xffu8; 128];
        let mut i = 0u8;
        while i < 26 {
            t[(b'A' + i) as usize] = i;
            t[(b'a' + i) as usize] = 26 + i;
            i += 1;
        }
        i = 0;
        while i < 10 {
            t[(b'0' + i) as usize] = 52 + i;
            i += 1;
        }
        t[b'+' as usize] = 62;
        t[b'/' as usize] = 63;
        t
    };

    let clean: Vec<u8> = input
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if clean.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::with_capacity(clean.len() * 3 / 4);
    let mut buf = [0u8; 4];
    let mut buf_len = 0;
    for &b in &clean {
        if b == b'=' {
            break;
        }
        if b as usize >= TABLE.len() || TABLE[b as usize] == 0xff {
            return Err("base64 inválido en data URL".into());
        }
        buf[buf_len] = TABLE[b as usize];
        buf_len += 1;
        if buf_len == 4 {
            out.push((buf[0] << 2) | (buf[1] >> 4));
            out.push((buf[1] << 4) | (buf[2] >> 2));
            out.push((buf[2] << 6) | buf[3]);
            buf_len = 0;
        }
    }
    if buf_len == 2 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
    } else if buf_len == 3 {
        out.push((buf[0] << 2) | (buf[1] >> 4));
        out.push((buf[1] << 4) | (buf[2] >> 2));
    } else if buf_len == 1 {
        return Err("base64 inválido en data URL".into());
    }
    Ok(out)
}

pub fn set_theme_wallpaper_url(
    conn: &Connection,
    app_data: &Path,
    theme_id: &str,
    url: &str,
    label: &str,
    opacity: f64,
) -> Result<ThemeWallpaperDto, String> {
    let theme_id = sanitize_theme_id(theme_id)?;
    let url = url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("Solo se aceptan URLs http(s)".into());
    }

    let previous = existing_file_name(conn, &theme_id);

    conn.execute(
        "INSERT INTO theme_wallpapers (theme_id, label, opacity, source_kind, file_name, remote_url, updated_at)
         VALUES (?1, ?2, ?3, 'url', NULL, ?4, datetime('now'))
         ON CONFLICT(theme_id) DO UPDATE SET
           label = excluded.label,
           opacity = excluded.opacity,
           source_kind = 'url',
           file_name = NULL,
           remote_url = excluded.remote_url,
           updated_at = datetime('now')",
        params![
            theme_id,
            if label.trim().is_empty() {
                url
            } else {
                label.trim()
            },
            clamp_opacity(opacity),
            url
        ],
    )
    .map_err(|e| format!("Error al guardar wallpaper URL: {}", e))?;

    if let Some(prev) = previous {
        delete_wallpaper_file(app_data, Some(&prev));
    }

    get_theme_wallpaper(conn, app_data, &theme_id)
}

pub fn set_theme_wallpaper_opacity(
    conn: &Connection,
    app_data: &Path,
    theme_id: &str,
    opacity: f64,
) -> Result<ThemeWallpaperDto, String> {
    let theme_id = sanitize_theme_id(theme_id)?;
    let current = get_theme_wallpaper(conn, app_data, &theme_id)?;
    if matches!(current.source_kind, WallpaperSourceKind::None) && current.display_url.is_empty() {
        // Persist opacity-only row as empty url kind? Spec says opacity with wallpaper.
        // Keep a url row with empty remote? Better: upsert opacity on existing or create url with empty — skip.
        // If no wallpaper, still store opacity by creating a stub? Design: opacity is part of wallpaper entry.
        // For slider with no image, we can no-op get with updated opacity in memory only —
        // but persistBackgroundOpacity currently always saves. Create file-less? Use url with empty invalid.
        // Simplest: if none, insert url kind with remote_url '' — violates CHECK usefulness.
        // Allow source_kind to stay and only update if row exists; if not, insert with source url and remote_url placeholder?
        // Looking at frontend: opacity is saved even without image. Store as:
        // source_kind 'url', remote_url NULL or '' — but then display is empty.
        conn.execute(
            "INSERT INTO theme_wallpapers (theme_id, label, opacity, source_kind, file_name, remote_url, updated_at)
             VALUES (?1, '', ?2, 'url', NULL, '', datetime('now'))
             ON CONFLICT(theme_id) DO UPDATE SET
               opacity = excluded.opacity,
               updated_at = datetime('now')",
            params![theme_id, clamp_opacity(opacity)],
        )
        .map_err(|e| e.to_string())?;
        return get_theme_wallpaper(conn, app_data, &theme_id);
    }

    conn.execute(
        "UPDATE theme_wallpapers SET opacity = ?1, updated_at = datetime('now') WHERE theme_id = ?2",
        params![clamp_opacity(opacity), theme_id],
    )
    .map_err(|e| e.to_string())?;
    get_theme_wallpaper(conn, app_data, &theme_id)
}

pub fn clear_theme_wallpaper(
    conn: &Connection,
    app_data: &Path,
    theme_id: &str,
) -> Result<(), String> {
    let theme_id = sanitize_theme_id(theme_id)?;
    let previous = existing_file_name(conn, &theme_id);
    conn.execute(
        "DELETE FROM theme_wallpapers WHERE theme_id = ?1",
        params![theme_id],
    )
    .map_err(|e| format!("Error al eliminar wallpaper: {}", e))?;
    delete_wallpaper_file(app_data, previous.as_deref());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn open_db() -> Connection {
        let conn = Connection::open_in_memory().expect("mem");
        ensure_theme_wallpapers_schema(&conn).expect("schema");
        conn
    }

    #[test]
    fn get_sin_fila_devuelve_none() {
        let conn = open_db();
        let tmp = tempfile_dir();
        let dto = get_theme_wallpaper(&conn, &tmp, "nekossh").unwrap();
        assert!(matches!(dto.source_kind, WallpaperSourceKind::None));
        assert_eq!(dto.display_url, "");
        assert_eq!(dto.opacity, DEFAULT_OPACITY);
    }

    #[test]
    fn set_file_get_clear_round_trip() {
        let conn = open_db();
        let tmp = tempfile_dir();
        let src = tmp.join("source.png");
        fs::write(&src, b"fake-png-bytes").unwrap();

        let dto = set_theme_wallpaper_file(
            &conn,
            &tmp,
            "nekossh",
            src.to_str().unwrap(),
            "source.png",
            0.55,
        )
        .unwrap();
        assert!(matches!(dto.source_kind, WallpaperSourceKind::File));
        assert_eq!(dto.label, "source.png");
        assert!((dto.opacity - 0.55).abs() < f64::EPSILON);
        assert!(PathBuf::from(&dto.display_url.replace('/', std::path::MAIN_SEPARATOR_STR)).exists() || Path::new(&dto.display_url).exists() || fs::metadata(wallpapers_dir(&tmp).join("nekossh.png")).is_ok());

        assert!(wallpapers_dir(&tmp).join("nekossh.png").is_file());

        clear_theme_wallpaper(&conn, &tmp, "nekossh").unwrap();
        assert!(!wallpapers_dir(&tmp).join("nekossh.png").exists());
        let after = get_theme_wallpaper(&conn, &tmp, "nekossh").unwrap();
        assert!(matches!(after.source_kind, WallpaperSourceKind::None));
    }

    #[test]
    fn set_url_y_opacity() {
        let conn = open_db();
        let tmp = tempfile_dir();
        let dto = set_theme_wallpaper_url(
            &conn,
            &tmp,
            "hatsune-miku",
            "https://example.com/bg.png",
            "",
            0.2,
        )
        .unwrap();
        assert!(matches!(dto.source_kind, WallpaperSourceKind::Url));
        assert_eq!(dto.display_url, "https://example.com/bg.png");

        let updated = set_theme_wallpaper_opacity(&conn, &tmp, "hatsune-miku", 0.9).unwrap();
        assert!((updated.opacity - 0.9).abs() < f64::EPSILON);
        assert_eq!(updated.display_url, "https://example.com/bg.png");
    }

    #[test]
    fn data_url_persiste_bytes() {
        let conn = open_db();
        let tmp = tempfile_dir();
        // "hi" in base64 = aGk=
        let data = "data:image/png;base64,aGk=";
        let dto =
            set_theme_wallpaper_data_url(&conn, &tmp, "persona5", data, "x.png", 0.4).unwrap();
        assert!(matches!(dto.source_kind, WallpaperSourceKind::File));
        let path = wallpapers_dir(&tmp).join("persona5.png");
        assert_eq!(fs::read(path).unwrap(), b"hi");
    }

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nekossh-wp-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
