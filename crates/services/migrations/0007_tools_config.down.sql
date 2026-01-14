-- Add down migration script here

-- Drop indexes first
DROP INDEX IF EXISTS idx_user_tool_configs_enabled;
DROP INDEX IF EXISTS idx_user_tool_configs_tool_id;
DROP INDEX IF EXISTS idx_user_tool_configs_user_id;

-- Drop the table
DROP TABLE IF EXISTS user_tool_configs;
