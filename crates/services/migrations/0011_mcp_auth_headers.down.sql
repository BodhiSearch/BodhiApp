-- Migration 0011 down: Remove auth columns from mcps table
-- SQLite doesn't support DROP COLUMN before 3.35.0, so recreate the table

CREATE TABLE mcps_backup AS SELECT
  id, user_id, mcp_server_id, name, slug, description, enabled,
  tools_cache, tools_filter, created_at, updated_at
FROM mcps;

DROP TABLE mcps;

CREATE TABLE mcps (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    mcp_server_id TEXT NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    tools_cache TEXT,
    tools_filter TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(user_id, slug COLLATE NOCASE)
);

INSERT INTO mcps SELECT * FROM mcps_backup;
DROP TABLE mcps_backup;

CREATE INDEX IF NOT EXISTS idx_mcps_user_id ON mcps(user_id);
CREATE INDEX IF NOT EXISTS idx_mcps_mcp_server_id ON mcps(mcp_server_id);
