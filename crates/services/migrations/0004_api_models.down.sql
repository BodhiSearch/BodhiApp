-- Add down migration script here

-- Drop indexes first
DROP INDEX IF EXISTS idx_api_model_aliases_updated_at;
DROP INDEX IF EXISTS idx_api_model_aliases_prefix;
DROP INDEX IF EXISTS idx_api_model_aliases_api_format;

-- Drop the api_model_aliases table
DROP TABLE IF EXISTS api_model_aliases;