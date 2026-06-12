# Database Schema - Toolsets Feature

> Layer: `services` crate migrations | Status: ✅ Complete

## Migration: 0009_toolsets_schema

**Files**: `crates/services/migrations/0009_toolsets_schema.{up,down}.sql`

### Up Migration

```sql
-- crates/services/migrations/0009_toolsets_schema.up.sql

-- User toolset configurations (per-user API keys at toolset level)
CREATE TABLE IF NOT EXISTS user_toolset_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,                 -- JWT 'sub' claim (no FK)
    toolset_id TEXT NOT NULL,              -- e.g., "builtin-exa-web-search"
    enabled INTEGER NOT NULL DEFAULT 0,    -- boolean as integer
    -- Encrypted API key storage (one key per toolset)
    encrypted_api_key TEXT,
    salt TEXT,
    nonce TEXT,
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL,
    UNIQUE(user_id, toolset_id)            -- composite unique constraint
);

CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_user_id ON user_toolset_configs(user_id);
CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_toolset_id ON user_toolset_configs(toolset_id);
CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_enabled ON user_toolset_configs(enabled);

-- App-level toolset configurations (admin-controlled)
CREATE TABLE IF NOT EXISTS app_toolset_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    toolset_id TEXT NOT NULL UNIQUE,       -- e.g., "builtin-exa-web-search"
    enabled INTEGER NOT NULL DEFAULT 0,    -- boolean as integer
    updated_by TEXT NOT NULL,              -- user_id of admin who last updated
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_app_toolset_configs_toolset_id ON app_toolset_configs(toolset_id);

-- Seed default toolset config (enabled by default for setup)
INSERT INTO app_toolset_configs (toolset_id, enabled, updated_by, created_at, updated_at)
VALUES ('builtin-exa-web-search', 1, 'system', strftime('%s', 'now'), strftime('%s', 'now'));

-- App-client toolset configurations (cached from Keycloak)
CREATE TABLE IF NOT EXISTS app_client_toolset_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_client_id TEXT NOT NULL UNIQUE,
    config_version TEXT NOT NULL,
    toolsets_json TEXT NOT NULL,  -- JSON array: [{"toolset_id":"...","toolset_scope":"..."}]
    resource_scope TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_app_client_toolset_configs_client_id ON app_client_toolset_configs(app_client_id);
```

### Down Migration

```sql
-- crates/services/migrations/0009_toolsets_schema.down.sql
DROP INDEX IF EXISTS idx_app_client_toolset_configs_client_id;
DROP TABLE IF EXISTS app_client_toolset_configs;

DROP INDEX IF EXISTS idx_app_toolset_configs_toolset_id;
DROP TABLE IF EXISTS app_toolset_configs;

DROP INDEX IF EXISTS idx_user_toolset_configs_enabled;
DROP INDEX IF EXISTS idx_user_toolset_configs_toolset_id;
DROP INDEX IF EXISTS idx_user_toolset_configs_user_id;
DROP TABLE IF EXISTS user_toolset_configs;
```

## Storage Pattern

- **user_toolset_configs**: Per-user toolset configuration with API keys
- **app_toolset_configs**: Admin-controlled app-level enable/disable
- **app_client_toolset_configs**: Cached external app configurations from Keycloak

### Key Design Points

- Composite unique: `(user_id, toolset_id)` - one config per user per toolset
- User ID: JWT 'sub' claim stored directly as TEXT (no FK to users table)
- API keys encrypted using `encrypt_api_key`/`decrypt_api_key` from `db/encryption.rs`
- Master key from `SecretService` → `KeyringStore`
- SQLite INTEGER for boolean (0/1)
- Unix timestamps as INTEGER
- API key stored at **toolset level** (one key covers all tools in that toolset)

## Row Structures

### UserToolsetConfigRow

```rust
pub struct UserToolsetConfigRow {
    pub id: i64,
    pub user_id: String,
    pub toolset_id: String,
    pub enabled: bool,
    pub encrypted_api_key: Option<String>,
    pub salt: Option<String>,
    pub nonce: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### AppToolsetConfigRow

```rust
pub struct AppToolsetConfigRow {
    pub id: i64,
    pub toolset_id: String,
    pub enabled: bool,
    pub updated_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}
```

### AppClientToolsetConfigRow

```rust
pub struct AppClientToolsetConfigRow {
    pub id: i64,
    pub app_client_id: String,
    pub config_version: String,
    pub toolsets_json: String,  // JSON array
    pub resource_scope: String,
    pub created_at: i64,
    pub updated_at: i64,
}
```

## Row Example

```
id: 1
user_id: "abc-123-uuid"
toolset_id: "builtin-exa-web-search"
enabled: 1
encrypted_api_key: "base64..."
salt: "base64..."
nonce: "base64..."
created_at: 1705257600
updated_at: 1705257600
```

## Test Coverage

Tests cover:
- Migration up/down roundtrip
- CRUD operations with encryption/decryption
- Unique constraint validation
- Row conversion to/from domain model
