-- Migration 0012: Create OAuth tables for MCP OAuth 2.1 pre-registered client authentication
-- mcp_oauth_configs stores encrypted client credentials and endpoint URLs
-- mcp_oauth_tokens stores encrypted access/refresh tokens per config

CREATE TABLE IF NOT EXISTS mcp_oauth_configs (
    id TEXT PRIMARY KEY,                       -- UUID as TEXT
    mcp_server_id TEXT NOT NULL REFERENCES mcp_servers(id), -- FK to mcp_servers
    client_id TEXT NOT NULL,                   -- OAuth client_id
    encrypted_client_secret TEXT NOT NULL,      -- AES-256-GCM encrypted client_secret
    client_secret_salt TEXT NOT NULL,           -- Salt for key derivation
    client_secret_nonce TEXT NOT NULL,          -- Nonce for AES-256-GCM
    authorization_endpoint TEXT NOT NULL,       -- OAuth authorization URL
    token_endpoint TEXT NOT NULL,               -- OAuth token URL
    scopes TEXT,                               -- Requested scopes (space-separated)
    created_by TEXT NOT NULL,                  -- user_id who created this config
    created_at INTEGER NOT NULL,               -- Unix timestamp
    updated_at INTEGER NOT NULL                -- Unix timestamp
);

CREATE TABLE IF NOT EXISTS mcp_oauth_tokens (
    id TEXT PRIMARY KEY,                       -- UUID as TEXT
    mcp_oauth_config_id TEXT NOT NULL REFERENCES mcp_oauth_configs(id),
    encrypted_access_token TEXT NOT NULL,       -- AES-256-GCM encrypted access token
    access_token_salt TEXT NOT NULL,            -- Salt for key derivation
    access_token_nonce TEXT NOT NULL,           -- Nonce for AES-256-GCM
    encrypted_refresh_token TEXT,               -- AES-256-GCM encrypted refresh token (nullable)
    refresh_token_salt TEXT,                    -- Salt for key derivation (nullable)
    refresh_token_nonce TEXT,                   -- Nonce for AES-256-GCM (nullable)
    scopes_granted TEXT,                       -- Granted scopes from token response (space-separated)
    expires_at INTEGER,                        -- Unix timestamp for token expiry
    created_by TEXT NOT NULL,                  -- user_id who obtained this token
    created_at INTEGER NOT NULL,               -- Unix timestamp
    updated_at INTEGER NOT NULL                -- Unix timestamp
);

CREATE INDEX idx_mcp_oauth_configs_server ON mcp_oauth_configs(mcp_server_id);
CREATE INDEX idx_mcp_oauth_tokens_config ON mcp_oauth_tokens(mcp_oauth_config_id);
