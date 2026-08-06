//! Notes persistence: SQLite storage for user Markdown notes.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteDto {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,
    pub updated_at: Option<String>,
}

pub fn ensure_notes_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(include_str!("../migrations/007_notes.sql"))
        .map_err(|e| format!("Error al crear schema notes: {}", e))?;
    Ok(())
}

pub fn get_notes(conn: &Connection) -> Result<Vec<NoteDto>, String> {
    let mut stmt = conn
        .prepare("SELECT id, title, content, datetime(updated_at, 'localtime') FROM notes ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(NoteDto {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                content: row.get(2)?,
                updated_at: Some(row.get(3)?),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row.map_err(|e| e.to_string())?);
    }
    Ok(list)
}

pub fn create_note(conn: &Connection, title: &str, content: &str) -> Result<NoteDto, String> {
    conn.execute(
        "INSERT INTO notes (title, content, updated_at) VALUES (?, ?, datetime('now'))",
        params![title, content],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();

    let note = conn
        .query_row(
            "SELECT id, title, content, datetime(updated_at, 'localtime') FROM notes WHERE id = ?",
            params![id],
            |row| {
                Ok(NoteDto {
                    id: Some(row.get(0)?),
                    title: row.get(1)?,
                    content: row.get(2)?,
                    updated_at: Some(row.get(3)?),
                })
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(note)
}

pub fn update_note(conn: &Connection, id: i64, title: &str, content: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE notes SET title = ?, content = ?, updated_at = datetime('now') WHERE id = ?",
        params![title, content, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_note(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM notes WHERE id = ?", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
