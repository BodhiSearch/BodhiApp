-- Add up migration script here

-- Add forward_all_with_prefix column to api_model_aliases table
ALTER TABLE api_model_aliases ADD COLUMN forward_all_with_prefix BOOLEAN NOT NULL DEFAULT 0;

-- Add unique constraint on non-null prefix (allows multiple NULL/empty values)
CREATE UNIQUE INDEX idx_api_model_aliases_prefix_unique
ON api_model_aliases(prefix)
WHERE prefix IS NOT NULL AND prefix != '';
