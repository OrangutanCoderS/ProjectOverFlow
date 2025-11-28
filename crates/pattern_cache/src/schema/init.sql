CREATE TABLE IF NOT EXISTS patterns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    score REAL NOT NULL,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_patterns_category
    ON patterns(category);

CREATE INDEX IF NOT EXISTS idx_patterns_score
    ON patterns(score);
