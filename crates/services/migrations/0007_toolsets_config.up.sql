-- Migration 0007: Toolset Configuration Tables
-- Creates tables for user and app-level toolset configuration

-- Create the user_toolset_configs table for per-user toolset configuration
CREATE TABLE IF NOT EXISTS user_toolset_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,                 -- JWT 'sub' claim (no FK)
    toolset_id TEXT NOT NULL,              -- e.g., "builtin-exa-web-search"
    enabled INTEGER NOT NULL DEFAULT 0,    -- boolean as integer
    -- Encrypted API key storage (same pattern as api_model_aliases)
    encrypted_api_key TEXT,
    salt TEXT,
    nonce TEXT,
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL,
    UNIQUE(user_id, toolset_id)            -- composite unique constraint
);

-- Create index on user_id for faster lookups by user
CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_user_id ON user_toolset_configs(user_id);

-- Create index on toolset_id for faster lookups by toolset
CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_toolset_id ON user_toolset_configs(toolset_id);

-- Create index on enabled for filtering enabled toolsets
CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_enabled ON user_toolset_configs(enabled);

-- Create the app_toolset_configs table for app-level toolset configuration (admin-controlled)
CREATE TABLE IF NOT EXISTS app_toolset_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    toolset_id TEXT NOT NULL UNIQUE,       -- e.g., "builtin-exa-web-search"
    enabled INTEGER NOT NULL DEFAULT 0,    -- boolean as integer
    updated_by TEXT NOT NULL,              -- user_id of admin who last updated
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL
);

-- Create index on toolset_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_app_toolset_configs_toolset_id ON app_toolset_configs(toolset_id);

-- Seed default toolset config (web search enabled by default for setup)
INSERT INTO app_toolset_configs (toolset_id, enabled, updated_by, created_at, updated_at)
VALUES ('builtin-exa-web-search', 1, 'system', strftime('%s', 'now'), strftime('%s', 'now'));
