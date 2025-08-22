ALTER TABLE download_requests ADD COLUMN total_bytes INTEGER;
ALTER TABLE download_requests ADD COLUMN downloaded_bytes INTEGER DEFAULT 0;
ALTER TABLE download_requests ADD COLUMN started_at INTEGER;