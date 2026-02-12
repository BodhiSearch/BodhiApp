-- Restore scope column with UNIQUE constraint (requires table rebuild in SQLite)
-- Create temp table with scope column
CREATE TABLE app_toolset_configs_temp (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scope TEXT NOT NULL UNIQUE,
    scope_uuid TEXT NOT NULL,
    toolset_type TEXT,
    enabled INTEGER NOT NULL DEFAULT 0,
    updated_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Copy data back, restoring scope from toolset_type
INSERT INTO app_toolset_configs_temp (id, scope, scope_uuid, toolset_type, enabled, updated_by, created_at, updated_at)
SELECT id, toolset_type, '', toolset_type, enabled, updated_by, created_at, updated_at
FROM app_toolset_configs;

-- Drop new table
DROP TABLE app_toolset_configs;

-- Rename temp table
ALTER TABLE app_toolset_configs_temp RENAME TO app_toolset_configs;

-- Restore indices
CREATE INDEX IF NOT EXISTS idx_app_toolset_configs_scope ON app_toolset_configs(scope);
CREATE INDEX IF NOT EXISTS idx_app_toolset_configs_scope_uuid ON app_toolset_configs(scope_uuid);

-- Drop toolset_type column (keeping it in table but we could drop if needed)
-- Note: toolset_type is kept in the rollback for data preservation
