-- Migration 0008: App Client Toolset Configs
-- Caches app-client toolset configurations from Keycloak /resources/request-access endpoint
-- Used for external app toolset authorization

CREATE TABLE IF NOT EXISTS app_client_toolset_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_client_id TEXT NOT NULL UNIQUE,
    config_version TEXT,
    toolsets_json TEXT NOT NULL,  -- JSON array: [{"id":"...","scope":"..."}]
    resource_scope TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_app_client_toolset_configs_client_id ON app_client_toolset_configs(app_client_id);
