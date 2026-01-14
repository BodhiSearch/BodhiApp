-- Add down migration script here

-- Drop app_tool_configs indexes and table first
DROP INDEX IF EXISTS idx_app_tool_configs_tool_id;
DROP TABLE IF EXISTS app_tool_configs;

-- Drop user_tool_configs indexes
DROP INDEX IF EXISTS idx_user_tool_configs_enabled;
DROP INDEX IF EXISTS idx_user_tool_configs_tool_id;
DROP INDEX IF EXISTS idx_user_tool_configs_user_id;

-- Drop the user_tool_configs table
DROP TABLE IF EXISTS user_tool_configs;
