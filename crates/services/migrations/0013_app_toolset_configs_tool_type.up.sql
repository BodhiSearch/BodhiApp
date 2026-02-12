-- SQLite doesn't allow dropping columns with UNIQUE constraints
-- Rebuild table without scope and scope_uuid columns

CREATE TABLE app_toolset_configs_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    toolset_type TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 0,
    updated_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Copy data, migrating scope to toolset_type
INSERT INTO app_toolset_configs_new (id, toolset_type, enabled, updated_by, created_at, updated_at)
SELECT
    id,
    CASE
        WHEN scope = 'scope_toolset-builtin-exa-web-search' THEN 'builtin-exa-search'
        ELSE REPLACE(scope, 'scope_toolset-', '')
    END,
    enabled,
    updated_by,
    created_at,
    updated_at
FROM app_toolset_configs;

DROP TABLE app_toolset_configs;
ALTER TABLE app_toolset_configs_new RENAME TO app_toolset_configs;

-- Create UNIQUE index on toolset_type
CREATE UNIQUE INDEX idx_app_toolset_configs_toolset_type ON app_toolset_configs(toolset_type);
