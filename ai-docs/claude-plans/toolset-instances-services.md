# Toolset Multi-Instance: Services Layer

## Context Summary

Services layer changes span database schema, database service methods, and tool service business logic. The core shift: from UPSERT by `(user_id, toolset_id)` to proper CRUD by `instance_id` (UUID) with `(user_id, name)` uniqueness.

## Database Schema Changes

### File: `crates/services/migrations/0007_toolsets_config.up.sql`

**Modify in place** - no backwards compatibility needed.

Current schema has:
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `toolset_id TEXT` (serves as both type and instance identifier)
- `UNIQUE(user_id, toolset_id)`

New schema:

```sql
-- User toolset instances (one per configured instance, not per type)
CREATE TABLE IF NOT EXISTS user_toolset_configs (
    id TEXT PRIMARY KEY,                   -- UUID (TEXT for SQLite)
    user_id TEXT NOT NULL,                 -- JWT 'sub' claim
    toolset_type TEXT NOT NULL,            -- e.g., "builtin-exa-web-search"
    name TEXT NOT NULL,                    -- user-defined, alphanumeric + hyphens
    description TEXT,                      -- optional, max 255 chars
    enabled INTEGER NOT NULL DEFAULT 0,    -- boolean as integer
    encrypted_api_key TEXT,
    salt TEXT,
    nonce TEXT,
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL,
    UNIQUE(user_id, name)                  -- instance name unique per user
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_user_id
    ON user_toolset_configs(user_id);
CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_toolset_type
    ON user_toolset_configs(toolset_type);
CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_user_type
    ON user_toolset_configs(user_id, toolset_type);
CREATE INDEX IF NOT EXISTS idx_user_toolset_configs_enabled_created
    ON user_toolset_configs(user_id, toolset_type, enabled, created_at);
```

### File: `crates/services/migrations/0007_toolsets_config.down.sql`

Update to match new indexes:

```sql
DROP INDEX IF EXISTS idx_user_toolset_configs_enabled_created;
DROP INDEX IF EXISTS idx_user_toolset_configs_user_type;
DROP INDEX IF EXISTS idx_user_toolset_configs_toolset_type;
DROP INDEX IF EXISTS idx_user_toolset_configs_user_id;
DROP TABLE IF EXISTS user_toolset_configs;
```

**Note:** Keep `app_toolset_configs` table unchanged - it controls type-level enable/disable.

---

## Database Objects

### File: `crates/services/src/db/objs.rs`

Current `UserToolsetConfigRow` structure:
- `id: i64`
- `user_id: String`
- `toolset_id: String`
- ... encryption fields

New structure:

```rust
#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct UserToolsetConfigRow {
    pub id: String,                        // UUID as TEXT
    pub user_id: String,
    pub toolset_type: String,              // renamed from toolset_id
    pub name: String,                      // NEW
    pub description: Option<String>,       // NEW
    pub enabled: bool,
    pub encrypted_api_key: Option<String>,
    pub salt: Option<String>,
    pub nonce: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

---

## Database Service

### File: `crates/services/src/db/service.rs`

**Reference:** Existing trait `DbService` with methods:
- `get_user_toolset_config` - by (user_id, toolset_id)
- `update_user_toolset_config` - UPSERT semantics
- `delete_user_toolset_config` - by (user_id, toolset_id)

**Replace with instance-based methods:**

```rust
// In DbService trait, replace toolset config methods:

/// Get instance by UUID
async fn get_user_toolset_instance_by_id(
    &self,
    id: &str,
) -> Result<Option<UserToolsetConfigRow>, DbError>;

/// Get instance by user_id and name (for uniqueness check)
async fn get_user_toolset_instance_by_name(
    &self,
    user_id: &str,
    name: &str,
) -> Result<Option<UserToolsetConfigRow>, DbError>;

/// Create new instance (INSERT only, not UPSERT)
async fn create_user_toolset_instance(
    &self,
    row: &UserToolsetConfigRow,
) -> Result<UserToolsetConfigRow, DbError>;

/// Update instance by UUID
async fn update_user_toolset_instance(
    &self,
    row: &UserToolsetConfigRow,
) -> Result<UserToolsetConfigRow, DbError>;

/// List all instances for user
async fn list_user_toolset_instances(
    &self,
    user_id: &str,
) -> Result<Vec<UserToolsetConfigRow>, DbError>;

/// List instances by user and type (for chat default selection)
async fn list_user_toolset_instances_by_type(
    &self,
    user_id: &str,
    toolset_type: &str,
) -> Result<Vec<UserToolsetConfigRow>, DbError>;

/// Delete instance by UUID
async fn delete_user_toolset_instance(
    &self,
    id: &str,
) -> Result<(), DbError>;
```

**Reference:** Implementation in same file, `SqliteDbService` impl. Pattern follows existing methods like `get_api_token`, `create_chat`, etc.

---

## Tool Service

### File: `crates/services/src/tool_service.rs`

**Reference:** Existing `ToolService` trait and `DefaultToolService` impl.

Key existing methods to replace/adapt:
- `list_tools_for_user` - returns tools, not instances
- `get_user_toolset_config` - by (user_id, toolset_id)
- `update_user_toolset_config` - UPSERT
- `delete_user_toolset_config` - by (user_id, toolset_id)
- `execute_toolset_tool` - by (user_id, toolset_id, method)
- `is_toolset_available_for_user` - by toolset_id

**New trait methods:**

```rust
#[async_trait::async_trait]
pub trait ToolService: Debug + Send + Sync {
    // === Instance Management ===

    /// List all instances for user with tools info
    async fn list_user_instances(
        &self,
        user_id: &str,
    ) -> Result<Vec<ToolsetInstanceWithTools>, ToolsetError>;

    /// Create new instance
    async fn create_instance(
        &self,
        user_id: &str,
        toolset_type: &str,
        name: &str,
        description: Option<String>,
        enabled: bool,
        api_key: String,  // required on create
    ) -> Result<UserToolsetInstance, ToolsetError>;

    /// Get instance by UUID (validates user ownership)
    async fn get_instance(
        &self,
        user_id: &str,
        instance_id: &str,
    ) -> Result<Option<ToolsetInstanceWithTools>, ToolsetError>;

    /// Update instance (partial update)
    async fn update_instance(
        &self,
        user_id: &str,
        instance_id: &str,
        updates: InstanceUpdates,
    ) -> Result<UserToolsetInstance, ToolsetError>;

    /// Delete instance
    async fn delete_instance(
        &self,
        user_id: &str,
        instance_id: &str,
    ) -> Result<(), ToolsetError>;

    /// Execute tool on instance
    async fn execute_instance_tool(
        &self,
        user_id: &str,
        instance_id: &str,
        method: &str,
        request: ToolsetExecutionRequest,
    ) -> Result<ToolsetExecutionResponse, ToolsetError>;

    /// Check if instance is available (owned, type enabled, instance enabled, has key)
    async fn is_instance_available(
        &self,
        user_id: &str,
        instance_id: &str,
    ) -> Result<bool, ToolsetError>;

    // === Type-Level (keep for admin) ===

    /// List available toolset types
    fn list_toolset_types(&self) -> Vec<ToolsetDefinition>;

    /// Check if type is enabled at app level
    async fn is_toolset_type_enabled(
        &self,
        toolset_type: &str,
    ) -> Result<bool, ToolsetError>;

    // Keep existing app-level enable/disable methods
    async fn set_app_toolset_enabled(...);
    async fn get_app_toolset_config(...);
}
```

**Supporting types:**

```rust
/// Partial update fields for instance
#[derive(Debug, Default)]
pub struct InstanceUpdates {
    pub name: Option<String>,
    pub description: Option<Option<String>>,  // Some(None) = clear
    pub enabled: Option<bool>,
    pub api_key: Option<String>,  // Some = update key
}
```

**Reference for update pattern:** See `routes_api_models.rs` alias update handling where API key can be modified but not retrieved.

---

## Authorization Logic

### File: `crates/services/src/tool_service.rs`

**Execute authorization flow** (in `execute_instance_tool`):

1. Get instance by UUID
2. Verify `instance.user_id == user_id` → else `InstanceNotOwned`
3. Get `toolset_type` from instance
4. Check `is_toolset_type_enabled(toolset_type)` → else `ToolsetAppDisabled`
5. Check `instance.enabled` → else `ToolsetDisabled`
6. Check `instance.encrypted_api_key.is_some()` → else `ToolsetNotConfigured`
7. Decrypt API key and execute

**Reference:** Current authorization in `is_toolset_available_for_user` method.

---

## API Key Handling

**Reference:** Existing `encrypt_api_key` / `decrypt_api_key` functions in `tool_service.rs`.

No changes to encryption logic - same AES-GCM approach:
- On create: encrypt API key, store with salt/nonce
- On update (if key provided): re-encrypt with new salt/nonce
- On execute: decrypt key for external API call
- Never return decrypted key in API responses

---

## Existing Code Paths to Adapt

| Current | New |
|---------|-----|
| `builtin_toolsets()` registry | Keep - provides type definitions |
| `get_toolset_for_id(toolset_id)` | Keep - lookup type definition by type ID |
| `is_toolset_available_for_user(user_id, toolset_id)` | Replace with `is_instance_available(user_id, instance_id)` |
| `execute_toolset_tool(user_id, toolset_id, ...)` | Replace with `execute_instance_tool(user_id, instance_id, ...)` |

---

## Files to Modify

| File | Changes |
|------|---------|
| `crates/services/migrations/0007_toolsets_config.up.sql` | Schema: UUID id, add name/description, change unique constraint |
| `crates/services/migrations/0007_toolsets_config.down.sql` | Update drop indexes |
| `crates/services/src/db/objs.rs` | Update `UserToolsetConfigRow` struct |
| `crates/services/src/db/service.rs` | Replace toolset config methods with instance methods |
| `crates/services/src/tool_service.rs` | Replace with instance-based methods, update `DefaultToolService` |

---

## Test Considerations

### Database Service Tests (`crates/services/src/db/service_test.rs`)

- CRUD operations for instances
- Uniqueness constraint: same name for same user fails
- Same name for different users succeeds
- List by user returns only that user's instances
- List by type filters correctly

### Tool Service Tests (`crates/services/src/tool_service_test.rs`)

- Create instance with valid/invalid names
- Update partial fields
- Authorization: user can only access own instances
- Execute on enabled vs disabled instance
- Execute when type disabled at app level
- API key encryption/decryption roundtrip

**Reference:** Existing test patterns in `tool_service_test.rs` with mock DbService.
