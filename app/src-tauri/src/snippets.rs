//! Local snippets dictionary (categories + bodies) for Fase 4a.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetCategory {
    pub id: Option<i64>,
    pub name: String,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: Option<i64>,
    pub category_id: i64,
    pub title: String,
    pub body: String,
    pub sort_order: i64,
}

/// Idempotent schema for rusqlite path + in-memory tests (plugin migration 004 runs once).
pub fn ensure_snippets_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(include_str!("../migrations/004_snippets.sql"))
        .map_err(|e| format!("Error al crear schema snippets: {}", e))?;
    Ok(())
}

pub fn list_categories(conn: &Connection) -> Result<Vec<SnippetCategory>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, sort_order FROM snippet_categories
             ORDER BY sort_order ASC, name COLLATE NOCASE ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(SnippetCategory {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                sort_order: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

pub fn create_category(conn: &Connection, name: &str) -> Result<SnippetCategory, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("El nombre de la categoría no puede estar vacío".into());
    }
    let next_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM snippet_categories",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    conn.execute(
        "INSERT INTO snippet_categories (name, sort_order) VALUES (?1, ?2)",
        params![name, next_order],
    )
    .map_err(|e| format!("Error al crear categoría: {}", e))?;
    let id = conn.last_insert_rowid();
    Ok(SnippetCategory {
        id: Some(id),
        name: name.to_string(),
        sort_order: next_order,
    })
}

pub fn delete_category(conn: &Connection, id: i64) -> Result<(), String> {
    let n = conn
        .execute("DELETE FROM snippet_categories WHERE id = ?1", params![id])
        .map_err(|e| format!("Error al eliminar categoría: {}", e))?;
    if n == 0 {
        return Err("Categoría no encontrada".into());
    }
    Ok(())
}

pub fn list_snippets(
    conn: &Connection,
    category_id: Option<i64>,
    query: Option<&str>,
) -> Result<Vec<Snippet>, String> {
    let q = query.map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty());
    let mut sql = String::from(
        "SELECT id, category_id, title, body, sort_order FROM snippets WHERE 1=1",
    );
    if category_id.is_some() {
        sql.push_str(" AND category_id = ?1");
    }
    sql.push_str(" ORDER BY sort_order ASC, title COLLATE NOCASE ASC");

    let map_row = |row: &rusqlite::Row<'_>| -> rusqlite::Result<Snippet> {
        Ok(Snippet {
            id: Some(row.get(0)?),
            category_id: row.get(1)?,
            title: row.get(2)?,
            body: row.get(3)?,
            sort_order: row.get(4)?,
        })
    };

    let mut out = Vec::new();
    if let Some(cid) = category_id {
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![cid], map_row)
            .map_err(|e| e.to_string())?;
        for r in rows {
            out.push(r.map_err(|e| e.to_string())?);
        }
    } else {
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], map_row).map_err(|e| e.to_string())?;
        for r in rows {
            out.push(r.map_err(|e| e.to_string())?);
        }
    }

    if let Some(ref qq) = q {
        out.retain(|s| {
            s.title.to_lowercase().contains(qq) || s.body.to_lowercase().contains(qq)
        });
    }
    Ok(out)
}

pub fn create_snippet(
    conn: &Connection,
    category_id: i64,
    title: &str,
    body: &str,
) -> Result<Snippet, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("El título no puede estar vacío".into());
    }
    let exists: Option<i64> = conn
        .query_row(
            "SELECT id FROM snippet_categories WHERE id = ?1",
            params![category_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if exists.is_none() {
        return Err("Categoría no encontrada".into());
    }
    let next_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM snippets WHERE category_id = ?1",
            params![category_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    conn.execute(
        "INSERT INTO snippets (category_id, title, body, sort_order) VALUES (?1, ?2, ?3, ?4)",
        params![category_id, title, body, next_order],
    )
    .map_err(|e| format!("Error al crear snippet: {}", e))?;
    Ok(Snippet {
        id: Some(conn.last_insert_rowid()),
        category_id,
        title: title.to_string(),
        body: body.to_string(),
        sort_order: next_order,
    })
}

pub fn update_snippet(
    conn: &Connection,
    id: i64,
    category_id: i64,
    title: &str,
    body: &str,
) -> Result<Snippet, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("El título no puede estar vacío".into());
    }
    let n = conn
        .execute(
            "UPDATE snippets SET category_id = ?1, title = ?2, body = ?3,
             updated_at = datetime('now') WHERE id = ?4",
            params![category_id, title, body, id],
        )
        .map_err(|e| format!("Error al actualizar snippet: {}", e))?;
    if n == 0 {
        return Err("Snippet no encontrado".into());
    }
    Ok(Snippet {
        id: Some(id),
        category_id,
        title: title.to_string(),
        body: body.to_string(),
        sort_order: 0,
    })
}

pub fn delete_snippet(conn: &Connection, id: i64) -> Result<(), String> {
    let n = conn
        .execute("DELETE FROM snippets WHERE id = ?1", params![id])
        .map_err(|e| format!("Error al eliminar snippet: {}", e))?;
    if n == 0 {
        return Err("Snippet no encontrado".into());
    }
    Ok(())
}

/// Seed demo when there are zero categories (idempotent empty⇒seed).
pub fn ensure_snippet_seed(conn: &Connection) -> Result<bool, String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM snippet_categories", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if count > 0 {
        return Ok(false);
    }

    let seed: &[(&str, &[(&str, &str)])] = &[
        (
            "Apache",
            &[
                ("Reiniciar Apache", "sudo systemctl restart apache2"),
                ("Probar configuración", "sudo apachectl configtest"),
                ("Ver error log", "sudo tail -n 50 /var/log/apache2/error.log"),
            ],
        ),
        (
            "Tomcat",
            &[
                ("Status Tomcat", "sudo systemctl status tomcat"),
                ("Tail catalina.out", "sudo tail -n 100 $CATALINA_HOME/logs/catalina.out"),
                ("Hint deploy", "ls -la $CATALINA_HOME/webapps"),
            ],
        ),
        (
            "Permisos",
            &[
                ("chmod 755", "chmod 755 path/to/file"),
                ("chown www-data", "sudo chown -R www-data:www-data /var/www/html"),
                ("ls detallado", "ls -la"),
            ],
        ),
    ];

    for (i, (cat_name, items)) in seed.iter().enumerate() {
        conn.execute(
            "INSERT INTO snippet_categories (name, sort_order) VALUES (?1, ?2)",
            params![*cat_name, i as i64],
        )
        .map_err(|e| e.to_string())?;
        let cid = conn.last_insert_rowid();
        for (j, (title, body)) in items.iter().enumerate() {
            conn.execute(
                "INSERT INTO snippets (category_id, title, body, sort_order) VALUES (?1, ?2, ?3, ?4)",
                params![cid, *title, *body, j as i64],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_db() -> Connection {
        let conn = Connection::open_in_memory().expect("mem");
        ensure_snippets_schema(&conn).expect("schema");
        conn
    }

    #[test]
    fn schema_y_crud_cascade() {
        let conn = open_db();
        let cat = create_category(&conn, "Apache").expect("cat");
        let cid = cat.id.expect("id");
        let sn = create_snippet(&conn, cid, "test", "echo hi").expect("snip");
        assert!(sn.id.is_some());
        let list = list_snippets(&conn, Some(cid), None).expect("list");
        assert_eq!(list.len(), 1);
        delete_category(&conn, cid).expect("del cat");
        let list2 = list_snippets(&conn, None, None).expect("list2");
        assert!(list2.is_empty());
    }

    #[test]
    fn seed_solo_si_vacio() {
        let conn = open_db();
        assert!(ensure_snippet_seed(&conn).expect("seed1"));
        assert!(!ensure_snippet_seed(&conn).expect("seed2"));
        let cats = list_categories(&conn).expect("cats");
        assert_eq!(cats.len(), 3);
    }

    #[test]
    fn filtro_query() {
        let conn = open_db();
        let cat = create_category(&conn, "X").unwrap();
        let cid = cat.id.unwrap();
        create_snippet(&conn, cid, "reiniciar", "systemctl restart").unwrap();
        create_snippet(&conn, cid, "otro", "ls").unwrap();
        let found = list_snippets(&conn, None, Some("restart")).unwrap();
        assert_eq!(found.len(), 1);
    }
}
