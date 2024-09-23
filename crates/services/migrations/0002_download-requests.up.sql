CREATE TABLE IF NOT EXISTS download_requests (
    id TEXT PRIMARY KEY,
    repo TEXT NOT NULL,
    filename TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_download_requests_status ON download_requests(status);
