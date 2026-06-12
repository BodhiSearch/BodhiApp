# Toolset Multi-Instance: objs + services Implementation Plan

## Summary

Implement multi-instance toolset support in the foundational layers (objs + services). This replaces single-instance-per-type with UUID-identified instances supporting user-defined names.

## Design Decisions (from Q&A)

| Decision | Choice |
|----------|--------|
| Domain type name | `Toolset` (simple REST resource) |
| Error location | Keep `ToolsetError` in services crate |
| toolset_type validation | Strict - validate against `builtin_toolsets()` |
| name validation | objs layer, max **24 chars**, alphanumeric + hyphens |
| API key update | `ApiKeyUpdate` enum (Keep/Set pattern) |
| Update semantics | PUT - all fields required except API key |
| Data handling | Fresh DB - no migration logic |
| Test strategy | Replace tests entirely |
| Domain purity | `Toolset` pure (no API key fields), `api_key_masked` in routes layer |

---

## Phase objs-types: Domain Types (objs crate)

### Files: `crates/objs/src/toolsets.rs`

**Keep unchanged:**
- `ToolsetScope` - OAuth scope parsing (type-level)
- `ToolsetDefinition` - Toolset type definition (id, name, tools)
- `ToolDefinition` - Single tool definition
- `FunctionDefinition` - Function within tool
- `AppToolsetConfig` - Admin type-level config

**Modify:**
- `ToolsetWithTools` - Keep for admin type listing, remove `user_config` field (multi-instance model means user can have 0-N instances per type; routes layer will build appropriate responses)

**Remove:**
- `UserToolsetConfig` - Replaced by `Toolset`
- `UserToolsetConfigSummary` - Replaced by routes layer response DTO (multi-instance model needs different representation)

**Add:**

```rust
/// User's configured toolset (pure domain type, no API key fields)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct Toolset {
    /// Unique toolset identifier (UUID)
    pub id: String,
    /// User-defined toolset name (alphanumeric + hyphens, max 24 chars, unique per user)
    pub name: String,
    /// The toolset type this is an instance of (e.g., "builtin-exa-web-search")
    pub toolset_type: String,
    /// Optional user description (max 255 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether user has enabled this instance
    pub enabled: bool,
    /// Creation timestamp
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}
```

### Files: `crates/objs/src/toolsets.rs` - Validation

**Add validation function:**

```rust
use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for valid toolset toolset names: alphanumeric and hyphens only
static TOOLSET_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9-]+$").unwrap()
});

/// Maximum toolset toolset name length
pub const MAX_TOOLSET_NAME_LEN: usize = 24;

/// Maximum toolset description length
pub const MAX_TOOLSET_DESCRIPTION_LEN: usize = 255;

/// Validate toolset toolset name format
pub fn validate_toolset_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("name cannot be empty".to_string());
    }
    if name.len() > MAX_TOOLSET_NAME_LEN {
        return Err(format!("name exceeds {} characters", MAX_TOOLSET_NAME_LEN));
    }
    if !TOOLSET_NAME_REGEX.is_match(name) {
        return Err("name must contain only alphanumeric characters and hyphens".to_string());
    }
    Ok(())
}

/// Validate toolset description
pub fn validate_toolset_description(description: &str) -> Result<(), String> {
    if description.len() > MAX_TOOLSET_DESCRIPTION_LEN {
        return Err(format!("description exceeds {} characters", MAX_TOOLSET_DESCRIPTION_LEN));
    }
    Ok(())
}
```

### Files: `crates/objs/src/toolsets.rs` - ToolsetWithTools Modification

**Modify ToolsetWithTools (remove user_config field):**

```rust
/// Toolset type with app-level configuration status (API response model for admin type listing)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ToolsetWithTools {
    /// Unique toolset type identifier (e.g., "builtin-exa-web-search")
    pub toolset_id: String,
    /// Human-readable name (e.g., "Exa Web Search")
    pub name: String,
    /// Description of the toolset
    pub description: String,
    /// Whether the toolset type is enabled at app level (admin-controlled)
    pub app_enabled: bool,
    /// Tools provided by this toolset type
    pub tools: Vec<ToolDefinition>,
    // REMOVED: user_config field - multi-instance model means routes layer builds instance lists
}
```

### Files: `crates/objs/src/lib.rs`

**Update exports:**
- Export `Toolset` (new)
- Export validation functions and constants
- Remove `UserToolsetConfig`, `UserToolsetConfigSummary` exports

**Note:** Tool name encoding/parsing is frontend/client responsibility. Backend only ensures toolset names use safe characters (alphanumeric + hyphens) to avoid conflicts with client-side patterns.

### Tests for objs (inline in toolsets.rs)

```rust
#[cfg(test)]
mod toolset_tests {
    use crate::{
        validate_toolset_name, validate_toolset_description,
        MAX_TOOLSET_NAME_LEN, MAX_TOOLSET_DESCRIPTION_LEN,
    };
    use rstest::rstest;

    // Name validation tests
    #[rstest]
    #[case("my-toolset", true)]
    #[case("MyToolset123", true)]
    #[case("a", true)]
    #[case("a-b-c", true)]
    #[case("", false)]                    // empty
    #[case("my_toolset", false)]          // underscore
    #[case("my toolset", false)]          // space
    #[case("my.toolset", false)]          // dot
    fn test_validate_toolset_name_format(#[case] name: &str, #[case] valid: bool) {
        let result = validate_toolset_name(name);
        assert_eq!(valid, result.is_ok(), "name='{}', result={:?}", name, result);
    }

    #[test]
    fn test_validate_toolset_name_max_length() {
        let max_name = "a".repeat(MAX_TOOLSET_NAME_LEN);
        assert!(validate_toolset_name(&max_name).is_ok());

        let too_long = "a".repeat(MAX_TOOLSET_NAME_LEN + 1);
        assert!(validate_toolset_name(&too_long).is_err());
    }

    // Description validation tests
    #[test]
    fn test_validate_toolset_description() {
        assert!(validate_toolset_description("A valid description").is_ok());
        assert!(validate_toolset_description("").is_ok()); // empty is valid

        let max_desc = "a".repeat(MAX_TOOLSET_DESCRIPTION_LEN);
        assert!(validate_toolset_description(&max_desc).is_ok());

        let too_long = "a".repeat(MAX_TOOLSET_DESCRIPTION_LEN + 1);
        assert!(validate_toolset_description(&too_long).is_err());
    }
}
```

---

## Phase services-schema: Database Schema (services crate)

### Files: `crates/services/migrations/0007_toolsets_config.up.sql`

**Replace entire contents:**

```sql
-- Toolset instances (one per configured instance, not per type)
CREATE TABLE IF NOT EXISTS toolsets (
    id TEXT PRIMARY KEY,                   -- UUID (TEXT for SQLite)
    user_id TEXT NOT NULL,                 -- JWT 'sub' claim
    toolset_type TEXT NOT NULL,            -- e.g., "builtin-exa-web-search"
    name TEXT NOT NULL,                    -- user-defined, alphanumeric + hyphens, max 24
    description TEXT,                      -- optional, max 255 chars
    enabled INTEGER NOT NULL DEFAULT 0,    -- boolean as integer
    encrypted_api_key TEXT,
    salt TEXT,
    nonce TEXT,
    created_at INTEGER NOT NULL,           -- Unix timestamp
    updated_at INTEGER NOT NULL,
    UNIQUE(user_id, name)                  -- toolset name unique per user
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_toolsets_user_id
    ON toolsets(user_id);
CREATE INDEX IF NOT EXISTS idx_toolsets_toolset_type
    ON toolsets(toolset_type);
CREATE INDEX IF NOT EXISTS idx_toolsets_user_type
    ON toolsets(user_id, toolset_type);

-- App-level toolset configuration (unchanged from original)
CREATE TABLE IF NOT EXISTS app_toolset_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    toolset_id TEXT NOT NULL UNIQUE,
    enabled INTEGER NOT NULL DEFAULT 0,
    updated_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_app_toolset_configs_toolset_id
    ON app_toolset_configs(toolset_id);

-- Seed Exa web search as disabled by default
INSERT OR IGNORE INTO app_toolset_configs (toolset_id, enabled, updated_by, created_at, updated_at)
VALUES ('builtin-exa-web-search', 0, 'system', strftime('%s', 'now'), strftime('%s', 'now'));
```

### Files: `crates/services/migrations/0007_toolsets_config.down.sql`

**Replace entire contents:**

```sql
DROP INDEX IF EXISTS idx_app_toolset_configs_toolset_id;
DROP TABLE IF EXISTS app_toolset_configs;
DROP INDEX IF EXISTS idx_toolsets_user_type;
DROP INDEX IF EXISTS idx_toolsets_toolset_type;
DROP INDEX IF EXISTS idx_toolsets_user_id;
DROP TABLE IF EXISTS toolsets;
```

---

## Phase services-row: Database Row Struct (services crate)

### Files: `crates/services/src/db/objs.rs`

**Rename `UserToolsetConfigRow` → `ToolsetRow`:**

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ToolsetRow {
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

**Keep unchanged:**
- `AppToolsetConfigRow` (admin type-level config)
- `AppClientToolsetConfigRow` (OAuth app registration)
- `ApiKeyUpdate` enum (reuse for toolsets)

---

## Phase services-db: DbService Methods (services crate)

### Files: `crates/services/src/db/service.rs`

**Remove old methods from DbService trait:**
- `get_user_toolset_config(user_id, toolset_id)`
- `upsert_user_toolset_config(config)`
- `list_user_toolset_configs(user_id)`
- `delete_user_toolset_config(user_id, toolset_id)`

**Add new methods to DbService trait:**

```rust
// === Toolset Instance Methods ===

/// Get toolset by UUID
async fn get_toolset_by_id(&self, id: &str) -> Result<Option<ToolsetRow>, DbError>;

/// Get toolset by user_id and name (for uniqueness check)
async fn get_toolset_by_name(&self, user_id: &str, name: &str) -> Result<Option<ToolsetRow>, DbError>;

/// Create new toolset (INSERT only)
async fn create_toolset(&self, row: &ToolsetRow) -> Result<ToolsetRow, DbError>;

/// Update toolset by UUID
async fn update_toolset(&self, row: &ToolsetRow, api_key_update: ApiKeyUpdate) -> Result<ToolsetRow, DbError>;

/// List all toolsets for user
async fn list_toolsets(&self, user_id: &str) -> Result<Vec<ToolsetRow>, DbError>;

/// List toolsets by user and type
async fn list_toolsets_by_type(&self, user_id: &str, toolset_type: &str) -> Result<Vec<ToolsetRow>, DbError>;

/// Delete toolset by UUID
async fn delete_toolset(&self, id: &str) -> Result<(), DbError>;

/// Get decrypted API key for toolset
async fn get_toolset_api_key(&self, id: &str) -> Result<Option<String>, DbError>;
```

**Implement in SqliteDbService** with proper SQL queries against `toolsets` table.

---

## Phase services-tool: ToolService Methods (services crate)

### Files: `crates/services/src/tool_service.rs`

**Add error variants to ToolsetError:**

```rust
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolsetError {
    // ... existing variants ...

    #[error("instance_not_found")]
    #[error_meta(error_type = ErrorType::NotFound, status = 404)]
    ToolsetNotFound(String),

    #[error("instance_name_exists")]
    #[error_meta(error_type = ErrorType::Conflict, status = 409)]
    NameExists(String),

    #[error("invalid_instance_name")]
    #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
    InvalidName(String),

    #[error("instance_not_owned")]
    #[error_meta(error_type = ErrorType::Forbidden, status = 403)]
    NotOwned,

    #[error("invalid_toolset_type")]
    #[error_meta(error_type = ErrorType::BadRequest, status = 400)]
    InvalidToolsetType(String),
}
```

**Update localization messages:** `crates/services/src/resources/en-US/messages.ftl`

```ftl
instance_not_found = Toolset instance '{$id}' not found
instance_name_exists = Toolset toolset name '{$name}' already exists
invalid_instance_name = Invalid toolset name: {$reason}
instance_not_owned = You don't have permission to access this toolset instance
invalid_toolset_type = Invalid toolset type: {$toolset_type}
```

**Remove old methods from ToolService trait:**
- `get_user_toolset_config()`
- `update_user_toolset_config()`
- `delete_user_toolset_config()`
- `execute_toolset_tool()`
- `is_toolset_available_for_user()`

**Add new methods to ToolService trait:**

```rust
#[async_trait]
pub trait ToolService: Debug + Send + Sync {
    // === Instance Management ===

    /// List all toolset instances for user
    async fn list_instances(&self, user_id: &str) -> Result<Vec<Toolset>, ToolsetError>;

    /// Create new toolset instance
    async fn create_instance(
        &self,
        user_id: &str,
        toolset_type: &str,
        name: &str,
        description: Option<String>,
        enabled: bool,
        api_key: String,  // required on create
    ) -> Result<Toolset, ToolsetError>;

    /// Get toolset instance by UUID (validates user ownership)
    async fn get_instance(&self, user_id: &str, instance_id: &str) -> Result<Option<Toolset>, ToolsetError>;

    /// Update toolset instance (PUT semantics - all fields except API key)
    async fn update_instance(
        &self,
        user_id: &str,
        instance_id: &str,
        name: &str,
        description: Option<String>,
        enabled: bool,
        api_key_update: ApiKeyUpdate,
    ) -> Result<Toolset, ToolsetError>;

    /// Delete toolset instance
    async fn delete_instance(&self, user_id: &str, instance_id: &str) -> Result<(), ToolsetError>;

    /// Execute tool on toolset instance
    async fn execute_instance_tool(
        &self,
        user_id: &str,
        instance_id: &str,
        method: &str,
        request: ToolsetExecutionRequest,
    ) -> Result<ToolsetExecutionResponse, ToolsetError>;

    /// Check if toolset instance is available for execution
    async fn is_instance_available(&self, user_id: &str, instance_id: &str) -> Result<bool, ToolsetError>;

    // === Type-Level (keep for admin + validation) ===

    /// List available toolset type definitions
    fn list_toolset_types(&self) -> Vec<ToolsetDefinition>;

    /// Get toolset type definition by ID
    fn get_toolset_type(&self, toolset_type: &str) -> Option<ToolsetDefinition>;

    /// Validate toolset type exists
    fn validate_toolset_type(&self, toolset_type: &str) -> Result<(), ToolsetError>;

    /// Check if toolset type is enabled at app level
    async fn is_type_enabled(&self, toolset_type: &str) -> Result<bool, ToolsetError>;

    // Keep existing app-level methods unchanged
    async fn get_app_toolset_config(&self, toolset_id: &str) -> Result<Option<AppToolsetConfig>, ToolsetError>;
    async fn set_app_toolset_enabled(...) -> Result<AppToolsetConfig, ToolsetError>;
    async fn list_app_toolset_configs(&self) -> Result<Vec<AppToolsetConfig>, ToolsetError>;

    // Keep existing methods for tool definitions
    fn list_all_tool_definitions(&self) -> Vec<ToolDefinition>;
}
```

**Implement in DefaultToolService:**

Key implementation patterns:
1. `create_instance()`:
   - Validate name format (via `objs::validate_toolset_name`)
   - Validate toolset_type against `builtin_toolsets()`
   - Check name uniqueness via `db.get_toolset_by_name()`
   - Generate UUID
   - Encrypt API key
   - Insert via `db.create_toolset()`

2. `update_instance()`:
   - Get existing by ID
   - Verify ownership
   - Validate new name format
   - If name changed, check uniqueness
   - Handle ApiKeyUpdate (Keep/Set)
   - Update via `db.update_toolset()`

3. `execute_instance_tool()`:
   - Get instance by ID
   - Verify ownership
   - Check app-level type enabled
   - Check instance enabled
   - Get decrypted API key
   - Execute tool method

---

## Phase services-tests: Replace Tests (services crate)

### Files: `crates/services/src/db/service.rs` (test module)

**Remove old toolset config tests, add new tests:**

```rust
#[cfg(test)]
mod toolset_tests {
    // CRUD tests
    #[tokio::test]
    async fn test_create_toolset_success() { ... }

    #[tokio::test]
    async fn test_create_toolset_duplicate_name_fails() { ... }

    #[tokio::test]
    async fn test_create_toolset_same_name_different_user_succeeds() { ... }

    #[tokio::test]
    async fn test_get_toolset_by_id() { ... }

    #[tokio::test]
    async fn test_get_toolset_by_name() { ... }

    #[tokio::test]
    async fn test_update_toolset_keeps_api_key() { ... }

    #[tokio::test]
    async fn test_update_toolset_updates_api_key() { ... }

    #[tokio::test]
    async fn test_update_toolset_clears_api_key() { ... }

    #[tokio::test]
    async fn test_list_toolsets_returns_user_instances_only() { ... }

    #[tokio::test]
    async fn test_list_toolsets_by_type() { ... }

    #[tokio::test]
    async fn test_delete_toolset() { ... }

    #[tokio::test]
    async fn test_get_toolset_api_key_decrypts() { ... }
}
```

### Files: `crates/services/src/tool_service.rs` (test module)

**Remove old tests, add new tests:**

```rust
#[cfg(test)]
mod tests {
    // Instance management tests
    #[tokio::test]
    async fn test_create_instance_validates_name() { ... }

    #[tokio::test]
    async fn test_create_instance_validates_toolset_type() { ... }

    #[tokio::test]
    async fn test_create_instance_requires_api_key() { ... }

    #[tokio::test]
    async fn test_create_instance_name_uniqueness_per_user() { ... }

    #[tokio::test]
    async fn test_get_instance_validates_ownership() { ... }

    #[tokio::test]
    async fn test_update_instance_validates_ownership() { ... }

    #[tokio::test]
    async fn test_update_instance_validates_new_name() { ... }

    #[tokio::test]
    async fn test_delete_instance_validates_ownership() { ... }

    // Execution tests
    #[tokio::test]
    async fn test_execute_instance_tool_checks_app_enabled() { ... }

    #[tokio::test]
    async fn test_execute_instance_tool_checks_instance_enabled() { ... }

    #[tokio::test]
    async fn test_execute_instance_tool_checks_api_key() { ... }

    // Type validation tests
    #[tokio::test]
    async fn test_validate_toolset_type_accepts_builtin() { ... }

    #[tokio::test]
    async fn test_validate_toolset_type_rejects_unknown() { ... }
}
```

---

## Implementation Order

1. **Phase objs-types** - Domain types, validation, encoding
2. **Phase services-schema** - Migration files
3. **Phase services-row** - Row struct
4. **Phase services-db** - DbService trait + implementation
5. **Phase services-tool** - ToolService trait + implementation + errors
6. **Phase services-tests** - Replace all tests

## Verification

After each phase:
```bash
cargo fmt --all
cargo test -p objs
cargo test -p services
```

Final verification:
```bash
make test.backend
```

## Files Modified Summary

| Crate | File | Action |
|-------|------|--------|
| objs | `src/toolsets.rs` | Modify (add Toolset, validation; remove UserToolsetConfig, UserToolsetConfigSummary) |
| objs | `src/lib.rs` | Update exports |
| services | `migrations/0007_toolsets_config.up.sql` | Replace (table: `toolsets`) |
| services | `migrations/0007_toolsets_config.down.sql` | Replace |
| services | `src/db/objs.rs` | Rename `UserToolsetConfigRow` → `ToolsetRow` |
| services | `src/db/service.rs` | Replace toolset methods + tests |
| services | `src/tool_service.rs` | Replace methods, add errors + tests |
| services | `src/resources/en-US/messages.ftl` | Add error messages |

## Out of Scope

- Routes/API endpoints (routes_app crate)
- Auth middleware (auth_middleware crate)
- Frontend (bodhi crate)
- `app_toolset_configs` table (unchanged)
- `app_client_toolset_configs` table (unchanged)
