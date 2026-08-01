-- Connection folders (one level) + profiles.folder_id
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS connection_folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_connection_folders_sort ON connection_folders(sort_order);

INSERT INTO connection_folders (name, sort_order)
SELECT 'General', 0
WHERE NOT EXISTS (SELECT 1 FROM connection_folders WHERE name = 'General');

ALTER TABLE profiles ADD COLUMN folder_id INTEGER REFERENCES connection_folders(id) ON DELETE CASCADE;

UPDATE profiles
SET folder_id = (SELECT id FROM connection_folders WHERE name = 'General' LIMIT 1)
WHERE folder_id IS NULL;
