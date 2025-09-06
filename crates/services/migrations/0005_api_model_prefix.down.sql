-- Rollback migration for prefix support
-- Remove the prefix column and associated index

-- Remove prefix index
DROP INDEX IF EXISTS idx_api_model_aliases_prefix;

-- Remove prefix column
ALTER TABLE api_model_aliases DROP COLUMN prefix;