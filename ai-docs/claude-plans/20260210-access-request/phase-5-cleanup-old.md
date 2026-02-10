# Phase 5: Remove Old Flow Code

## Purpose

Clean up all artifacts from the old server-to-server access request flow after the new user-approval flow is fully implemented and validated.

## Dependencies

- **Phase 4**: New auth middleware working with access_request_id validation

## Code to Remove

### 1. Middleware Artifacts

**File**: `crates/auth_middleware/src/auth_middleware.rs`

Remove:
- `X-BodhiApp-Tool-Scopes` header constant (if defined)
- Header injection logic for tool scopes in OAuth flow
- Any references to `scope_toolset-*` parsing in middleware

**File**: `crates/auth_middleware/src/toolset_auth_middleware.rs`

Should already be removed in Phase 4, but verify:
- No calls to `is_app_client_registered_for_toolset`
- No checks for `X-BodhiApp-Tool-Scopes` header

### 2. Service Layer Artifacts

**File**: `crates/services/src/auth_service.rs`

Should already be removed in Phase 2, but verify:
- `request_access()` method fully removed from trait and implementations
- `RequestAccessRequest` struct removed
- `RequestAccessResponse` struct removed

**File**: `crates/services/src/tool_service/service.rs`

Should already be removed in Phase 2, but verify:
- `is_app_client_registered_for_toolset()` method removed from trait
- Implementation removed
- Mock implementation removed

**File**: `crates/services/src/db/toolset_repository.rs`

Should already be removed in Phase 2, but verify:
- `get_app_client_toolset_config()` method removed
- `upsert_app_client_toolset_config()` method removed

**File**: `crates/services/src/db/objs.rs`

Should already be removed in Phase 2, but verify:
- `AppClientToolsetConfigRow` struct removed

### 3. Route Handler Artifacts

**File**: `crates/routes_app/src/routes_auth/request_access.rs`

Should already be rewritten in Phase 3, but verify:
- Old server-to-server handler completely replaced
- No references to `AppClientToolset` or old flow types

### 4. Domain Object Artifacts

**File**: `crates/objs/src/` (various)

Search and remove:
- `AppClientToolset` struct (if separate file)
- Any types related to old flow that aren't reused

### 5. Library Exports

**File**: `crates/services/src/lib.rs`

Remove exports:
- `RequestAccessRequest`
- `RequestAccessResponse`
- `AppClientToolset`
- `AppClientToolsetConfigRow`
- Any other old flow types

**File**: `crates/objs/src/lib.rs`

Remove exports:
- Old access request types if not reused

### 6. Test Code

**Search entire codebase** for:
- Tests referencing `request_access()` method
- Tests for `is_app_client_registered_for_toolset()`
- Tests for `app_client_toolset_configs` table operations
- Mock setups using old types

Either remove or update to use new flow.

## Verification Strategy

### 1. Compilation Check
```bash
cargo check --workspace
```
Should compile without errors after cleanup.

### 2. Grep for Old Artifacts
```bash
# Search for old method names
grep -r "request_access" crates/
grep -r "is_app_client_registered_for_toolset" crates/
grep -r "AppClientToolset" crates/
grep -r "app_client_toolset_configs" crates/

# Search for old table name in SQL/code
grep -r "app_client_toolset_configs" crates/services/migrations/
```

Should return no results (except in down migrations for rollback).

### 3. Test Suite
```bash
make test.backend
```
All tests should pass after cleanup.

### 4. Dead Code Warning Check
```bash
cargo build --workspace 2>&1 | grep "warning: unused"
```
Look for warnings about unused code related to old flow.

## Files to Review/Modify

| File | Action | What to Remove |
|------|--------|----------------|
| `crates/auth_middleware/src/auth_middleware.rs` | Review | Tool scopes header logic |
| `crates/auth_middleware/src/toolset_auth_middleware.rs` | Review | Old OAuth checks (should be done in Phase 4) |
| `crates/services/src/auth_service.rs` | Review | Old request_access method (should be done in Phase 2) |
| `crates/services/src/tool_service/service.rs` | Review | Old registration check (should be done in Phase 2) |
| `crates/services/src/db/toolset_repository.rs` | Review | Old app_client methods (should be done in Phase 2) |
| `crates/services/src/db/objs.rs` | Review | Old row types (should be done in Phase 2) |
| `crates/services/src/lib.rs` | Modify | Remove old exports |
| `crates/objs/src/lib.rs` | Modify | Remove old exports |
| `crates/routes_app/src/routes_auth/request_access.rs` | Review | Verify rewrite complete (Phase 3) |

## Research Questions

1. **ToolsetScope type**: Is `ToolsetScope` still used elsewhere? If yes, keep the type but remove middleware parsing logic.
2. **Scope constants**: Are there scope-related constants we should remove?
3. **Migration rollback**: Should we keep old table schema in down migration for rollback?
4. **Test fixtures**: Are there test fixtures or mock data referencing old types?
5. **Documentation**: Any inline comments or docs referencing old flow?

## Acceptance Criteria

- [ ] All old method signatures removed from traits
- [ ] All old implementations removed
- [ ] All old types and structs removed
- [ ] All old exports removed from lib.rs files
- [ ] Old test code removed or updated
- [ ] Grep searches return no results (except down migrations)
- [ ] `cargo check --workspace` succeeds
- [ ] `cargo clippy --workspace` shows no warnings about dead code for old flow
- [ ] `make test.backend` passes
- [ ] No compilation warnings about unused imports/functions related to old flow

## Notes for Sub-Agent

- **Be thorough**: Search entire codebase, not just obvious files
- **Check imports**: Remove unused imports after deleting code
- **Update tests**: Some tests may need updating to use new flow instead of removal
- **Migration history**: Keep down migrations intact for potential rollback
- **Documentation**: Update any README or doc comments referencing old flow
- **Git history**: Old code is preserved in git history if needed for reference

## Next Phase

Phase 6 will implement the frontend review/approve page to complete the user-facing flow.
