-- Migration 0011: Create mcp_auth_headers table for header-based MCP authentication
-- Stores encrypted header key/value pairs referenced by mcps.auth_uuid when auth_type = 'header'
-- Auth headers are admin-managed children of mcp_servers

CREATE TABLE IF NOT EXISTS mcp_auth_headers (
    id TEXT PRIMARY KEY,                       -- UUID as TEXT
    name TEXT NOT NULL DEFAULT 'Header',        -- human-readable display name
    mcp_server_id TEXT NOT NULL REFERENCES mcp_servers(id), -- FK to parent server
    header_key TEXT NOT NULL,                  -- HTTP header name (e.g. 'Authorization', 'X-API-Key')
    encrypted_header_value TEXT NOT NULL,       -- AES-256-GCM encrypted header value
    header_value_salt TEXT NOT NULL,            -- Salt for key derivation
    header_value_nonce TEXT NOT NULL,           -- Nonce for AES-256-GCM
    created_by TEXT NOT NULL,                  -- user_id who created this config
    created_at INTEGER NOT NULL,               -- Unix timestamp
    updated_at INTEGER NOT NULL                -- Unix timestamp
);

-- Unique name per server (case-insensitive)
CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_auth_headers_server_name ON mcp_auth_headers(mcp_server_id, name COLLATE NOCASE);

-- Index for listing auth headers by server
CREATE INDEX IF NOT EXISTS idx_mcp_auth_headers_server ON mcp_auth_headers(mcp_server_id);
