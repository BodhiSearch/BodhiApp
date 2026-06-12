# Plan: Restore Missing Auth Tests in routes_app

## Context

A test reorganization (commits `7e8432b8..f3222226`) moved auth tests from integration test files (`tests/*_auth_test.rs`) into their respective module test files (`src/*/tests/*_test.rs`). However, 8 modules were left with only stub comments (`// Auth tier tests merged (stub for plan completion)`) instead of actual test implementations. The original tests were deleted, leaving a gap of ~48 tests.

## Current State

- **Current test count**: 319 (`cargo test -p routes_app --lib`)
- **8 stub files** needing restoration (confirmed via grep)
- **5 modules already migrated** successfully (settings, api_token, setup, models×3) - these serve as canonical patterns
- **Test utilities available**: `build_test_router()`, `create_authenticated_session()`, `session_request()`, `unauth_request()` in `crate::test_utils`

## Auth Tier Mapping (from `routes.rs`)

| Tier | Role | Modules |
|------|------|---------|
| Optional | none required | user_info, auth (initiate/callback/request-access) |
| User | `ResourceRole::User` | OAI models, Ollama, bodhi models (read) |
| User Session | `ResourceRole::User`, session-only | Toolsets CRUD |
| PowerUser | `ResourceRole::PowerUser` | api-models, models (write), pull |
| Manager Session | `ResourceRole::Manager`, session-only | access-requests, users |
| Admin Session | `ResourceRole::Admin`, session-only | settings, toolset_types |

## Files to Modify (8 files, replace stub comments)

### 1. `src/routes_api_models/tests/api_models_test.rs` (line 1376)
**Auth tier**: PowerUser | **Deleted file**: `routes_api_models_auth_test.rs`

Replace stub with 3 test functions:
- `test_api_models_endpoints_reject_unauthenticated` - 9 endpoints via `#[case]`
- `test_api_models_endpoints_reject_insufficient_role` - `#[values]` cartesian: `"resource_user"` × 9 endpoints → `assert_eq!(FORBIDDEN)`
- `test_api_models_endpoints_allow_power_user_and_above` - `#[values]` cartesian: 3 roles × 2 safe GET endpoints (`/bodhi/v1/api-models`, `/bodhi/v1/api-models/api-formats`) → `assert_eq!(OK)`

Endpoints: GET/POST `/bodhi/v1/api-models`, GET/PUT/DELETE `/bodhi/v1/api-models/some_id`, POST `.../sync-models`, POST `.../test`, POST `.../fetch-models`, GET `.../api-formats`

**Safe endpoints for allow test**: `GET /bodhi/v1/api-models` (db_service returns empty list → 200), `GET /bodhi/v1/api-models/api-formats` (static data → 200). Other endpoints use `MockAiApiService` or need valid bodies.

### 2. `src/routes_oai/tests/models_test.rs` (line 309)
**Auth tier**: User | **Deleted file**: `routes_oai_auth_test.rs`

Replace stub with 2 test functions:
- `test_oai_endpoints_reject_unauthenticated` - 4 endpoints via `#[case]`: GET `/v1/models`, GET `/v1/models/some_model`, POST `/v1/chat/completions`, POST `/v1/embeddings`
- `test_oai_endpoints_allow_all_roles` - `#[values]` cartesian: 4 roles × `("GET", "/v1/models")` → `assert_eq!(OK)` (DataService returns empty list)

Only `GET /v1/models` is safe for allow test. `chat/completions` and `embeddings` use `MockSharedContext`. `GET /v1/models/some_model` returns 404 (could also test if desired).

### 3. `src/routes_ollama/tests/handlers_test.rs` (line 88, outside `mod test`)
**Auth tier**: User | **Deleted file**: `routes_ollama_auth_test.rs`

Replace stub with 2 test functions:
- `test_ollama_endpoints_reject_unauthenticated` - 3 endpoints via `#[case]`: GET `/api/tags`, POST `/api/show`, POST `/api/chat`
- `test_ollama_endpoints_allow_all_roles` - `#[values]` cartesian: 4 roles × `("GET", "/api/tags")` → `assert_eq!(OK)` (DataService returns empty list)

Only `GET /api/tags` safe. `POST /api/show` needs body, `POST /api/chat` uses `MockSharedContext`.

### 4. `src/routes_toolsets/tests/toolsets_test.rs` (line 829)
**Auth tier**: User (session-only) | **Deleted file**: `routes_toolsets_auth_test.rs`

Replace stub with 1 test function:
- `test_toolset_endpoints_reject_unauthenticated` - 6 endpoints via `#[case]`: POST/GET/PUT/DELETE `/bodhi/v1/toolsets[/{id}]`, GET `/bodhi/v1/toolsets`, POST `.../execute/some_method`

No allow test needed - all handlers use `MockToolService` which panics without expectations. Auth layer proven by unauthenticated rejection.

### 5. `src/routes_auth/tests/request_access_test.rs` (line 443)
**Auth tier**: Optional | **Deleted file**: `routes_auth_auth_test.rs`

Replace stub with 1 test function:
- `test_optional_auth_endpoints_accept_unauthenticated` - 3 endpoints: POST `/bodhi/v1/auth/initiate`, POST `/bodhi/v1/auth/callback`, POST `/bodhi/v1/apps/request-access`
- POST without body → assert non-401/non-403 status code. Determine exact codes during implementation and use `assert_eq!` where possible.

### 6. `src/routes_users/tests/user_info_test.rs` (line 353)
**Auth tier**: Optional | **Deleted file**: `routes_users_info_auth_test.rs`

Replace stub with 2 test functions:
- `test_optional_auth_endpoints_accept_unauthenticated` - 3 endpoints: GET `/bodhi/v1/user`, POST `/bodhi/v1/user/request-access`, GET `/bodhi/v1/user/request-status`
  - GET `/bodhi/v1/user` → likely 200 OK
  - POST `/bodhi/v1/user/request-access` without body → BAD_REQUEST (proves not auth-blocked)
  - GET `/bodhi/v1/user/request-status` → determine exact status during implementation
- `test_user_info_allows_authenticated` - GET `/bodhi/v1/user` with session → assert_eq!(OK)

### 7. `src/routes_users/tests/management_test.rs` (line 512)
**Auth tier**: Manager | **Deleted file**: `routes_users_management_auth_test.rs`

Replace stub with 2 test functions:
- `test_user_management_endpoints_reject_unauthenticated` - 3 endpoints via `#[case]`: GET `/bodhi/v1/users`, PUT `/bodhi/v1/users/some_user_id/role`, DELETE `/bodhi/v1/users/some_user_id`
- `test_user_management_endpoints_reject_insufficient_role` - `#[values]` cartesian: 2 roles (`"resource_user"`, `"resource_power_user"`) × 3 endpoints → `assert_eq!(FORBIDDEN)`

No allow test - all 3 handlers call `MockAuthService` (no expectations). Auth proven by rejection tests + shared `manager_session_apis` route layer with access_request endpoints.

### 8. `src/routes_users/tests/access_request_test.rs` (line 699)
**Auth tier**: Manager | **Deleted file**: `routes_users_access_request_auth_test.rs`

Replace stub with 3 test functions:
- `test_access_request_endpoints_reject_unauthenticated` - 4 endpoints via `#[case]`: GET `/bodhi/v1/access-requests/pending`, GET `/bodhi/v1/access-requests`, POST `.../1/approve`, POST `.../1/reject`
- `test_access_request_endpoints_reject_insufficient_role` - `#[values]` cartesian: 2 roles × 4 endpoints → `assert_eq!(FORBIDDEN)`
- `test_access_request_endpoints_allow_manager_and_admin` - `#[values]` cartesian: 2 roles (`"resource_manager"`, `"resource_admin"`) × 2 safe GET endpoints → `assert_eq!(OK)` (db_service returns empty list)

Only `list_pending` and `list_all` safe for allow test. `approve` and `reject` call `MockAuthService`.

## Canonical Test Pattern (from `settings_test.rs`, `aliases_test.rs`)

```rust
// --- Reject unauthenticated ---
#[anyhow_trace]
#[rstest]
#[case::endpoint_name("METHOD", "/path")]
#[tokio::test]
async fn test_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, unauth_request};
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

// --- Reject insufficient role ---
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_endpoints_reject_insufficient_role(
  #[values("role1", "role2")] role: &str,
  #[values(("METHOD", "/path"), ...)] endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status(), "{role} should be forbidden from {method} {path}");
  Ok(())
}

// --- Allow eligible roles ---
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_endpoints_allow_eligible_roles(
  #[values("role1", "role2")] role: &str,
  #[values(("METHOD", "/safe/path"))] endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  assert_eq!(StatusCode::OK, response.status()); // or NOT_FOUND for nonexistent resources
  Ok(())
}
```

Key rules:
1. `assert_eq!` not `assert_ne!` - use actual expected status codes
2. `#[values]` for cartesian products instead of loops
3. `#[anyhow_trace]` on all tests, return `anyhow::Result<()>`
4. Imports inside test function body (not at file top)
5. `use tower::ServiceExt;` needed for `.oneshot()`
6. Comments explaining why only certain endpoints are safe for allow tests

## Execution Strategy

### Workflow per batch:
1. Launch sub-agent with 2-3 modules, full context (file paths, patterns, endpoints)
2. Sub-agent writes tests, runs `cargo test -p routes_app --lib` until pass
3. If a test is correct but won't pass → sub-agent notes it, skips it
4. Sub-agent returns summary: tests added, pass/fail status
5. Main agent updates this plan with progress/deviations
6. Main agent makes a local `git commit` for the batch
7. Repeat for next batch

### Failing test policy:
- If test logic is correct but environment/services prevent passing → leave failing, note in "Failing Tests" section
- Add `#[ignore]` to failing tests so they don't block future runs
- Present full report at end of task

## Implementation Batches

### Batch 1: Protected tier (PowerUser + User)
- `routes_api_models` (PowerUser tier)
- `routes_oai` (User tier)
- `routes_ollama` (User tier)
- **Status**: PENDING

### Batch 2: Session-only + Optional auth
- `routes_toolsets` (User session tier)
- `routes_auth/request_access` (Optional auth)
- **Status**: PENDING

### Batch 3: Users module (3 files)
- `routes_users/user_info` (Optional auth)
- `routes_users/management` (Manager tier)
- `routes_users/access_request` (Manager tier)
- **Status**: PENDING

## Failing Tests (to fix manually)
(none yet)

## Final Report
(generated at end of task)

## Verification

```bash
# After each module, verify compilation:
cargo check -p routes_app

# After all modules, run full test suite:
cargo test -p routes_app --lib

# Verify no stubs remain:
grep -r "stub for plan completion" crates/routes_app/

# Expected: test count > 319 (current), zero stubs, zero failures
```

## Implementation Note for Optional Auth Endpoints

For optional auth endpoints (auth, user_info), POST without body should return a non-auth error (e.g., BAD_REQUEST). During implementation, run each test once to determine the exact status code returned, then use `assert_eq!` with that code. If the response is unpredictable (e.g., depends on session state), fall back to asserting the status is not UNAUTHORIZED and not FORBIDDEN.
