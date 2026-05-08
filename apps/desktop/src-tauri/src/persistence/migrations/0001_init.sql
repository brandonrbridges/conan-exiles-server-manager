-- Initial schema. v0 design ships three tables: servers, ban_cache, app_settings.
-- Booleans are stored as INTEGER (SQLite convention) — 0/1.

CREATE TABLE servers (
    id            TEXT PRIMARY KEY,                 -- uuid v4 string
    name          TEXT NOT NULL,
    host          TEXT NOT NULL,
    rcon_port     INTEGER NOT NULL,
    has_admin_pw  INTEGER NOT NULL CHECK (has_admin_pw IN (0, 1)),
    created_at    INTEGER NOT NULL,                 -- unix epoch seconds
    last_used_at  INTEGER
);

CREATE TABLE ban_cache (
    server_id     TEXT NOT NULL,
    player_id     TEXT NOT NULL,
    player_name   TEXT,
    reason        TEXT,
    banned_at     INTEGER,
    PRIMARY KEY (server_id, player_id),
    FOREIGN KEY (server_id) REFERENCES servers(id) ON DELETE CASCADE
);

CREATE TABLE app_settings (
    key    TEXT PRIMARY KEY,
    value  TEXT NOT NULL
);
