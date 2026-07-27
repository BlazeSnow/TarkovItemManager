CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL COLLATE NOCASE UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token_hash TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS sessions_token_hash_index ON sessions(token_hash);

CREATE TABLE IF NOT EXISTS facility_levels (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    facility_id TEXT NOT NULL,
    level INTEGER NOT NULL CHECK (level >= 0),
    PRIMARY KEY (user_id, facility_id)
);

CREATE TABLE IF NOT EXISTS checked_materials (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL,
    PRIMARY KEY (user_id, item_id)
);
