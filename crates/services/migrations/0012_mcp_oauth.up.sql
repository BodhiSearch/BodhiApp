CREATE TABLE IF NOT EXISTS mcp_oauth_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL DEFAULT 'OAuth',          -- human-readable display name
    mcp_server_id TEXT NOT NULL REFERENCES mcp_servers(id),
    registration_type TEXT NOT NULL DEFAULT 'pre-registered',
    client_id TEXT NOT NULL,
    encrypted_client_secret TEXT,
    client_secret_salt TEXT,
    client_secret_nonce TEXT,
    authorization_endpoint TEXT NOT NULL,
    token_endpoint TEXT NOT NULL,
    registration_endpoint TEXT,
    encrypted_registration_access_token TEXT,
    registration_access_token_salt TEXT,
    registration_access_token_nonce TEXT,
    client_id_issued_at INTEGER,
    token_endpoint_auth_method TEXT,
    scopes TEXT,
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS mcp_oauth_tokens (
    id TEXT PRIMARY KEY,
    mcp_oauth_config_id TEXT NOT NULL REFERENCES mcp_oauth_configs(id),
    encrypted_access_token TEXT NOT NULL,
    access_token_salt TEXT NOT NULL,
    access_token_nonce TEXT NOT NULL,
    encrypted_refresh_token TEXT,
    refresh_token_salt TEXT,
    refresh_token_nonce TEXT,
    scopes_granted TEXT,
    expires_at INTEGER,
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_mcp_oauth_configs_server ON mcp_oauth_configs(mcp_server_id);
CREATE INDEX IF NOT EXISTS idx_mcp_oauth_tokens_config ON mcp_oauth_tokens(mcp_oauth_config_id);

-- Unique name per server (case-insensitive)
CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_oauth_configs_server_name ON mcp_oauth_configs(mcp_server_id, name COLLATE NOCASE);
