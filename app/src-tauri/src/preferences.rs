//! Persistencia de preferencias de app (`app_preferences` key/value).

use rusqlite::Connection;

pub const PREFERRED_EXTERNAL_EDITOR_KEY: &str = "preferred_external_editor";

/// Idempotent schema for rusqlite path + in-memory tests (plugin migration 003 runs once).
pub fn ensure_app_preferences_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS app_preferences (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL DEFAULT ''
        );
        "#,
    )
    .map_err(|e| format!("Error al crear app_preferences: {}", e))?;
    Ok(())
}

pub fn get_preference(conn: &Connection, key: &str) -> Result<String, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM app_preferences WHERE key = ?1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map([key], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    match rows.next() {
        Some(Ok(v)) => Ok(v),
        Some(Err(e)) => Err(e.to_string()),
        None => Ok(String::new()),
    }
}

pub fn set_preference(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO app_preferences (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        (key, value),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_preferred_external_editor(conn: &Connection) -> Result<String, String> {
    get_preference(conn, PREFERRED_EXTERNAL_EDITOR_KEY)
}

pub fn set_preferred_external_editor(conn: &Connection, path: &str) -> Result<(), String> {
    set_preference(conn, PREFERRED_EXTERNAL_EDITOR_KEY, path.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn open_prefs_db() -> Connection {
        let conn = Connection::open_in_memory().expect("mem");
        ensure_app_preferences_schema(&conn).expect("schema");
        conn
    }

    #[test]
    fn preferencia_editor_default_vacia() {
        let conn = open_prefs_db();
        let v = get_preferred_external_editor(&conn).expect("get");
        assert_eq!(v, "");
    }

    #[test]
    fn preferencia_editor_round_trip() {
        let conn = open_prefs_db();
        set_preferred_external_editor(&conn, r"C:\Editors\code.exe").expect("set");
        let v = get_preferred_external_editor(&conn).expect("get");
        assert_eq!(v, r"C:\Editors\code.exe");
    }

    #[test]
    fn preferencia_editor_trim_al_guardar() {
        let conn = open_prefs_db();
        set_preferred_external_editor(&conn, "  /usr/bin/nvim  ").expect("set");
        assert_eq!(
            get_preferred_external_editor(&conn).unwrap(),
            "/usr/bin/nvim"
        );
    }
}
