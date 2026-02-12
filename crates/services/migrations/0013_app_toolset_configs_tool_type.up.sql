-- Add toolset_type column to app_toolset_configs
ALTER TABLE app_toolset_configs ADD COLUMN toolset_type TEXT;

-- Migrate existing data: scope_toolset-builtin-exa-web-search -> builtin-exa-search
UPDATE app_toolset_configs
SET toolset_type = 'builtin-exa-search'
WHERE scope = 'scope_toolset-builtin-exa-web-search';

-- Drop old scope_uuid column and its index
DROP INDEX IF EXISTS idx_app_toolset_configs_scope_uuid;
ALTER TABLE app_toolset_configs DROP COLUMN scope_uuid;

-- Create index on toolset_type for lookups
CREATE INDEX IF NOT EXISTS idx_app_toolset_configs_toolset_type ON app_toolset_configs(toolset_type);
