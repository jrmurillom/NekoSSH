-- Rename key_path → private_key (contenido PEM; sin migración de rutas).
ALTER TABLE auth_credentials RENAME COLUMN key_path TO private_key;
