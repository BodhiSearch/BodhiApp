-- Migration 0007 down: Remove toolset configuration tables

-- Drop app_toolset_configs indexes and table first
DROP INDEX IF EXISTS idx_app_toolset_configs_toolset_id;
DROP TABLE IF EXISTS app_toolset_configs;

-- Drop toolsets indexes
DROP INDEX IF EXISTS idx_toolsets_user_type;
DROP INDEX IF EXISTS idx_toolsets_toolset_type;
DROP INDEX IF EXISTS idx_toolsets_user_id;

-- Drop the toolsets table
DROP TABLE IF EXISTS toolsets;
