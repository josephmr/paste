CREATE TABLE pastes (
    id TEXT PRIMARY KEY NOT NULL,
    paste TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP NOT NULL
);