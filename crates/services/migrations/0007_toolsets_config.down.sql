-- Migration 0007 down: Remove toolset configuration tables

-- Drop app_toolset_configs indexes and table first
DROP INDEX IF EXISTS idx_app_toolset_configs_scope_uuid;
DROP INDEX IF EXISTS idx_app_toolset_configs_scope;
DROP TABLE IF EXISTS app_toolset_configs;

-- Drop toolsets indexes
DROP INDEX IF EXISTS idx_toolsets_user_scope_uuid;
DROP INDEX IF EXISTS idx_toolsets_scope_uuid;
DROP INDEX IF EXISTS idx_toolsets_user_id;

-- Drop the toolsets table
DROP TABLE IF EXISTS toolsets;
