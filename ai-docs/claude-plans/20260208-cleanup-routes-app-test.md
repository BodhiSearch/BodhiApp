# Plan: Revamp routes_app Testing for Uniformity

## Context

The `routes_app` crate contains all route handlers (app routes, OpenAI routes via `routes_oai/` module, and Ollama routes). Testing evolved organically, resulting in 3+ different fixture setup patterns, inconsistent request construction, brittle error assertions (exact JSON message matching), duplicated macros, and mixed test file organization. This plan standardizes all route testing to a canonical pattern, extracts shared test utilities, and fills coverage gaps.

**Scope**: All `routes_app` test modules EXCEPT `routes_ollama/` (excluded per user request).

## Decisions Made

| Decision | Choice |
|----------|--------|
| Router scope | Minimal single-handler (pure unit test) |
| Auth testing | Bypass auth; out-of-scope: routes_all integration tests with auth middleware |
| Test data | Shared canonical fixtures in `routes_app/src/test_utils/`, exported via `test-utils` feature |
| Parameterization | Per-module decision based on security profile |
| SSE helpers | Already in `server_core::test_utils` (`ResponseTestExt::sse()`) |
| Request helpers | Use `RequestTestExt` + new `RequestAuthExt` in `server_core::test_utils` |
| Test file org | Threshold-based: inline for <5 tests, separate `tests/` dir for >=5 |
| Coverage tool | `cargo-llvm-cov` per-crate |
| Migration | Module-by-module, sequential sub-agent phases |
| Auth helpers | Add `.with_user_auth(token, role)` and `.with_api_token(token, scope)` to `RequestAuthExt` |
| Priority | Uniformity first, then coverage gaps |
| Test naming | `test_<handler>_<scenario>` convention |
| Error assertions | Check error **code** (stable), not exact message text (brittle) |
| pretty_assertions | Use everywhere |
| Test annotations | `#[rstest]` + `#[awt]` + `#[anyhow_trace]` consistently on all async tests |
| Test module naming | `mod tests` (not `mod test`) |

## Canonical Test Pattern

```rust
// In separate test file (e.g., routes_oai/tests/chat_test.rs):
use pretty_assertions::assert_eq;
use anyhow_trace::anyhow_trace;
use rstest::rstest;
use axum::{body::Body, http::{Request, StatusCode}, routing::post, Router};
use server_core::test_utils::{RequestTestExt, RequestAuthExt, ResponseTestExt};
use server_core::{DefaultRouterState, MockSharedContext};
use services::test_utils::AppServiceStubBuilder;
use std::sync::Arc;
use tower::ServiceExt;

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_handler_name_success() -> anyhow::Result<()> {
  // Setup - minimal single-handler router
  let app_service = AppServiceStubBuilder::default()
    .with_data_service().await
    .build()?;
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));
  let router = Router::new()
    .route("/path", post(handler_under_test))
    .with_state(state);

  // Act
  let response = router
    .oneshot(Request::post("/path").json(payload)?)
    .await?;

  // Assert
  assert_eq!(StatusCode::OK, response.status());
  let result = response.json::<ExpectedType>().await?;
  assert_eq!(expected, result);
  Ok(())
}

#[rstest]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_handler_name_error_scenario() -> anyhow::Result<()> {
  // ... setup ...
  let response = router.oneshot(request).await?;
  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let body = response.json::<serde_json::Value>().await?;
  // Check error CODE only, not message text
  assert_eq!("error_enum-variant_name", body["error"]["code"].as_str().unwrap());
  Ok(())
}
```

---

## Execution Phases

Each phase is executed by a sub-agent that:
1. Makes the specified changes
2. Runs `cargo test -p routes_app` (or more targeted filter) to verify
3. Returns a summary of changes made

### Phase 0: Coverage Baseline

**Goal**: Establish baseline coverage numbers before any changes.

**Actions**:
1. Install `cargo-llvm-cov` if not present
2. Run `cargo llvm-cov --package routes_app --text`
3. Document baseline in `ai-docs/claude-plans/test-revamp-baseline.md` with per-file coverage

**Verification**: `cargo test -p routes_app` (no code changes)

---

### Phase 1: Infrastructure - Add `RequestAuthExt` to server_core

**Goal**: Add auth header helper trait used by all subsequent phases.

**Changes**:

#### 1a. Add `RequestAuthExt` trait

**File**: `crates/server_core/src/test_utils/http.rs`

Add new trait (separate from `RequestTestExt` since auth methods return `Builder`, not `Request<Body>`):

```rust
pub trait RequestAuthExt {
  fn with_user_auth(self, token: &str, role: &str) -> Builder;
  fn with_api_token(self, token: &str, scope: &str) -> Builder;
}

impl RequestAuthExt for Builder {
  fn with_user_auth(self, token: &str, role: &str) -> Builder {
    self
      .header(auth_middleware::KEY_HEADER_BODHIAPP_TOKEN, token)
      .header(auth_middleware::KEY_HEADER_BODHIAPP_ROLE, role)
  }
  fn with_api_token(self, token: &str, scope: &str) -> Builder {
    self
      .header(auth_middleware::KEY_HEADER_BODHIAPP_TOKEN, token)
      .header(auth_middleware::KEY_HEADER_BODHIAPP_SCOPE, scope)
  }
}
```

**File**: `crates/server_core/Cargo.toml`

Add `auth_middleware` as optional dependency and include in `test-utils` feature:
```toml
[dependencies]
auth_middleware = { workspace = true, optional = true }

[features]
test-utils = [
  ...,
  "auth_middleware",
]
```

**File**: `crates/server_core/src/test_utils/mod.rs`

Export `RequestAuthExt`.

#### 1b. Remove duplicated `wait_for_event!` macro

**File**: `crates/routes_app/src/routes_models/tests/pull_test.rs`
- Remove local `wait_for_event!` macro definition
- Add `use crate::wait_for_event;` import (macro is `#[macro_export]` in `crate::test_utils`)

**Verification**: `cargo test -p server_core && cargo test -p routes_app`

---

### Phase 2: Migrate `routes_oai` Tests (10 tests, 2 files)

**Goal**: Standardize routes_oai tests to canonical pattern.

#### 2a. `routes_oai/tests/chat_test.rs` (3 tests)
**File**: `crates/routes_app/src/routes_oai/tests/chat_test.rs`

Current issues:
- Missing `use pretty_assertions::assert_eq;`
- Has `#[rstest]` but missing `#[awt]` on all 3 tests
- Test names prefixed with `test_routes_` instead of `test_<handler>_`
- Uses `.unwrap()` instead of `?` in several places

Changes:
- Add `use pretty_assertions::assert_eq;`
- Add `#[awt]` to all 3 tests
- Rename: `test_routes_chat_completions_non_stream` -> `test_chat_completions_handler_non_stream`, `test_routes_chat_completions_stream` -> `test_chat_completions_handler_stream`, `test_routes_embeddings_non_stream` -> `test_embeddings_handler_non_stream`
- Replace `.unwrap()` with `?` where test returns `anyhow::Result`

#### 2b. `routes_oai/tests/models_test.rs` (7 tests)
**File**: `crates/routes_app/src/routes_oai/tests/models_test.rs`

Current issues:
- 4 tests (`test_oai_models_handler_api_alias_*`) use bare `#[tokio::test]` without `#[rstest]` or `#[anyhow_trace]`
- Uses `create_router()` helper that creates router with 2 handlers (not minimal single-handler)
- Uses `Request::builder().uri(...).body(Body::empty())?` (verbose) instead of `Request::get()`

Changes:
- Add `#[rstest]`, `#[awt]`, `#[anyhow_trace]` to all 7 tests
- Rename: `test_oai_models_handler` -> `test_oai_models_handler_list_all`, `test_oai_model_handler` -> `test_oai_model_handler_found`
- Keep `create_router()` helper (tests both models endpoints which share the same router in production; splitting would lose coverage of route path config)

**Verification**: `cargo test -p routes_app -- routes_oai`

---

### Phase 3: Migrate `routes_app::routes_setup` Tests

**File**: `crates/routes_app/src/routes_setup_test.rs` (~9 tests)

- Ensure `#[anyhow_trace]` on all test functions
- Add `#[rstest]` + `#[awt]` where missing
- Convert brittle error assertions to error code checks
- Replace manual `Request::post().header("Content-Type",...).body(Body::from(...))` with `Request::post().json(...)` via `RequestTestExt`

**Verification**: `cargo test -p routes_app -- routes_setup`

---

### Phase 4: Migrate `routes_app::routes_settings` Tests

**File**: `crates/routes_app/src/routes_settings_test.rs` (~9 tests)

- Convert remaining brittle error assertions to error code checks
- Minor cleanup (already well-structured)

**Verification**: `cargo test -p routes_app -- routes_settings`

---

### Phase 5: Migrate `routes_app::routes_api_token` Tests

**File**: `crates/routes_app/src/routes_api_token_test.rs` (~12-20 tests)

- Add `#[anyhow_trace]` where missing
- Replace manual `.header(KEY_HEADER_BODHIAPP_TOKEN, &token).header(KEY_HEADER_BODHIAPP_ROLE, ...)` with `.with_user_auth(token, role)` from `RequestAuthExt`
- Convert brittle error assertions to error code checks

**Verification**: `cargo test -p routes_app -- routes_api_token`

---

### Phase 6: Migrate `routes_app::routes_users` Tests (3 files, ~24 tests)

#### 6a. `routes_users/tests/user_info_test.rs` (~10 tests)
- Add `#[anyhow_trace]` on all tests
- Convert error assertions from full JSON to error code
- Use `RequestAuthExt` where applicable

#### 6b. `routes_users/tests/management_test.rs` (~5 tests)
- Add `use pretty_assertions::assert_eq;`
- Add `#[anyhow_trace]`
- Use `RequestAuthExt`

#### 6c. `routes_users/tests/access_request_test.rs` (~9 tests)
- Add `use pretty_assertions::assert_eq;`
- Add `#[anyhow_trace]`
- Use `RequestAuthExt`

**Verification**: `cargo test -p routes_app -- routes_users`

---

### Phase 7: Migrate `routes_app::routes_models` Tests (3 files, ~22 tests)

#### 7a. `routes_models/tests/aliases_test.rs` (~12 tests)
- Add `use pretty_assertions::assert_eq;`
- Add `#[anyhow_trace]`
- Convert error assertions to error code checks
- Replace manual `Content-Type` + `Body::from(...)` with `RequestTestExt::json()`

#### 7b. `routes_models/tests/metadata_test.rs` (~2 tests)
- Add `#[anyhow_trace]`
- Already well-structured, minor cleanup

#### 7c. `routes_models/tests/pull_test.rs` (~8 tests)
- Add `use pretty_assertions::assert_eq;`
- Add `#[anyhow_trace]`
- Verify `wait_for_event!` import from `crate::wait_for_event` (Phase 1 removed the local copy)
- Replace manual request construction with `RequestTestExt::json()`
- Convert error assertions to error code checks

**Verification**: `cargo test -p routes_app -- routes_models`

---

### Phase 8: Migrate `routes_app::routes_toolsets` Tests (~13 tests)

**File**: `crates/routes_app/src/routes_toolsets/tests/toolsets_test.rs`

- Add `use pretty_assertions::assert_eq;`
- Add `#[anyhow_trace]`
- Use `RequestAuthExt` for token/role/scope headers
- Convert error assertions to error code checks

**Verification**: `cargo test -p routes_app -- routes_toolsets`

---

### Phase 9: Migrate `routes_app::routes_api_models` Tests (~23 tests)

**File**: `crates/routes_app/src/routes_api_models/tests/api_models_test.rs`

- Add `#[anyhow_trace]`
- Convert brittle error assertions to error code checks

**Verification**: `cargo test -p routes_app -- routes_api_models`

---

### Phase 10: Migrate `routes_app::routes_auth` Tests

#### 10a. `routes_auth/tests/login_test.rs` (~13 tests)
- Add `#[anyhow_trace]` where missing
- Keep `TestServer` approach (needed for session/cookie management)
- Convert brittle error assertions to error code checks

#### 10b. `routes_auth/tests/request_access_test.rs` (~5-11 tests)
- Add `use pretty_assertions::assert_eq;`
- Add `#[anyhow_trace]`
- Use `RequestAuthExt`
- Convert brittle error assertions to error code checks

**Verification**: `cargo test -p routes_app -- routes_auth`

---

### Phase 11: Full Suite Verification

**Goal**: Run complete test suite to catch any cross-module regressions.

**Verification**: `cargo test -p server_core && cargo test -p routes_app`

---

### Phase 12: Coverage Gap Analysis and Fill

**Goal**: Re-run coverage, compare to Phase 0 baseline, fill critical gaps.

**Actions**:
1. Run `cargo llvm-cov --package routes_app --text`
2. Compare with baseline from Phase 0
3. For each uncovered security/domain-critical handler, add:
   - 1 success-path test
   - 1 primary error-path test
4. All new tests follow the canonical pattern
5. Skip `routes_ollama` coverage gaps (excluded from scope)

**Priority coverage targets**:
- `routes_oai`: validation error paths in `validate_chat_completion_request()`
- `routes_app`: any uncovered error branches in auth, token, and access control handlers

**Verification**: `cargo test -p routes_app`

---

## Critical Files Reference

| File | Role |
|------|------|
| `crates/server_core/src/test_utils/http.rs` | `ResponseTestExt`, `RequestTestExt`, new `RequestAuthExt` |
| `crates/server_core/src/test_utils/state.rs` | `router_state_stub`, `DefaultRouterState` |
| `crates/server_core/Cargo.toml` | Add `auth_middleware` optional dep for test-utils |
| `crates/services/src/test_utils/app.rs` | `AppServiceStubBuilder` - primary fixture builder |
| `crates/routes_app/src/test_utils/mod.rs` | `wait_for_event!` macro, shared constants |
| `crates/routes_app/Cargo.toml` | Already has all needed dev-dependencies |
| `crates/auth_middleware/src/auth_middleware.rs` | `KEY_HEADER_BODHIAPP_*` constants |

## Modules In Scope

| Module | Test Location | Approx Tests |
|--------|--------------|--------------|
| `routes_oai` | `routes_oai/tests/chat_test.rs`, `models_test.rs` | 10 |
| `routes_setup` | `routes_setup_test.rs` | 9 |
| `routes_settings` | `routes_settings_test.rs` | 9 |
| `routes_api_token` | `routes_api_token_test.rs` | 12-20 |
| `routes_users` | `routes_users/tests/*.rs` | 24 |
| `routes_models` | `routes_models/tests/*.rs` | 22 |
| `routes_toolsets` | `routes_toolsets/tests/*.rs` | 13 |
| `routes_api_models` | `routes_api_models/tests/*.rs` | 23 |
| `routes_auth` | `routes_auth/tests/*.rs` | 18-24 |

## Out of Scope

- `routes_ollama/` module (excluded per user request)
- Auth middleware integration tests in `routes_all` (separate task)
- Test data canonical fixtures (can be added incrementally)
- Frontend/UI test changes
