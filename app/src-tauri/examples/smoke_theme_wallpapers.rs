//! Smoke: CRUD theme_wallpapers (SQLite + disco) sin UI.
use app_lib::theme_wallpapers::{
    clear_theme_wallpaper, ensure_theme_wallpapers_schema, get_theme_wallpaper,
    set_theme_wallpaper_file, set_theme_wallpaper_url, WallpaperSourceKind,
};
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

fn main() {
    let conn = Connection::open_in_memory().expect("db");
    ensure_theme_wallpapers_schema(&conn).expect("schema");

    let tmp = std::env::temp_dir().join(format!("nekossh-wp-smoke-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp).expect("tmpdir");
    let src = tmp.join("src.png");
    fs::write(&src, b"PNG-SMOKE").expect("write src");

    let set = set_theme_wallpaper_file(
        &conn,
        &tmp,
        "nekossh",
        src.to_str().unwrap(),
        "src.png",
        0.4,
    )
    .expect("set file");
    assert!(matches!(set.source_kind, WallpaperSourceKind::File));
    let disk = PathBuf::from(tmp.join("wallpapers").join("nekossh.png"));
    assert!(disk.is_file(), "archivo copiado");
    assert_eq!(fs::read(&disk).unwrap(), b"PNG-SMOKE");

    let got = get_theme_wallpaper(&conn, &tmp, "nekossh").expect("get");
    assert!((got.opacity - 0.4).abs() < f64::EPSILON);

    let url = set_theme_wallpaper_url(
        &conn,
        &tmp,
        "nekossh",
        "https://example.com/a.png",
        "a.png",
        0.7,
    )
    .expect("set url");
    assert!(matches!(url.source_kind, WallpaperSourceKind::Url));
    assert!(!disk.exists(), "archivo borrado al pasar a URL");

    clear_theme_wallpaper(&conn, &tmp, "nekossh").expect("clear");
    let empty = get_theme_wallpaper(&conn, &tmp, "nekossh").expect("get empty");
    assert!(matches!(empty.source_kind, WallpaperSourceKind::None));

    let _ = fs::remove_dir_all(&tmp);
    println!("PASS theme_wallpapers smoke (file + url + clear)");
}
