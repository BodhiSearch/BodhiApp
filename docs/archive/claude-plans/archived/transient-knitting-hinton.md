# Toolset Multi-Instance: objs + services Implementation Plan

## Summary

Implement multi-instance toolset support in foundational layers (objs + services). Replaces single-instance-per-type with UUID-identified instances supporting user-defined names.

## Key Decisions

| Decision | Choice |
|----------|--------|
| Method naming | `create`, `get`, `update`, `delete`, `list` (not `create_instance`) |
| ID field name | `id` (UUID), not `instance_id` |
| Name uniqueness | Case-insensitive per user |
| App-level check | At create/update time (not just execution) |
| Ownership error | Return `NotFound` (hide existence of other users' instances) |
| Test strategy | Real SQLite (file in tempdir), replace tests entirely |

---

## Phase objs-types: Domain Types (objs crate)

### File: `crates/objs/src/toolsets.rs`

**Keep unchanged:**
- `ToolsetScope`, `ToolsetDefinition`, `ToolDefinition`, `FunctionDefinition`
- `AppToolsetConfig`, `ToolsetExecutionRequest`, `ToolsetExecutionResponse`

**Modify `ToolsetWithTools`** - remove `user_config` field:
```rust
pub struct ToolsetWithTools {
    pub toolset_id: String,
    pub name: String,
    pub description: String,
    pub app_enabled: bool,
    pub tools: Vec<ToolDefinition>,
    // REMOVED: user_config: Option<UserToolsetConfigSummary>
}
```

**Remove:**
- `UserToolsetConfig`
- `UserToolsetConfigSummary`

**Add `Toolset` struct:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct Toolset {
    pub id: String,
    pub name: String,
    pub toolset_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub enabled: bool,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}
```

**Add validation:**
```rust
use once_cell::sync::Lazy;
use regex::Regex;

static TOOLSET_NAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9-]+$").unwrap()
});

pub const MAX_TOOLSET_NAME_LEN: usize = 24;
pub const MAX_TOOLSET_DESCRIPTION_LEN: usize = 255;

pub fn validate_toolset_name(name: &str) -> Result<(), String> { ... }
pub fn validate_toolset_description(description: &str) -> Result<(), String> { ... }
```

**Update `crates/objs/src/lib.rs` exports.**

**Tests:** Add validation tests using rstest with explicit imports (no `use super::*`).

---

## Phase services-schema: Database Schema

### File: `crates/services/migrations/0007_toolsets_config.up.sql`

**Replace with:**
```sql
CREATE TABLE IF NOT EXISTS toolsets (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    toolset_type TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 0,
    encrypted_api_key TEXT,
    salt TEXT,
    nonce TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE(user_id, name COLLATE NOCASE)  -- case-insensitive uniqueness
);

CREATE INDEX IF NOT EXISTS idx_toolsets_user_id ON toolsets(user_id);
CREATE INDEX IF NOT EXISTS idx_toolsets_toolset_type ON toolsets(toolset_type);
CREATE INDEX IF NOT EXISTS idx_toolsets_user_type ON toolsets(user_id, toolset_type);

-- app_toolset_configs unchanged
CREATE TABLE IF NOT EXISTS app_toolset_configs (...);
-- seed data unchanged
```

### File: `crates/services/migrations/0007_toolsets_config.down.sql`

**Replace with drop statements.**

---

## Phase services-row: Database Row Struct

### File: `crates/services/src/db/objs.rs`

**Rename `UserToolsetConfigRow` → `ToolsetRow`:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ToolsetRow {
    pub id: String,           // UUID as TEXT (changed from i64)
    pub user_id: String,
    pub toolset_type: String, // renamed from toolset_id
    pub name: String,         // NEW
    pub description: Option<String>, // NEW
    pub enabled: bool,
    pub encrypted_api_key: Option<String>,
    pub salt: Option<String>,
    pub nonce: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

**Keep unchanged:** `AppToolsetConfigRow`, `AppClientToolsetConfigRow`, `ApiKeyUpdate`

---

## Phase services-db: DbService Methods

### File: `crates/services/src/db/service.rs`

**Remove from DbService trait:**
- `get_user_toolset_config(user_id, toolset_id)`
- `upsert_user_toolset_config(config)`
- `list_user_toolset_configs(user_id)`
- `delete_user_toolset_config(user_id, toolset_id)`

**Add to DbService trait:**
```rust
async fn get_toolset(&self, id: &str) -> Result<Option<ToolsetRow>, DbError>;
async fn get_toolset_by_name(&self, user_id: &str, name: &str) -> Result<Option<ToolsetRow>, DbError>;
async fn create_toolset(&self, row: &ToolsetRow) -> Result<ToolsetRow, DbError>;
async fn update_toolset(&self, row: &ToolsetRow, api_key_update: ApiKeyUpdate) -> Result<ToolsetRow, DbError>;
async fn list_toolsets(&self, user_id: &str) -> Result<Vec<ToolsetRow>, DbError>;
async fn list_toolsets_by_type(&self, user_id: &str, toolset_type: &str) -> Result<Vec<ToolsetRow>, DbError>;
async fn delete_toolset(&self, id: &str) -> Result<(), DbError>;
async fn get_toolset_api_key(&self, id: &str) -> Result<Option<String>, DbError>;
```

**Implement in SqliteDbService:**
- `get_toolset_by_name`: Use `COLLATE NOCASE` for case-insensitive lookup
- `create_toolset`: Generate UUID, encrypt API key, INSERT
- `update_toolset`: Handle `ApiKeyUpdate::Keep` / `ApiKeyUpdate::Set(...)`
- `get_toolset_api_key`: Decrypt using `encryption::decrypt_api_key`

**Update `test_utils/db.rs`:** Add new methods to `TestDbService`.

---

## Phase services-tool: ToolService Methods

### File: `crates/services/src/tool_service.rs`

**Add error variants to `ToolsetError`:**
```rust
#[error("toolset_not_found")]
#[error_meta(error_type = ErrorType::NotFound, status = 404)]
ToolsetNotFound(String),  // reuse existing, used for instance not found too

#[error("name_exists")]
#[error_meta(error_type = ErrorType::Conflict, status = 409)]
NameExists(String),

#[error("invalid_name")]
#[error_meta(error_type = ErrorType::BadRequest, status = 400)]
InvalidName(String),

#[error("invalid_toolset_type")]
#[error_meta(error_type = ErrorType::BadRequest, status = 400)]
InvalidToolsetType(String),
```

**Update `crates/services/src/resources/en-US/messages.ftl`:**
```ftl
name_exists = Toolset name '{$name}' already exists
invalid_name = Invalid toolset name: {$reason}
invalid_toolset_type = Invalid toolset type: {$toolset_type}
```

**Remove from ToolService trait:**
- `get_user_toolset_config`, `update_user_toolset_config`, `delete_user_toolset_config`
- `execute_toolset_tool`, `is_toolset_available_for_user`

**Add to ToolService trait:**
```rust
async fn list(&self, user_id: &str) -> Result<Vec<Toolset>, ToolsetError>;
async fn get(&self, user_id: &str, id: &str) -> Result<Option<Toolset>, ToolsetError>;
async fn create(
    &self,
    user_id: &str,
    toolset_type: &str,
    name: &str,
    description: Option<String>,
    enabled: bool,
    api_key: String,
) -> Result<Toolset, ToolsetError>;
async fn update(
    &self,
    user_id: &str,
    id: &str,
    name: &str,
    description: Option<String>,
    enabled: bool,
    api_key_update: ApiKeyUpdate,
) -> Result<Toolset, ToolsetError>;
async fn delete(&self, user_id: &str, id: &str) -> Result<(), ToolsetError>;
async fn execute(
    &self,
    user_id: &str,
    id: &str,
    method: &str,
    request: ToolsetExecutionRequest,
) -> Result<ToolsetExecutionResponse, ToolsetError>;
async fn is_available(&self, user_id: &str, id: &str) -> Result<bool, ToolsetError>;

// Type-level methods (keep for admin + validation)
fn list_types(&self) -> Vec<ToolsetDefinition>;
fn get_type(&self, toolset_type: &str) -> Option<ToolsetDefinition>;
fn validate_type(&self, toolset_type: &str) -> Result<(), ToolsetError>;
async fn is_type_enabled(&self, toolset_type: &str) -> Result<bool, ToolsetError>;
```

**Implementation patterns:**

1. `create()`:
   - Validate name format via `objs::validate_toolset_name`
   - Validate toolset_type against `builtin_toolsets()`
   - Check app-level enabled via `is_type_enabled()`
   - Check name uniqueness (case-insensitive) via `db.get_toolset_by_name()`
   - Generate UUID via `uuid::Uuid::new_v4().to_string()`
   - Encrypt API key
   - Insert via `db.create_toolset()`

2. `get()`:
   - Fetch by ID via `db.get_toolset(id)`
   - Return `None` if not found OR if `user_id` doesn't match (hide existence)

3. `update()`:
   - Fetch existing by ID
   - If not found or user_id mismatch → return `ToolsetNotFound`
   - Validate new name format
   - If name changed, check uniqueness (case-insensitive)
   - Check app-level enabled (prevent enabling if type disabled)
   - Handle `ApiKeyUpdate`
   - Update via `db.update_toolset()`

4. `delete()`:
   - Fetch by ID, verify ownership (return `ToolsetNotFound` if mismatch)
   - Allow delete even if app-level type disabled
   - Delete via `db.delete_toolset()`

5. `execute()`:
   - Fetch instance, verify ownership
   - Check app-level type enabled
   - Check instance enabled
   - Get decrypted API key
   - Execute via Exa service

---

## Phase services-tests: Replace Tests

### File: `crates/services/src/db/service.rs` (test module)

Replace old `user_toolset_config` tests with:
```rust
#[cfg(test)]
mod toolset_tests {
    use crate::db::{...};
    use crate::test_utils::test_db_service;
    use rstest::rstest;

    #[rstest]
    #[tokio::test]
    async fn test_create_toolset_success(#[future] test_db_service: TestDbService) { ... }

    #[rstest]
    #[tokio::test]
    async fn test_create_toolset_duplicate_name_fails(#[future] test_db_service: TestDbService) { ... }

    #[rstest]
    #[tokio::test]
    async fn test_create_toolset_same_name_different_user_succeeds(#[future] test_db_service: TestDbService) { ... }

    #[rstest]
    #[tokio::test]
    async fn test_name_uniqueness_case_insensitive(#[future] test_db_service: TestDbService) { ... }

    // ... more tests per plan
}
```

### File: `crates/services/src/tool_service.rs` (test module)

Replace old tests with service-level tests covering:
- Name validation
- Type validation
- App-level enabled check at create/update
- Ownership validation (returns NotFound)
- Execute flow checks

---

## Files Modified Summary

| Crate | File | Action |
|-------|------|--------|
| objs | `src/toolsets.rs` | Add `Toolset`, validation; remove `UserToolsetConfig*`; modify `ToolsetWithTools` |
| objs | `src/lib.rs` | Update exports |
| services | `migrations/0007_toolsets_config.up.sql` | Replace |
| services | `migrations/0007_toolsets_config.down.sql` | Replace |
| services | `src/db/objs.rs` | Rename row struct |
| services | `src/db/service.rs` | Replace toolset methods + impl |
| services | `src/test_utils/db.rs` | Add new method implementations |
| services | `src/tool_service.rs` | Replace methods, add errors |
| services | `src/resources/en-US/messages.ftl` | Add error messages |

---

## Verification

After each phase:
```bash
cargo fmt --all
cargo test -p objs        # for objs changes
cargo test -p services    # for services changes
```

Final:
```bash
make test.backend
```

---

## Out of Scope

- Routes/API endpoints (routes_app crate)
- Auth middleware (auth_middleware crate)
- Frontend (bodhi crate)
- `app_toolset_configs` table (unchanged)
- `app_client_toolset_configs` table (unchanged)
