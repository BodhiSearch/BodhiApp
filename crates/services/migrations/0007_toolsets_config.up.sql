-- Migration 0007: Toolset Configuration Tables
-- Creates tables for user toolset instances and app-level toolset configuration

-- Create the toolsets table for user-owned toolset instances
CREATE TABLE IF NOT EXISTS toolsets (
    id TEXT PRIMARY KEY,                   -- UUID as TEXT
    user_id TEXT NOT NULL,                 -- JWT 'sub' claim (no FK)
    toolset_type TEXT,                     -- Toolset type identifier (e.g., 'builtin-exa-search')
    slug TEXT NOT NULL,                    -- user-defined instance slug
    description TEXT,                      -- optional instance description
    enabled INTEGER NOT NULL DEFAULT 0,    -- boolean as integer
    -- Encrypted API key storage (same pattern as api_model_aliases)
    encrypted_api_key TEXT,
    salt TEXT,
    nonce TEXT,
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL,
    UNIQUE(user_id, slug COLLATE NOCASE)   -- case-insensitive uniqueness per user
);

-- Create index on user_id for faster lookups by user
CREATE INDEX IF NOT EXISTS idx_toolsets_user_id ON toolsets(user_id);

-- Create index on toolset_type for faster lookups by type
CREATE INDEX IF NOT EXISTS idx_toolsets_toolset_type ON toolsets(toolset_type);

-- Create composite index on user_id and toolset_type for efficient filtering
CREATE INDEX IF NOT EXISTS idx_toolsets_user_toolset_type ON toolsets(user_id, toolset_type);

-- Create the app_toolset_configs table for app-level toolset configuration (admin-controlled)
CREATE TABLE IF NOT EXISTS app_toolset_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    toolset_type TEXT NOT NULL,            -- Toolset type identifier (e.g., 'builtin-exa-search')
    enabled INTEGER NOT NULL DEFAULT 0,    -- boolean as integer
    updated_by TEXT NOT NULL,              -- user_id of admin who last updated
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL
);

-- Create UNIQUE index on toolset_type
CREATE UNIQUE INDEX idx_app_toolset_configs_toolset_type ON app_toolset_configs(toolset_type);
