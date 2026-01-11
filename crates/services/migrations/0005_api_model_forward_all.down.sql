-- Add down migration script here

-- Drop unique index on prefix
DROP INDEX IF EXISTS idx_api_model_aliases_prefix_unique;

-- SQLite doesn't support DROP COLUMN, so we recreate the table without new columns
-- Create backup table with original columns only
CREATE TABLE api_model_aliases_backup AS
SELECT id, api_format, prefix, base_url, models_json, encrypted_api_key, salt, nonce, created_at, updated_at
FROM api_model_aliases;

-- Drop original table
DROP TABLE api_model_aliases;

-- Recreate table with original schema (from 0004 migration)
CREATE TABLE api_model_aliases (
    id TEXT PRIMARY KEY NOT NULL,
    api_format TEXT NOT NULL,
    prefix TEXT,
    base_url TEXT NOT NULL,
    models_json TEXT NOT NULL,
    encrypted_api_key TEXT,
    salt TEXT,
    nonce TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Restore data from backup
INSERT INTO api_model_aliases SELECT * FROM api_model_aliases_backup;

-- Drop backup table
DROP TABLE api_model_aliases_backup;

-- Recreate original indexes
CREATE INDEX idx_api_model_aliases_api_format ON api_model_aliases(api_format);
CREATE INDEX idx_api_model_aliases_prefix ON api_model_aliases(prefix);
CREATE INDEX idx_api_model_aliases_updated_at ON api_model_aliases(updated_at);
