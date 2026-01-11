-- Add down migration script here

-- Drop unique index on prefix
DROP INDEX idx_api_model_aliases_prefix_unique;

-- Note: SQLite doesn't support DROP COLUMN easily
-- For complete rollback, would need to recreate table without forward_all_with_prefix
-- In production, consider keeping the column for backward compatibility
