-- Migration 0011: Create mcp_auth_headers table for header-based MCP authentication
-- Stores encrypted header key/value pairs referenced by mcps.auth_uuid when auth_type = 'header'

CREATE TABLE IF NOT EXISTS mcp_auth_headers (
    id TEXT PRIMARY KEY,                       -- UUID as TEXT
    header_key TEXT NOT NULL,                  -- HTTP header name (e.g. 'Authorization', 'X-API-Key')
    encrypted_header_value TEXT NOT NULL,       -- AES-256-GCM encrypted header value
    header_value_salt TEXT NOT NULL,            -- Salt for key derivation
    header_value_nonce TEXT NOT NULL,           -- Nonce for AES-256-GCM
    created_by TEXT NOT NULL,                  -- user_id who created this config
    created_at INTEGER NOT NULL,               -- Unix timestamp
    updated_at INTEGER NOT NULL                -- Unix timestamp
);
