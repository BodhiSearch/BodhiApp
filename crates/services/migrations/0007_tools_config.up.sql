-- Add up migration script here

-- Create the user_tool_configs table for per-user tool configuration
CREATE TABLE IF NOT EXISTS user_tool_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,               -- JWT 'sub' claim (no FK)
    tool_id TEXT NOT NULL,               -- e.g., "builtin-exa-web-search"
    enabled INTEGER NOT NULL DEFAULT 0,  -- boolean as integer
    -- Encrypted API key storage (same pattern as api_model_aliases)
    encrypted_api_key TEXT,
    salt TEXT,
    nonce TEXT,
    created_at INTEGER NOT NULL,         -- Unix timestamp
    updated_at INTEGER NOT NULL,
    UNIQUE(user_id, tool_id)             -- composite unique constraint
);

-- Create index on user_id for faster lookups by user
CREATE INDEX IF NOT EXISTS idx_user_tool_configs_user_id ON user_tool_configs(user_id);

-- Create index on tool_id for faster lookups by tool
CREATE INDEX IF NOT EXISTS idx_user_tool_configs_tool_id ON user_tool_configs(tool_id);

-- Create index on enabled for filtering enabled tools
CREATE INDEX IF NOT EXISTS idx_user_tool_configs_enabled ON user_tool_configs(enabled);

-- Create the app_tool_configs table for app-level tool configuration (admin-controlled)
CREATE TABLE IF NOT EXISTS app_tool_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tool_id TEXT NOT NULL UNIQUE,        -- e.g., "builtin-exa-web-search"
    enabled INTEGER NOT NULL DEFAULT 0,  -- boolean as integer
    updated_by TEXT NOT NULL,            -- user_id of admin who last updated
    created_at INTEGER NOT NULL,         -- Unix timestamp
    updated_at INTEGER NOT NULL
);

-- Create index on tool_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_app_tool_configs_tool_id ON app_tool_configs(tool_id);
