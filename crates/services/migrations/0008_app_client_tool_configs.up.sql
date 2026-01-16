-- Migration 0008: App Client Tool Configs
-- Caches app-client tool configurations from Keycloak /resources/request-access endpoint
-- Used for external app tool authorization

CREATE TABLE IF NOT EXISTS app_client_tool_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_client_id TEXT NOT NULL UNIQUE,
    config_version TEXT NOT NULL,
    tools_json TEXT NOT NULL,  -- JSON array: [{"tool_id":"...","tool_scope":"..."}]
    resource_scope TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_app_client_tool_configs_client_id ON app_client_tool_configs(app_client_id);
