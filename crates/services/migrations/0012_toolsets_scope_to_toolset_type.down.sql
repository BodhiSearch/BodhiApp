-- Drop the toolset_type indexes
DROP INDEX IF EXISTS idx_toolsets_toolset_type;
DROP INDEX IF EXISTS idx_toolsets_user_toolset_type;

-- Restore scope_uuid column
ALTER TABLE toolsets ADD COLUMN scope_uuid TEXT;

-- Reverse migration: builtin-exa-search → scope_uuid
-- Dev environment
UPDATE toolsets SET scope_uuid = '4ff0e163-36fb-47d6-a5ef-26e396f067d6'
WHERE toolset_type = 'builtin-exa-search';

-- Note: This assumes all existing rows are dev environment.
-- In prod, this migration would need conditional logic based on deployment context.

-- Drop toolset_type column
ALTER TABLE toolsets DROP COLUMN toolset_type;

-- Restore the scope_uuid indexes
CREATE INDEX IF NOT EXISTS idx_toolsets_scope_uuid ON toolsets(scope_uuid);
CREATE INDEX IF NOT EXISTS idx_toolsets_user_scope_uuid ON toolsets(user_id, scope_uuid);
