CREATE TABLE IF NOT EXISTS role_reactions (
    id INTEGER PRIMARY KEY,
    emoji VARCHAR(10) NOT NULL,
    role_id INTEGER NOT NULL,
    role_name TEXT NOT NULL
)
