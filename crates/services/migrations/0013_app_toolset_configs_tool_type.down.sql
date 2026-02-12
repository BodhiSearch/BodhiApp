-- Restore scope_uuid column
ALTER TABLE app_toolset_configs ADD COLUMN scope_uuid TEXT;

-- Drop toolset_type column and its index
DROP INDEX IF EXISTS idx_app_toolset_configs_toolset_type;
ALTER TABLE app_toolset_configs DROP COLUMN toolset_type;

-- Restore scope_uuid index
CREATE INDEX IF NOT EXISTS idx_app_toolset_configs_scope_uuid ON app_toolset_configs(scope_uuid);
