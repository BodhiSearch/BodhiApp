-- Migration 0007 down: Remove toolset configuration tables

-- Drop app_toolset_configs indexes and table first
DROP INDEX IF EXISTS idx_app_toolset_configs_toolset_id;
DROP TABLE IF EXISTS app_toolset_configs;

-- Drop user_toolset_configs indexes
DROP INDEX IF EXISTS idx_user_toolset_configs_enabled;
DROP INDEX IF EXISTS idx_user_toolset_configs_toolset_id;
DROP INDEX IF EXISTS idx_user_toolset_configs_user_id;

-- Drop the user_toolset_configs table
DROP TABLE IF EXISTS user_toolset_configs;
