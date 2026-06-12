# Review and Prepare Changes for Commit

## Summary

Review git diff for debugging statements vs. required changes, remove debugging code, and prepare staged changes for commit.

---

## Changes Overview

### Core Fixes (KEEP)

| File | Change | Status |
|------|--------|--------|
| `auth_middleware.rs` | AZP extraction from `validate_bearer_token` result instead of scope_claims | ✅ Keep |
| `token_service.rs` | `CachedExchangeResult` struct, updated return types, caching with azp | ✅ Keep |
| `tool_service.rs:773` | `toolset_id` → `id` JSON field lookup (THE FIX) | ✅ Keep |
| `db/objs.rs` | `Serialize` derive on `AppClientToolsetConfigRow` | ✅ Keep |

### Debug Statements to Remove

| File | Lines | Code | Action |
|------|-------|------|--------|
| `routes_login.rs` | 518-524 | `tracing::debug!("Cached app client toolset config...")` | ❌ Remove |
| `tool_service.rs` | 759-762 | `tracing::info!("No app-client toolset config found...")` | ❌ Remove |
| `tool_service.rs` | 772-779 | `tracing::info!("Found app-client toolsets...")` with JSON logging | ❌ Remove |

### Test Updates (KEEP - all are legitimate refactors)

- `chat-agentic.spec.mjs` - Refactored to use ChatPage methods
- `chat-toolsets.spec.mjs` - Consolidated 8 tests into 1 comprehensive test
- `toolsets-auth-restrictions.spec.mjs` - Fixed test order, use page object methods
- `toolsets-config.spec.mjs` - Removed try-catch per project convention
- Page objects (`ChatPage.mjs`, `ChatSettingsPage.mjs`, `ToolsetsPage.mjs`) - New helper methods

---

## Phase cleanup-rust: Remove Debug Statements

### File: `crates/routes_app/src/routes_login.rs`

**Remove lines 521-527** (the else block with debug logging):
```rust
// REMOVE:
} else {
  tracing::debug!(
    "Cached app client toolset config for {}: {}",
    request.app_client_id,
    serde_json::to_string_pretty(&config_row).unwrap_or_default()
  );
}
```

Result: Keep only the error case logging.

### File: `crates/services/src/tool_service.rs`

**Remove debug logging statements** (lines 759-762 and 772-779):

1. Remove "No app-client toolset config found" info log
2. Remove "Found app-client toolsets" info log with JSON pretty-print
3. Keep the `toolset_id` → `id` fix on line 773

**After cleanup**, the function should be:
```rust
let Some(config) = config else {
  return Ok(false);
};

// Parse toolsets from JSON field
let toolsets: Vec<serde_json::Value> =
  serde_json::from_str(&config.toolsets_json).unwrap_or_default();

Ok(toolsets.iter().any(|t| {
  t.get("id")
    .and_then(|v| v.as_str())
    .map(|id| id == toolset_id)
    .unwrap_or(false)
}))
```

---

## Phase format: Run cargo fmt

After removing debug statements, run:
```bash
cargo fmt --all
```

---

## Phase stage: Stage All Changes

```bash
git add crates/auth_middleware/src/auth_middleware.rs
git add crates/auth_middleware/src/token_service.rs
git add crates/routes_app/src/routes_login.rs
git add crates/services/src/db/objs.rs
git add crates/services/src/tool_service.rs
git add crates/lib_bodhiserver_napi/tests-js/
```

---

## Phase verify: Final Review

1. Run `git diff --staged` to review all staged changes
2. Verify no access tokens or sensitive data exposed in tracing statements
3. Verify `toolset_id` → `id` fix is present
4. Verify tests have no if-else or try-catch (per project convention)

---

## Verification Commands

```bash
# Backend tests
cargo test -p auth_middleware
cargo test -p services
cargo test -p routes_app

# NAPI tests (if needed)
cd crates/lib_bodhiserver_napi && npm run test
```

---

## Files Modified Summary

| File | Action |
|------|--------|
| `crates/auth_middleware/src/auth_middleware.rs` | Keep - AZP extraction fix |
| `crates/auth_middleware/src/token_service.rs` | Keep - Token service refactor with azp |
| `crates/routes_app/src/routes_login.rs` | Edit - Remove debug logging |
| `crates/services/src/db/objs.rs` | Keep - Serialize derive |
| `crates/services/src/tool_service.rs` | Edit - Remove debug, keep `id` fix |
| `tests-js/pages/*.mjs` | Keep - Page object updates |
| `tests-js/specs/**/*.spec.mjs` | Keep - Test consolidations and fixes |
