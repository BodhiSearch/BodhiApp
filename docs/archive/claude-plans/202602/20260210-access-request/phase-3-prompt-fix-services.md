# Fix `services` Crate Compilation Errors After Phase 3 Migration

## Context

We just completed Phase 3 of the access-request implementation which migrated the codebase from scope-based toolset identification (using `scope_uuid`/`scope`) to a simpler `tool_type` identifier system, and renamed database columns from `tools_requested`/`tools_approved` to `requested`/`approved`.

The implementation phase successfully updated production code, but the test code was not fully migrated and now has compilation errors.

## Files Changed in Phase 3

**Key schema/type changes:**
- `AppAccessRequestRow`: Fields renamed `tools_requested` → `requested`, `tools_approved` → `approved`
- `AccessRequestRepository::update_approval()`: Parameter `access_request_scope` changed from `&str` to `Option<&str>` to support auto-approve flow which doesn't return this value
- `ToolService`: All methods changed from `scope_uuid` parameter to `tool_type`
- Database migrations 0011 (modified) and 0012 (new) updated column names and added tool_type support

## Compilation Errors to Fix

Running `cargo test -p services` produces **18 compilation errors** in 3 categories:

### 1. Missing Import: `StubNetworkService` (2 errors)

**File:** `crates/services/src/test_utils/app.rs:12`

```
error[E0432]: unresolved import `crate::StubNetworkService`
  --> crates/services/src/test_utils/app.rs:12:25
   |
12 |   SqliteSessionService, StubNetworkService, ToolService, BODHI_EXEC_LOOKUP_PATH,
   |                         ^^^^^^^^^^^^^^^^^^
```

**Root cause:** `StubNetworkService` is defined in `crates/services/src/network_service.rs` with `#[cfg(feature = "test-utils")]` but the import path in test_utils/app.rs is incorrect.

**Location of StubNetworkService:** `crates/services/src/network_service.rs:36-48`
```rust
#[cfg(feature = "test-utils")]
pub struct StubNetworkService {
  pub ip: Option<String>,
}
```

**Fix needed:** Change import in `test_utils/app.rs:12` from `crate::StubNetworkService` to `crate::network_service::StubNetworkService` OR ensure conditional re-export in lib.rs.

### 2. Mockall Lifetime Issue: `Option<&str>` in Mock (2 errors)

**File:** `crates/services/src/test_utils/db.rs:703`

```
error[E0106]: missing lifetime specifier
   --> crates/services/src/test_utils/db.rs:703:36
    |
703 |       access_request_scope: Option<&str>,
    |                                    ^ expected named lifetime parameter
```

**Root cause:** The `MockDbService` mock definition uses `Option<&str>` parameter in `update_approval` method, but mockall requires explicit lifetime annotations for reference parameters in mocks.

**Current signature (failing):**
```rust
async fn update_approval(
  &self,
  id: &str,
  user_id: &str,
  approved: &str,
  resource_scope: &str,
  access_request_scope: Option<&str>,  // ❌ needs lifetime
) -> Result<AppAccessRequestRow, DbError>;
```

**Fix needed:** Add `for<'a>` higher-ranked trait bound or use `Option<String>` instead of `Option<&str>` in the mock definition. The actual trait in `access_request_repository.rs:12-19` uses `Option<&str>` correctly.

**Reference files:**
- Trait definition: `crates/services/src/db/access_request_repository.rs:12-19`
- Mock definition: `crates/services/src/test_utils/db.rs:697-704`
- Mock implementation: `crates/services/src/test_utils/db.rs:582-595`

### 3. Test Code Using Old Field Names (14 errors)

**Files:** `crates/services/src/db/tests.rs` (multiple test functions)

**Errors:**
- `E0560`: struct `AppAccessRequestRow` has no field named `tools_requested` (5 occurrences)
- `E0560`: struct `AppAccessRequestRow` has no field named `tools_approved` (5 occurrences)
- `E0609`: no field `tools_requested` on type `AppAccessRequestRow` (2 occurrences)
- `E0609`: no field `tools_approved` on type `AppAccessRequestRow` (3 occurrences)
- `E0308`: passing `&str` instead of `Option<&str>` to `update_approval` (1 occurrence)

**Affected tests:**
1. `test_create_draft_request` (lines 1213-1253)
2. `test_get_request` (lines 1259-1294)
3. `test_update_approval` (lines 1318-1370)
4. `test_update_denial` (lines 1375-1412)
5. `test_update_failure` (lines 1417-1455)

**Fix needed:**
- Change `tools_requested: "..."` → `requested: "..."`
- Change `tools_approved: None` → `approved: None`
- Change `result.tools_requested` → `result.requested`
- Change `result.tools_approved` → `result.approved`
- Change `"scope_access_request:..."` → `Some("scope_access_request:...")`

**Example fix from `test_create_draft_request` (line 1221-1236):**
```rust
// BEFORE:
let row = AppAccessRequestRow {
  id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
  app_client_id: "app-abc123".to_string(),
  flow_type: "redirect".to_string(),
  redirect_uri: Some("https://example.com/callback".to_string()),
  status: "draft".to_string(),
  tools_requested: r#"[{"tool_type":"builtin-exa-search"}]"#.to_string(),  // ❌ OLD
  tools_approved: None,  // ❌ OLD
  user_id: None,
  resource_scope: None,
  access_request_scope: None,
  error_message: None,
  expires_at: expires_at.timestamp(),
  created_at: now.timestamp(),
  updated_at: now.timestamp(),
};

// AFTER:
let row = AppAccessRequestRow {
  id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
  app_client_id: "app-abc123".to_string(),
  app_name: None,  // ✅ NEW
  app_description: None,  // ✅ NEW
  flow_type: "redirect".to_string(),
  redirect_uri: Some("https://example.com/callback".to_string()),
  status: "draft".to_string(),
  requested: r#"[{"tool_type":"builtin-exa-search"}]"#.to_string(),  // ✅ NEW
  approved: None,  // ✅ NEW
  user_id: None,
  resource_scope: None,
  access_request_scope: None,
  error_message: None,
  expires_at: expires_at.timestamp(),
  created_at: now.timestamp(),
  updated_at: now.timestamp(),
};
```

**Example fix from `test_update_approval` (line 1352):**
```rust
// BEFORE:
.update_approval(
  &row.id,
  "user-uuid",
  tools_approved_json,
  "scope_resource-xyz",
  "scope_access_request:550e8400-e29b-41d4-a716-446655440002",  // ❌ OLD
)

// AFTER:
.update_approval(
  &row.id,
  "user-uuid",
  tools_approved_json,
  "scope_resource-xyz",
  Some("scope_access_request:550e8400-e29b-41d4-a716-446655440002"),  // ✅ NEW
)
```

## Actual Field Definitions

**Reference:** `crates/services/src/db/objs.rs:222-239`
```rust
pub struct AppAccessRequestRow {
  pub id: String,
  pub app_client_id: String,
  pub app_name: Option<String>,       // ← NEW
  pub app_description: Option<String>, // ← NEW
  pub flow_type: String,
  pub redirect_uri: Option<String>,
  pub status: String,
  pub requested: String,              // ← RENAMED from tools_requested
  pub approved: Option<String>,       // ← RENAMED from tools_approved
  pub user_id: Option<String>,
  pub resource_scope: Option<String>,
  pub access_request_scope: Option<String>,
  pub error_message: Option<String>,
  pub expires_at: i64,
  pub created_at: i64,
  pub updated_at: i64,
}
```

## Task

Fix all 18 compilation errors in the `services` crate:

1. **Fix StubNetworkService import** in `test_utils/app.rs:12`
2. **Fix mockall lifetime issue** in `test_utils/db.rs:697-704`
3. **Update 5 test functions** in `db/tests.rs` to use new field names (`requested`, `approved`) and add new fields (`app_name`, `app_description`)
4. **Fix update_approval call** in `test_update_approval` to pass `Some(&str)` instead of `&str`

## Verification

After fixes, run:
```bash
cargo check -p services
cargo test -p services
```

All should compile and pass.

## Additional Context Files

- Plan document: `ai-docs/claude-plans/magical-plotting-chipmunk.md`
- Q&A context: `ai-docs/claude-plans/20260210-access-request/phase-3-ctx.md`
- Migration 0011: `crates/services/migrations/0011_app_access_requests.up.sql`
- Migration 0012: `crates/services/migrations/0012_toolsets_scope_to_tool_type.up.sql`
- AccessRequestService: `crates/services/src/access_request_service/service.rs`
- AccessRequestRepository trait: `crates/services/src/db/access_request_repository.rs`
