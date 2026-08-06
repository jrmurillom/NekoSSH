-- Theme wallpapers: metadata in SQLite, image bytes on disk under app_data/wallpapers/
CREATE TABLE IF NOT EXISTS theme_wallpapers (
    theme_id TEXT PRIMARY KEY NOT NULL,
    label TEXT NOT NULL DEFAULT '',
    opacity REAL NOT NULL DEFAULT 0.3,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('file', 'url')),
    file_name TEXT,
    remote_url TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
