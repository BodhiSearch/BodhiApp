-- Add toolset_type column
ALTER TABLE toolsets ADD COLUMN toolset_type TEXT;

-- Migrate existing data for builtin-exa-search
-- Dev environment scope_uuid
UPDATE toolsets SET toolset_type = 'builtin-exa-search'
WHERE scope_uuid = '4ff0e163-36fb-47d6-a5ef-26e396f067d6';

-- Prod environment scope_uuid
UPDATE toolsets SET toolset_type = 'builtin-exa-search'
WHERE scope_uuid = '7a89e236-9d23-4856-aa77-b52823ff9972';

-- Drop all indexes that reference scope_uuid before dropping the column
DROP INDEX IF EXISTS idx_toolsets_scope_uuid;
DROP INDEX IF EXISTS idx_toolsets_user_scope_uuid;

-- SQLite 3.35.0+ supports DROP COLUMN
-- For older versions, this would require CREATE TABLE AS SELECT pattern
ALTER TABLE toolsets DROP COLUMN scope_uuid;

-- Create new indexes on toolset_type
CREATE INDEX IF NOT EXISTS idx_toolsets_toolset_type ON toolsets(toolset_type);
CREATE INDEX IF NOT EXISTS idx_toolsets_user_toolset_type ON toolsets(user_id, toolset_type);
