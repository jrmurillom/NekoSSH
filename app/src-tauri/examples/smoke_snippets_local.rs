//! Local-only smoke for snippets CRUD/seed (no live SSH).
use app_lib::snippets::{
    create_category, create_snippet, delete_category, ensure_snippet_seed, ensure_snippets_schema,
    list_categories, list_snippets,
};
use rusqlite::Connection;

fn main() {
    let conn = Connection::open_in_memory().expect("mem");
    ensure_snippets_schema(&conn).expect("schema");
    assert!(ensure_snippet_seed(&conn).expect("seed"));
    assert!(!ensure_snippet_seed(&conn).expect("seed2"));
    let cats = list_categories(&conn).expect("cats");
    assert_eq!(cats.len(), 3);
    let all = list_snippets(&conn, None, None).expect("list");
    assert!(all.len() >= 9);
    let cat = create_category(&conn, "Lab").expect("cat");
    let cid = cat.id.expect("id");
    create_snippet(&conn, cid, "hi", "echo hi").expect("snip");
    delete_category(&conn, cid).expect("cascade");
    let left = list_snippets(&conn, Some(cid), None).expect("gone");
    assert!(left.is_empty());
    println!("smoke_snippets_local PASS");
}