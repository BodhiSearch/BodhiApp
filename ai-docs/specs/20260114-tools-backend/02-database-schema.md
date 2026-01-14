# Database Schema - Tools Feature

> Layer: `services` crate migrations | Status: Planning

## Migration: 0007_tools_config

### Up Migration

```sql
-- crates/services/migrations/0007_tools_config.up.sql
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

CREATE INDEX IF NOT EXISTS idx_user_tool_configs_user_id ON user_tool_configs(user_id);
CREATE INDEX IF NOT EXISTS idx_user_tool_configs_tool_id ON user_tool_configs(tool_id);
CREATE INDEX IF NOT EXISTS idx_user_tool_configs_enabled ON user_tool_configs(enabled);
```

### Down Migration

```sql
-- crates/services/migrations/0007_tools_config.down.sql
DROP INDEX IF EXISTS idx_user_tool_configs_enabled;
DROP INDEX IF EXISTS idx_user_tool_configs_tool_id;
DROP INDEX IF EXISTS idx_user_tool_configs_user_id;
DROP TABLE IF EXISTS user_tool_configs;
```

## Storage Pattern

- Table name: `user_tool_configs` (per-user configuration)
- Composite unique: `(user_id, tool_id)` - one config per user per tool
- User ID: JWT 'sub' claim stored directly as TEXT (no FK to users table)
- API keys encrypted using same `encrypt_api_key`/`decrypt_api_key` functions from `db/encryption.rs`
- Master key from `SecretService` → `KeyringStore`
- SQLite INTEGER for boolean (0/1)
- Unix timestamps as INTEGER

## Row Example

```
id: 1
user_id: "abc-123-uuid"
tool_id: "builtin-exa-web-search"
enabled: 1
encrypted_api_key: "base64..."
salt: "base64..."
nonce: "base64..."
created_at: 1705257600
updated_at: 1705257600
```
