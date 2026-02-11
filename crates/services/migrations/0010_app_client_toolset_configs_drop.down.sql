-- Recreate table from migration 0008 for rollback
CREATE TABLE IF NOT EXISTS app_client_toolset_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_client_id TEXT NOT NULL UNIQUE,
    config_version TEXT,
    toolsets_json TEXT NOT NULL,
    resource_scope TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
