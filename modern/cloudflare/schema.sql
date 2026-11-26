CREATE TABLE IF NOT EXISTS usage_events (
 id INTEGER PRIMARY KEY, event TEXT NOT NULL CHECK(event IN ('app_loaded','cloak_completed','decloak_completed','cloak_failed','decloak_failed')), tier TEXT CHECK(tier IN ('small','medium','large')), country TEXT, created_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE TABLE IF NOT EXISTS nps_feedback (
 id INTEGER PRIMARY KEY, score INTEGER NOT NULL CHECK(score BETWEEN 0 AND 10), comment TEXT NOT NULL, created_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE TABLE IF NOT EXISTS licensing_inquiries (
 id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL, organization TEXT, message TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'new', created_utc TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
