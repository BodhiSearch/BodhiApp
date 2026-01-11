-- Add up migration script here

-- Add forward_all_with_prefix column to api_model_aliases table
ALTER TABLE api_model_aliases ADD COLUMN forward_all_with_prefix BOOLEAN NOT NULL DEFAULT 0;

-- Add models_cache_json column for storing cached models from remote API
ALTER TABLE api_model_aliases ADD COLUMN models_cache_json TEXT;

-- Add cache_fetched_at column for tracking cache freshness (epoch = never fetched)
ALTER TABLE api_model_aliases ADD COLUMN cache_fetched_at DATETIME NOT NULL DEFAULT '1970-01-01 00:00:00';

-- Add unique constraint on non-null prefix (allows multiple NULL/empty values)
CREATE UNIQUE INDEX idx_api_model_aliases_prefix_unique
ON api_model_aliases(prefix)
WHERE prefix IS NOT NULL AND prefix != '';
