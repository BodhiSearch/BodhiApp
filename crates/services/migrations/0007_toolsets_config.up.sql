-- Migration 0007: Toolset Configuration Tables
-- Creates tables for user toolset instances and app-level toolset configuration

-- Create the toolsets table for user-owned toolset instances
CREATE TABLE IF NOT EXISTS toolsets (
    id TEXT PRIMARY KEY,                   -- UUID as TEXT
    user_id TEXT NOT NULL,                 -- JWT 'sub' claim (no FK)
    scope_uuid TEXT NOT NULL,              -- Keycloak client scope UUID (environment-specific)
    name TEXT NOT NULL,                    -- user-defined instance name
    description TEXT,                      -- optional instance description
    enabled INTEGER NOT NULL DEFAULT 0,    -- boolean as integer
    -- Encrypted API key storage (same pattern as api_model_aliases)
    encrypted_api_key TEXT,
    salt TEXT,
    nonce TEXT,
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL,
    UNIQUE(user_id, name COLLATE NOCASE)   -- case-insensitive uniqueness per user
);

-- Create index on user_id for faster lookups by user
CREATE INDEX IF NOT EXISTS idx_toolsets_user_id ON toolsets(user_id);

-- Create index on scope_uuid for faster lookups by type
CREATE INDEX IF NOT EXISTS idx_toolsets_scope_uuid ON toolsets(scope_uuid);

-- Create composite index on user_id and scope_uuid for efficient filtering
CREATE INDEX IF NOT EXISTS idx_toolsets_user_scope_uuid ON toolsets(user_id, scope_uuid);

-- Create the app_toolset_configs table for app-level toolset configuration (admin-controlled)
CREATE TABLE IF NOT EXISTS app_toolset_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scope TEXT NOT NULL UNIQUE,            -- OAuth scope string (e.g., "scope_toolset-builtin-exa-web-search")
    scope_uuid TEXT NOT NULL,              -- Keycloak client scope UUID (environment-specific)
    enabled INTEGER NOT NULL DEFAULT 0,    -- boolean as integer
    updated_by TEXT NOT NULL,              -- user_id of admin who last updated
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL
);

-- Create index on scope for faster lookups
CREATE INDEX IF NOT EXISTS idx_app_toolset_configs_scope ON app_toolset_configs(scope);

-- Create index on scope_uuid for Keycloak lookups
CREATE INDEX IF NOT EXISTS idx_app_toolset_configs_scope_uuid ON app_toolset_configs(scope_uuid);

-- Note: Seed data is now inserted programmatically in DbService::seed_toolset_configs()
-- to support environment-specific scope_uuid values (dev vs prod)
