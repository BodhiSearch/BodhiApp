# Plan: routes_app Test Uniformity, Coverage & Skill Update

## Context

The `crates/routes_app/` test suite has grown organically across 10 modules with 16 test files. Tests were recently migrated from standalone auth test files to inline modules, but inconsistencies remain in auth testing patterns, router construction approaches, helper placement, and coverage depth. This plan standardizes all tests to a uniform pattern, fills coverage gaps, and updates the test-routes-app skill to match.

## Key Decisions (from interview)

1. **Full router preferred**: All tests should use `build_routes()` with real services + session middleware, unless technically impossible (e.g., streaming tests needing `MockSharedContext`)
2. **Auth in happy path tests**: Add allowed roles as `#[values]` to existing happy path tests, combining auth verification + handler logic testing
3. **401 tests**: Use `#[case]` (named variants, e.g., `#[case::list_users("GET", "/path")]`)
4. **403 tests**: Use `#[values]` cartesian product: `#[values("role1", "role2")] role` x `#[values(("GET", "/path"), ...)] endpoint`
5. **Allow tests**: For GET endpoints assert 200. For POST/PUT/DELETE with disallowed roles, assert 403. Happy path tests with `#[values]` for allowed roles serve as allow tests
6. **Mock injection**: Build `AppServiceStubBuilder` with selected mock overrides, pass to `build_routes()`. If not feasible, keep existing test and note as violation
7. **Session flow exemption**: Multi-step OAuth/session tests (TestServer-based) stay as-is. Note as violation. Suggest migration to e2e tests
8. **OAI/Ollama streaming**: Leave `routes_oai` chat/embeddings and `routes_ollama` handler tests as-is (need `MockSharedContext.expect_forward_request()`)
9. **Test helpers**: Move reusable helpers to shared `crates/routes_app/src/test_utils/`
10. **Module ownership**: Each module owns its auth tests. Move cross-module tests back (e.g., toolset_types from settings_test.rs)
11. **Coverage**: Quantitative baseline via coverage tool before starting
12. **Skill update**: Descriptive with defaults (not prescriptive)

## Current State Analysis

### Auth Test Pattern Inventory

| Module | 401 Test | 403 Test | Allow Test | Pattern Used | Issues |
|--------|----------|----------|------------|--------------|--------|
| routes_users/management | `#[case]` | `#[values]` cartesian | N/A (200 needs mock) | Canonical | No allow test |
| routes_users/access_request | `#[case]` | `#[values]` cartesian | `#[values]` (GET only) | Canonical | Allow only for GET endpoints |
| routes_users/user_info | `#[case]` | N/A (optional auth) | `#[values]` all roles | Good | Optional auth tier |
| routes_auth/login | `#[case]` (optional) | N/A (optional auth) | N/A | Good | Session flows exempted |
| routes_auth/request_access | `#[case]` (optional) | N/A (optional auth) | N/A | Good | Optional auth tier |
| routes_oai/models | `#[case]` | N/A (User tier) | `#[values]` (GET only) | Good | Allow only for GET /v1/models |
| routes_oai/chat | None | None | None | Missing | No auth tests at all |
| routes_ollama/handlers | `#[case]` | N/A (User tier) | `#[values]` (GET only) | Good | Allow only for GET /api/tags |
| routes_settings | `#[case]` | `#[values]` cartesian | `#[case]` (admin only) | Mixed | Has toolset_types tests (cross-module) |
| routes_setup | Individual tests | N/A (public) | N/A | Ad-hoc | Public endpoints, no auth needed |
| routes_api_token | `#[case]` | **Manual for-loop** | `#[case]` individual | **Non-canonical** | Manual loop, only tests resource_user |
| routes_toolsets | `#[case]` | **MISSING** | **MISSING** | **Incomplete** | No 403/allow for CRUD endpoints |
| routes_api_models | `#[case]` | `#[values]` (resource_user only) | `#[values]` (GET only) | Good | 403 only tests resource_user; allow only for GET endpoints |
| routes_models | `#[case]` (read/write split) | `#[values]` (resource_user only, write) | `#[values]` (read: all roles, write: power_user+) | Good | Separate read/write auth tiers; 403 only for write with resource_user |

### Router Construction Patterns

| Pattern | Where Used | Migration Target |
|---------|-----------|-----------------|
| `build_test_router()` full middleware | Auth tests only | All tests (preferred) |
| Local `Router::new().route().with_state()` | Most handler tests | Migrate to full router |
| `test_router()` local helper | toolsets, user_info, api_models, oai/models | Remove, use full router |
| `app()` local helper | settings, api_token | Remove, use full router |
| `TestServer::new(router)` | login_test.rs (OAuth flows) | Exempt (session flows) |

## Phased Implementation Plan

### Phase 0: Coverage Baseline & Test Infrastructure

**Goal**: Establish baseline metrics and enhance shared test utilities.

**Coverage baseline**:
- Run `cargo tarpaulin -p routes_app --out json` to get per-file coverage
- Document baseline numbers for each module

**Test utils enhancement** (`crates/routes_app/src/test_utils/`):
- **New file: `crates/routes_app/src/test_utils/router.rs`** (already exists, enhance):
  - Add `build_test_router_with_services()` variant that accepts optional service overrides via `AppServiceStubBuilder`
  - Add `session_request_with_body(method, path, cookie, body)` for POST/PUT tests with auth
  - Add `unauth_request_with_body(method, path, body)` for POST/PUT without auth
- **New file: `crates/routes_app/src/test_utils/assertions.rs`**:
  - `assert_auth_rejected(response)` - asserts 401 status
  - `assert_forbidden(response, role, method, path)` - asserts 403 with descriptive message
  - `assert_auth_passed(response)` - asserts status is NOT 401/403

**Files to modify**:
- `crates/routes_app/src/test_utils/mod.rs`
- `crates/routes_app/src/test_utils/router.rs`

---

### Phase 1: routes_settings + routes_models (Simple CRUD, establish patterns)

**Goal**: Establish the canonical test pattern on simple modules first.

#### routes_settings (`crates/routes_app/src/routes_settings/tests/settings_test.rs`)
- **Move toolset_types auth tests OUT** to `routes_toolsets/tests/toolsets_test.rs` (lines 418-421, 438-444 are toolset_type endpoints)
- **Migrate handler tests to full router**: `test_routes_settings_list`, `test_routes_setting_update_*`, `test_delete_setting_*` currently use local `app()` helper with `Arc<dyn AppService>`. Migrate to use full router with `resource_admin` role
- **Add allowed roles to happy path**: Convert `test_routes_settings_list` to use `#[values("resource_admin")]` (only admin allowed)
- **Keep existing**: `test_admin_endpoints_reject_unauthenticated` (good `#[case]` pattern), `test_admin_endpoints_reject_insufficient_role` (good `#[values]` pattern)

#### routes_models (`crates/routes_app/src/routes_models/tests/`)
- **aliases_test.rs**: Already has good auth tests (read 401 `#[case]`, write 401 `#[case]`, write 403 `#[values]` for resource_user, read allow `#[values]` all roles, write allow `#[values]` power_user+). Uses local `test_router()` and `app()` fixture. Improvements needed:
  - Expand 403 test to cover all insufficient roles (not just resource_user)
  - Migrate handler tests to full router where practical
- **metadata_test.rs**: Check auth tier and add auth tests if missing
- **pull_test.rs**: Check auth tier and add auth tests if missing
- **Migrate to full router where practical**: Replace local `test_router()`/`app()` with full router pattern

**Files to modify**:
- `crates/routes_app/src/routes_settings/tests/settings_test.rs`
- `crates/routes_app/src/routes_models/tests/aliases_test.rs`
- `crates/routes_app/src/routes_models/tests/metadata_test.rs`
- `crates/routes_app/src/routes_models/tests/pull_test.rs`

---

### Phase 2: routes_toolsets + routes_api_token (Auth pattern fixes)

**Goal**: Fix the most inconsistent modules.

#### routes_toolsets (`crates/routes_app/src/routes_toolsets/tests/toolsets_test.rs`)
- **Add 403 test**: `test_toolset_endpoints_reject_insufficient_role` using `#[values]` cartesian (roles that can't access x endpoints)
- **Add allow test**: Use `#[values]` for allowed roles in existing happy path tests or add dedicated allow tests
- **Receive toolset_types auth tests** moved from settings_test.rs
- **Migrate handler tests**: Current tests use local `test_router()` with `MockToolService`. Try full router; if `MockToolService` injection is needed, use `AppServiceStubBuilder::with_tool_service()`. Note as violation if not feasible

#### routes_api_token (`crates/routes_app/src/routes_api_token/tests/api_token_test.rs`)
- **Replace manual for-loop** in `test_token_endpoints_reject_insufficient_role` with `#[values]` cartesian product
- **Expand role coverage**: Currently only tests `resource_user` for 403. Token endpoints require PowerUser+. Add all insufficient roles
- **Replace `#[case]` allow test** with `#[values]` pattern for `test_token_list_endpoint_allows_eligible_roles`
- **Migrate handler tests to full router**: Current tests use local `app()` helper with `test_db_service` fixture. Try full router with `db_service` from `build_test_router()`

**Files to modify**:
- `crates/routes_app/src/routes_toolsets/tests/toolsets_test.rs`
- `crates/routes_app/src/routes_api_token/tests/api_token_test.rs`

---

### Phase 3: routes_users (Complex) + routes_api_models (Medium)

**Goal**: Handle the complex user management module and API models.

#### routes_users (`crates/routes_app/src/routes_users/tests/`)
- **management_test.rs**:
  - Add allow test for manager/admin roles (currently only has 401 + 403, no allow)
  - Migrate `test_list_users_handler_*`, `test_change_user_role_*`, `test_remove_user_*` to full router with appropriate role `#[values]`
  - These tests use `MockAuthService` expectations - need to verify if `build_test_router()` + mock auth service injection works
- **access_request_test.rs**:
  - Expand allow test to cover POST endpoints (approve/reject), not just GET endpoints
  - Migrate handler tests to full router where feasible
  - `test_approve_request_clears_user_sessions` is complex (real SQLite + real session store + mock auth) - may need to stay as-is
- **user_info_test.rs**:
  - Good coverage already. Migrate `test_router()` local helper to full router
  - `user_info_handler` is special (optional auth, works for both authenticated and unauthenticated)

#### routes_api_models (`crates/routes_app/src/routes_api_models/tests/api_models_test.rs`)
- **Already has auth tests**: 401 `#[case]`, 403 `#[values]` (resource_user only), allow `#[values]` (GET only). Uses local `test_router()`. Improvements needed:
  - Expand 403 test to cover all insufficient roles (not just resource_user)
  - Expand allow test to cover POST/PUT/DELETE endpoints (not just GET)
  - Migrate handler tests from local `test_router()` to full router

**Files to modify**:
- `crates/routes_app/src/routes_users/tests/management_test.rs`
- `crates/routes_app/src/routes_users/tests/access_request_test.rs`
- `crates/routes_app/src/routes_users/tests/user_info_test.rs`
- `crates/routes_app/src/routes_api_models/tests/api_models_test.rs`

---

### Phase 4: routes_oai + routes_ollama (Auth additions, leave streaming)

**Goal**: Fill auth gaps while leaving complex streaming tests untouched.

#### routes_oai
- **models_test.rs**: Expand allow test to cover all OAI endpoints (GET /v1/models, GET /v1/models/{id}), not just GET /v1/models
- **chat_test.rs**: Leave streaming/handler tests as-is (need `MockSharedContext`). Add 401 tests for chat/embeddings endpoints. Note as violation
- **Add 401 test**: All OAI endpoints should reject unauthenticated (already done in models_test.rs but need to verify chat_test.rs)

#### routes_ollama
- **handlers_test.rs**: Expand allow test to cover all Ollama endpoints (GET /api/tags, POST /api/show, POST /api/chat), not just GET /api/tags
- Leave handler tests as-is (inner `mod test` with `router_state_stub`)
- Note handler tests as violations (not using full router)

**Files to modify**:
- `crates/routes_app/src/routes_oai/tests/models_test.rs`
- `crates/routes_app/src/routes_oai/tests/chat_test.rs`
- `crates/routes_app/src/routes_ollama/tests/handlers_test.rs`

---

### Phase 5: routes_auth + routes_setup (Complex, exemptions)

**Goal**: Improve coverage on complex modules while respecting exemptions.

#### routes_auth
- **login_test.rs**:
  - Multi-step OAuth flow tests (TestServer-based) are **EXEMPT** from full-router migration
  - Verify auth tier tests exist for all endpoints (auth/initiate, auth/callback are optional/public)
  - Note session flow tests as violations in final report
- **request_access_test.rs**:
  - Verify auth tier coverage. request-access is optional auth
  - Improve coverage for edge cases if any are missing

#### routes_setup
- **setup_test.rs**:
  - Public endpoints (info, setup, logout) - verify public access tests exist
  - Current tests are reasonable for public tier
  - Migrate local router construction to full router where practical

**Files to modify**:
- `crates/routes_app/src/routes_auth/tests/login_test.rs`
- `crates/routes_app/src/routes_auth/tests/request_access_test.rs`
- `crates/routes_app/src/routes_setup/tests/setup_test.rs`

---

### Follow-up Tasks (Separate from main plan)

#### Task A: Coverage Re-run
- Run coverage again after all phases
- Compare before/after per module
- Identify remaining low-coverage areas

#### Task B: Violations & Exceptions Report
- Document all tests that couldn't be migrated to full-router pattern
- Categorize: session flows, streaming tests, complex mock dependencies
- Include concise reason for each violation
- Suggest future migration path (e.g., e2e tests for session flows)

#### Task C: Skill Update
- Update `.claude/skills/test-routes-app/` to reflect new canonical patterns
- Tone: Descriptive with defaults
- Key updates:
  - Prefer full router (`build_routes()` + `build_test_router()`) over isolated mock routers
  - `#[case]` for 401 tests (named variants), `#[values]` for 403 tests (cartesian product)
  - Happy path tests should include `#[values]` for allowed roles
  - Session flow tests are exempted - suggest e2e tests for multi-step user journeys
  - All reusable helpers go in shared `test_utils/`
  - Each module owns its auth tests (no cross-module auth testing)
  - routes_oai/routes_ollama streaming tests exempted from full-router rule

## Canonical Test Pattern Reference

### 401 Test (Unauthenticated rejection)
```rust
#[anyhow_trace]
#[rstest]
#[case::list_items("GET", "/bodhi/v1/items")]
#[case::create_item("POST", "/bodhi/v1/items")]
#[tokio::test]
async fn test_endpoints_reject_unauthenticated(
  #[case] method: &str,
  #[case] path: &str,
) -> anyhow::Result<()> {
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request(method, path)).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}
```

### 403 Test (Insufficient role rejection)
```rust
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_endpoints_reject_insufficient_role(
  #[values("resource_user", "resource_power_user")] role: &str,
  #[values(
    ("GET", "/bodhi/v1/items"),
    ("POST", "/bodhi/v1/items"),
    ("DELETE", "/bodhi/v1/items/some_id")
  )]
  endpoint: (&str, &str),
) -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(), &[role]
  ).await?;
  let (method, path) = endpoint;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status(),
    "{role} should be forbidden from {method} {path}");
  Ok(())
}
```

### Happy Path with Auth (Combined allow + handler test)
```rust
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_list_items_success(
  #[values("resource_manager", "resource_admin")] role: &str,
) -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(), &[role]
  ).await?;
  let response = router.oneshot(session_request("GET", "/bodhi/v1/items", &cookie)).await?;
  assert_eq!(StatusCode::OK, response.status());
  // ... assert response body ...
  Ok(())
}
```

## Verification Strategy

After each phase:
1. Run `cargo check -p routes_app` to verify compilation
2. Run `cargo test -p routes_app` to verify all tests pass
3. Check for any newly failing tests
4. Note violations (tests that couldn't be migrated)

After all phases:
1. Run full `make test.backend`
2. Run coverage comparison
3. Generate violations report
