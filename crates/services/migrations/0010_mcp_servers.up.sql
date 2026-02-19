-- Migration 0010: MCP server URL allowlist and user MCP instances
-- Creates tables for admin-managed MCP server URLs and user-owned MCP instances

-- Create the mcp_servers table for admin-managed MCP server registry
CREATE TABLE IF NOT EXISTS mcp_servers (
    id TEXT PRIMARY KEY,                   -- UUID as TEXT (consistent with other tables)
    url TEXT NOT NULL,                     -- MCP server endpoint URL, trimmed whitespace
    name TEXT NOT NULL DEFAULT '',         -- human-readable display name
    description TEXT,                      -- optional description
    enabled INTEGER NOT NULL DEFAULT 0,    -- boolean as integer
    created_by TEXT NOT NULL,              -- user_id of admin who created
    updated_by TEXT NOT NULL,              -- user_id of admin who last updated
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL            -- Unix timestamp
);

-- Case-insensitive unique index on url (URLs are case-insensitive per RFC)
CREATE UNIQUE INDEX IF NOT EXISTS idx_mcp_servers_url ON mcp_servers(url COLLATE NOCASE);

-- Create the mcps table for user-owned MCP instances
CREATE TABLE IF NOT EXISTS mcps (
    id TEXT PRIMARY KEY,                   -- UUID as TEXT
    created_by TEXT NOT NULL,              -- JWT 'sub' claim of user who created (no FK)
    mcp_server_id TEXT NOT NULL,           -- FK to mcp_servers.id (link, not URL copy)
    name TEXT NOT NULL,                    -- human-readable name
    slug TEXT NOT NULL,                    -- user-defined instance slug
    description TEXT,                      -- optional instance description
    enabled INTEGER NOT NULL DEFAULT 1,    -- boolean as integer
    tools_cache TEXT,                      -- JSON array of tool schemas
    tools_filter TEXT,                     -- JSON array of whitelisted tool names
    auth_type TEXT NOT NULL DEFAULT 'public', -- auth type: 'public', 'header', 'oauth'
    auth_uuid TEXT,                        -- FK to auth config table (resolved by auth_type)
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL,           -- Unix timestamp
    UNIQUE(created_by, slug COLLATE NOCASE) -- case-insensitive uniqueness per user
);

-- Create index on created_by for faster lookups by user
CREATE INDEX IF NOT EXISTS idx_mcps_created_by ON mcps(created_by);

-- Create index on mcp_server_id for faster joins
CREATE INDEX IF NOT EXISTS idx_mcps_mcp_server_id ON mcps(mcp_server_id);
