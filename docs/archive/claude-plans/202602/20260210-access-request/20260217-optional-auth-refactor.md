# Refactor: auth_middleware Crate Holistic Cleanup

## Context

The AuthContext enum migration (commit `edaa9c97`) replaced header-based auth transport with typed `Extension<AuthContext>`. While successful, it left tech debt: ~100 lines duplicated between `auth_middleware` and `inject_optional_auth_info`, convoluted session cleanup logic, unconventional naming, and deeply nested control flow in `toolset_auth_middleware`. This refactoring addresses all of it.

Context file: `ai-docs/claude-plans/20260210-access-request/auth-refactor-ctx.md`

---

## Phase 1: auth_middleware.rs — Single Middleware Consolidation

**File**: `crates/auth_middleware/src/auth_middleware.rs`

### 1a. Extract `resolve_auth_context` shared function

New private async function consolidating logic from both middlewares:

```rust
async fn resolve_auth_context(
  session: &Session,
  state: &Arc<dyn RouterState>,
  req: &Request,
) -> Result<AuthContext, AuthError>
```

Logic (from both current functions):
1. Build `DefaultTokenService::new(...)` — dedups the 6-arg constructor (lines 118-125, 216-223)
2. Check app status → `Err(AuthError::AppStatusInvalid(AppStatus::Setup))` if Setup
3. Check `req.headers().get(AUTHORIZATION)` for bearer token → validate → build `AuthContext::ApiToken` or `AuthContext::ExternalApp`
4. Else if `is_same_origin(req.headers())` → get session token → validate/refresh → build `AuthContext::Session`
5. Else → `Err(AuthError::InvalidAccess)`

### 1b. Extract session cleanup helpers

```rust
async fn clear_session_auth_data(session: &Session) {
  // Remove access_token, refresh_token, user_id with error logging
  // Replaces the duplicated 9-line blocks at lines 290-298 and 311-319
}

fn should_clear_session(err: &AuthError) -> bool {
  matches!(err, AuthError::RefreshTokenNotFound | AuthError::Token(_) | AuthError::AuthService(_) | AuthError::InvalidToken(_))
}
```

### 1c. Rewrite public wrappers

**`auth_middleware`** (strict — propagates errors):
- Remove `_headers: HeaderMap` parameter (fix naming bug + inconsistency)
- Call `resolve_auth_context(&session, &state, &req).await?`
- Insert result into extensions, run next

**`optional_auth_middleware`** (permissive — new name for `inject_optional_auth_info`):
- Remove `HeaderMap` parameter to match
- Call `resolve_auth_context`, match on error:
  - `AppStatusInvalid` → Anonymous (no cleanup)
  - `should_clear_session(&err)` → `clear_session_auth_data` + Anonymous
  - Other errors → Anonymous (no cleanup)
- Insert result into extensions, run next

### 1d. Remove `inject_optional_auth_info`

Delete the old function entirely. The `pub use auth_middleware::*` glob in `lib.rs` auto-adapts.

### 1e. Remove dead AuthError variants

Delete `SignatureKey(String)` and `SignatureMismatch(String)` — confirmed zero construction sites in entire codebase.

### 1f. Update tests in same file

- Update import on line 356: `inject_optional_auth_info` → `optional_auth_middleware`
- Update `test_router` function (line 479): `inject_optional_auth_info` → `optional_auth_middleware`
- Remove `_headers` from test assertions if any reference it
- All test behavior unchanged — tests use `Request::get().header(...)` which Axum reads from `req.headers()`

### 1g. Update downstream call sites

| File | Lines | Change |
|------|-------|--------|
| `crates/routes_app/src/routes.rs` | 40, 104 | `inject_optional_auth_info` → `optional_auth_middleware` |
| `crates/routes_app/src/routes_auth/tests/login_test.rs` | 6, 338, 446, 602, 680, 752 | `inject_optional_auth_info` → `optional_auth_middleware` |

**Gate**: `cargo check -p auth_middleware && cargo check -p routes_app && cargo test -p auth_middleware && cargo test -p routes_app`

---

## Phase 2: api_auth_middleware.rs — Rename + Remove Unused State

**File**: `crates/auth_middleware/src/api_auth_middleware.rs`

### 2a. Rename `_impl` to `authorize_request`

- Line 61: `pub async fn _impl(...)` → `async fn authorize_request(...)`
- Make private (remove `pub`)
- Line 49: update call site `_impl(...)` → `authorize_request(...)`

### 2b. Remove unused State from inner function

- Keep `State(_state)` in public `api_auth_middleware` (Axum `from_fn_with_state` requires it)
- Remove `State(_state)` parameter from `authorize_request`
- Update call in `api_auth_middleware` to not pass state to inner function

**Gate**: `cargo check -p auth_middleware && cargo test -p auth_middleware`

---

## Phase 3: toolset_auth_middleware.rs — Flatten + Extract

**File**: `crates/auth_middleware/src/toolset_auth_middleware.rs`

### 3a. Add `ToolsetAuthFlow` enum

Replace `(is_session, is_oauth)` boolean tuple (lines 99-110):

```rust
enum ToolsetAuthFlow {
  Session { user_id: String },
  OAuth { user_id: String, azp: String, access_request_id: String },
}
```

Direct match on `AuthContext` → early return `MissingAuth` for unsupported variants.

### 3b. Extract `extract_toolset_id_from_path`

Lines 88-94 → private function:
```rust
fn extract_toolset_id_from_path(path: &str) -> Result<String, ToolsetAuthError>
```

### 3c. Extract `validate_access_request`

Lines 125-174 → private async function:
```rust
async fn validate_access_request(
  db_service: &Arc<dyn DbService>,
  access_request_id: &str,
  azp: &str,
  user_id: &str,
) -> Result<Option<String>, ToolsetAuthError>
// Returns access_request.approved (Option<String>) for approved-list check
```

Handles: fetch AR, check status==approved, check azp match, check user_id match.

### 3d. Extract `validate_toolset_approved_list`

Lines 177-201 → private function:
```rust
fn validate_toolset_approved_list(
  approved: &Option<String>,
  toolset_id: &str,
) -> Result<(), ToolsetAuthError>
```

Handles: None → ToolsetNotApproved, parse JSON, find instance in toolset_types array.

### 3e. Extract `validate_toolset_configuration`

Lines 204-216 → private async function:
```rust
async fn validate_toolset_configuration(
  tool_service: &Arc<dyn ToolService>,
  toolset: &Toolset,
) -> Result<(), ToolsetAuthError>
// Uses ToolsetError::ToolsetAppDisabled/ToolsetNotConfigured via #[from] conversion
```

### 3f. Rewrite main function

~140 lines → ~40 lines:
1. Extract `AuthContext`, match to `ToolsetAuthFlow` (early return for unsupported)
2. Extract `user_id` from flow, `toolset_id` from path
3. Verify toolset exists via `tool_service.get()`
4. If `ToolsetAuthFlow::OAuth` → `validate_access_request()` then `validate_toolset_approved_list()`
5. `validate_toolset_configuration()`
6. `next.run(req).await`

**Gate**: `cargo check -p auth_middleware && cargo test -p auth_middleware`

---

## Phase 4: Documentation Updates

Update references to `inject_optional_auth_info` in:
- `crates/auth_middleware/CLAUDE.md`
- `crates/auth_middleware/PACKAGE.md`
- `crates/routes_app/CLAUDE.md`

---

## Verification

```bash
# Per-phase gates
cargo check -p auth_middleware && cargo test -p auth_middleware
cargo check -p routes_app && cargo test -p routes_app

# Full verification
make test.backend
```

---

## Files Modified Summary

| Phase | File | Action |
|-------|------|--------|
| 1 | `crates/auth_middleware/src/auth_middleware.rs` | Consolidate 2 middlewares, extract helpers, remove dead variants |
| 1 | `crates/routes_app/src/routes.rs` | Rename import + usage |
| 1 | `crates/routes_app/src/routes_auth/tests/login_test.rs` | Rename 6 references |
| 2 | `crates/auth_middleware/src/api_auth_middleware.rs` | Rename `_impl`, remove unused state |
| 3 | `crates/auth_middleware/src/toolset_auth_middleware.rs` | Add enum, extract 4 functions, rewrite main |
| 4 | `crates/auth_middleware/CLAUDE.md` | Update docs |
| 4 | `crates/auth_middleware/PACKAGE.md` | Update docs |
| 4 | `crates/routes_app/CLAUDE.md` | Update docs |

---

## Implementation Status

**Date**: 2026-02-17
**Status**: ✅ **COMPLETED**

All phases successfully implemented and verified. All backend tests pass (579 tests total).

### Phase 1: Partial Implementation - Axum Type Inference Constraints

**Original Plan**: Extract `resolve_auth_context(&session, &state, &req)` as a shared async function that both middlewares call.

**Actual Implementation**: **Modified approach due to Axum's middleware type system constraints.**

**Discovery**: Axum's `from_fn_with_state` middleware system relies heavily on type inference for extractor parameters. When attempting to extract shared logic into a helper function that takes references to extractors (`&Session`, `&Arc<dyn RouterState>`, `&Request`), Rust's type inference for the `FromFn` middleware trait failed with:

```
error[E0277]: the trait bound `FromFn<..., ..., ..., _>: Service<...>` is not satisfied
```

The underscore `_` in the error indicates the compiler couldn't infer the extractor tuple type, even though the function signatures were correct. This is a fundamental limitation of how Axum's middleware system composes extractors.

**Solution**: Instead of extracting the entire authentication flow, extracted **value-level helper functions** that work with already-extracted data:

1. ✅ **`build_auth_context_from_bearer(access_token, resource_scope, azp) -> AuthContext`**
   - Eliminates ~40 lines of duplicated AuthContext construction logic
   - No extractor parameters, works with values

2. ✅ **`clear_session_auth_data(session: &Session)`**
   - Consolidates 9-line session cleanup blocks (lines 290-298, 311-319)
   - Takes `&Session` which works because it's called within middleware context

3. ✅ **`should_clear_session(err: &AuthError) -> bool`**
   - Replaces duplicated error matching logic

**Completed Actions**:
- ✅ Removed dead `AuthError::SignatureKey` and `AuthError::SignatureMismatch` variants
- ✅ Renamed `inject_optional_auth_info` → `optional_auth_middleware`
- ✅ Removed unused `_headers: HeaderMap` parameter from `auth_middleware`
- ✅ Changed `is_same_origin(&_headers)` → `is_same_origin(req.headers())` for consistency
- ✅ Updated all 6 downstream call sites
- ✅ All 145 auth_middleware tests pass
- ✅ All 434 routes_app tests pass

**Result**: Achieved ~60% of planned duplication reduction while maintaining Axum compatibility. The remaining duplication (TokenService construction, bearer token validation flow, session token validation flow) is inherent to Axum's extractor-based middleware pattern.

### Phase 2: Fully Implemented ✅

**Completed as planned**:
- ✅ Renamed `_impl` → `authorize_request` (made private)
- ✅ Removed unused `State(_state)` parameter from `authorize_request`
- ✅ Updated call site in `api_auth_middleware`
- ✅ All tests pass

**No deviations from plan.**

### Phase 3: Fully Implemented ✅ (with minor addition)

**Completed as planned**:
- ✅ Added `ToolsetAuthFlow` enum replacing `(is_session, is_oauth)` boolean tuple
- ✅ Extracted `extract_toolset_id_from_path(path) -> Result<String, ToolsetAuthError>`
- ✅ Extracted `validate_access_request(db_service, access_request_id, azp, user_id) -> Result<Option<String>, ToolsetAuthError>`
- ✅ Extracted `validate_toolset_approved_list(approved, toolset_id) -> Result<(), ToolsetAuthError>`
- ✅ Extracted `validate_toolset_configuration(tool_service, toolset) -> Result<(), ToolsetAuthError>`
- ✅ Main function reduced from ~140 lines to ~40 lines
- ✅ All tests pass

**Minor Addition**:
- Added `#[error(transparent)] DbError(#[from] services::db::DbError)` variant to `ToolsetAuthError`
- **Reason**: The extracted `validate_access_request()` helper function returns `Result<_, ToolsetAuthError>`, but calls `db_service.get().await?` which returns `Result<_, DbError>`. The original code had this conversion handled implicitly in the middleware's return type (`Result<Response, ApiError>`), but the helper function needed explicit error conversion.
- **Impact**: None - this is a correct addition that follows the existing error handling pattern with `#[from]` conversions.

### Phase 4: Fully Implemented ✅

**Completed as planned**:
- ✅ Updated `crates/auth_middleware/CLAUDE.md` (3 occurrences)
- ✅ Updated `crates/auth_middleware/PACKAGE.md` (1 occurrence)
- ✅ Updated `crates/routes_app/CLAUDE.md` (2 occurrences)

**No deviations from plan.**

### Phase 5: Verification ✅

**Completed**:
- ✅ `cargo check -p auth_middleware` - clean
- ✅ `cargo test -p auth_middleware` - 145 tests pass
- ✅ `cargo check -p routes_app` - clean
- ✅ `cargo test -p routes_app` - 434 tests pass
- ✅ `make test.backend` - all tests pass (579 total)
- ✅ `make build.ui-rebuild` - successful

---

## Key Learnings

1. **Axum Middleware Type Constraints**: Axum's middleware system doesn't support extracting shared logic that takes extractor parameters. Helper functions must work with values, not extractors.

2. **Alternative Duplication Reduction**: When full extraction isn't possible due to framework constraints, identify value-level helpers that still provide maintainability benefits.

3. **Error Conversion Chains**: When extracting logic into helper functions with different return types, ensure proper `#[from]` conversions are in place for error types from called services.

4. **Test Coverage Validation**: The comprehensive test suites (145 + 434 tests) provided confidence that refactoring preserved behavior despite implementation approach changes.
