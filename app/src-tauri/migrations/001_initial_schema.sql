-- SQLite database schema for NekoSSH Connection Profiles
-- Enables foreign key support
PRAGMA foreign_keys = ON;

-- Table: profiles
CREATE TABLE IF NOT EXISTS profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    host TEXT NOT NULL,
    port INTEGER NOT NULL DEFAULT 22,
    username TEXT NOT NULL,
    keepalive INTEGER NOT NULL DEFAULT 60,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Table: auth_credentials
CREATE TABLE IF NOT EXISTS auth_credentials (
    profile_id INTEGER PRIMARY KEY,
    auth_type TEXT NOT NULL CHECK(auth_type IN ('password', 'key')),
    password TEXT,
    key_path TEXT,
    passphrase TEXT,
    FOREIGN KEY(profile_id) REFERENCES profiles(id) ON DELETE CASCADE
);

-- Table: ssh_tunnels
CREATE TABLE IF NOT EXISTS ssh_tunnels (
    profile_id INTEGER PRIMARY KEY,
    tunnel_type TEXT NOT NULL CHECK(tunnel_type IN ('local', 'dynamic')),
    local_port INTEGER NOT NULL,
    dest TEXT, -- e.g., 'localhost:80' for local tunnels, null for dynamic tunnels
    FOREIGN KEY(profile_id) REFERENCES profiles(id) ON DELETE CASCADE
);

-- Indexes for performance and uniqueness
CREATE INDEX IF NOT EXISTS idx_profiles_name ON profiles(name);
CREATE INDEX IF NOT EXISTS idx_auth_credentials_profile ON auth_credentials(profile_id);
CREATE INDEX IF NOT EXISTS idx_ssh_tunnels_profile ON ssh_tunnels(profile_id);
