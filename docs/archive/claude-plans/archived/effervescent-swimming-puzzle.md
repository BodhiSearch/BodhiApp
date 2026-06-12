# Fix Failing E2E Tests: MCP OAuth, Auth Restrictions, and Toolset Auth

## Context

After the SeaORM migration (`7976a7fac`) and the approval status enum commit (`bb13c9f44`), 11 E2E tests are failing due to two root causes:

1. **ID format mismatch**: The SeaORM migration changed IDs from UUID (36 chars, e.g., `550e8400-e29b-41d4-a716-446655440000`) to ULID (26 chars, e.g., `01KJJN3GMPQJ7N03G3J5P7BHB1`). The `access_request_auth_middleware` still uses UUID-format matching to extract entity IDs from URL paths, causing all per-entity MCP and toolset endpoints to return 404 for external apps.

2. **Serde format mismatch**: The `RegistrationType` enum uses `snake_case` serialization (`pre_registered`), but the E2E test fixture sends `pre-registered` (with hyphen), causing HTTP 422 deserialization errors.

## Root Cause 1: `extract_uuid_from_path` doesn't handle ULID format

**File**: `crates/auth_middleware/src/access_request_auth_middleware/middleware.rs:80-86`

```rust
fn extract_uuid_from_path(path: &str) -> Result<String, AccessRequestAuthError> {
  path
    .split('/')
    .find(|seg| seg.len() == 36 && seg.contains('-'))  // UUID: 36 chars with hyphens
    .map(|s| s.to_string())
    .ok_or(AccessRequestAuthError::EntityNotFound)
}
```

ULID strings are 26 characters with no hyphens. This function always fails for ULID IDs.

**Affects**: Tests 14, 16, 24 (MCP auth restrictions), test 11 (toolset auth restrictions)

### Fix

Rename to `extract_id_from_path` and update the matching logic to handle **both** ULID and UUID formats (backwards-compatible for unit tests that still use UUID-format IDs):

```rust
fn extract_id_from_path(path: &str) -> Result<String, AccessRequestAuthError> {
  path
    .split('/')
    .find(|seg| {
      // ULID format: 26 Crockford Base32 characters
      (seg.len() == 26 && seg.chars().all(|c| c.is_ascii_alphanumeric()))
      // UUID format: 36 chars with hyphens (backwards compatibility)
      || (seg.len() == 36 && seg.contains('-'))
    })
    .map(|s| s.to_string())
    .ok_or(AccessRequestAuthError::EntityNotFound)
}
```

Update call sites at lines 180 and 219 in the same file.

**Note**: The existing unit tests in `tests.rs` use UUID-format IDs in both URL paths and approved JSON. Supporting both formats avoids needing to update all test fixtures while the migration is in progress.

## Root Cause 2: Test fixture `registration_type` format mismatch

**File**: `crates/lib_bodhiserver_napi/tests-js/fixtures/mcpFixtures.mjs:105`

```javascript
registration_type: 'pre-registered',  // ❌ hyphen
```

Backend `RegistrationType` enum uses `#[serde(rename_all = "snake_case")]`, so it expects `pre_registered` (underscore).

**Affects**: Tests 25, 26, 27, 28, 29, 30, 31 (all OAuth/DCR tests)

### Fix

Change line 105:
```javascript
registration_type: 'pre_registered',  // ✅ underscore
```

## Files to Modify

1. `crates/auth_middleware/src/access_request_auth_middleware/middleware.rs`
   - Rename `extract_uuid_from_path` → `extract_id_from_path`
   - Update matching logic to support both ULID (26 alphanumeric chars) and UUID (36 chars with hyphens)
   - Update call sites (lines 180, 219)

2. `crates/lib_bodhiserver_napi/tests-js/fixtures/mcpFixtures.mjs`
   - Line 105: `'pre-registered'` → `'pre_registered'`

## Verification

1. `cargo test -p auth_middleware` - Verify middleware unit tests still pass (they use UUID-format IDs)
2. `cargo test -p routes_app` - Verify route tests pass
3. `make test.backend` - Full backend test suite
4. Re-run failing E2E tests:
   ```
   cd crates/lib_bodhiserver_napi && npm run test:playwright -- --grep "@oauth|@mcps"
   ```
