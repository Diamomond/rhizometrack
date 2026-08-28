CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    icon TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    category_id INTEGER NOT NULL,
    session_name TEXT NOT NULL DEFAULT '',

    started_at TEXT NOT NULL,
    ended_at TEXT NOT NULL,

    duration_seconds INTEGER NOT NULL,

    note_markdown TEXT NOT NULL DEFAULT '',

    FOREIGN KEY(category_id)
        REFERENCES categories(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sessions_category
ON sessions(category_id);

CREATE INDEX IF NOT EXISTS idx_sessions_started
ON sessions(started_at);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
