# routes_app Auth Test Patterns - Quick Reference

## Critical Pattern Rules (All Phases)

### Rule 1: MockService Endpoints → No Allow Tests
**When**: Endpoint uses MockToolService, MockAuthService, or MockSharedContext without real services
**Action**: Document as violation, skip allow tests
**Reason**: Mocks panic without expectations; adding allow tests provides no value

### Rule 2: Auth Tier Determines Test Structure
| Auth Tier | 401 Test | 403 Test | Allow Test |
|-----------|----------|----------|------------|
| Public | ❌ No | ❌ No | ❌ No |
| Optional Auth | ✅ Yes | ❌ No | ✅ Both auth + unauth |
| User | ✅ Yes | ❌ No | ✅ All 4 roles |
| PowerUser | ✅ Yes | ✅ Yes (resource_user) | ✅ 3 roles (power+) |
| Manager | ✅ Yes | ✅ Yes (user, power_user) | ✅ 2 roles (manager+) |
| Admin | ✅ Yes | ✅ Yes (user, power_user, manager) | ✅ 1 role (admin) |

### Rule 3: Allow Test Response Assertions
**Pattern**: Real services may return 404 (resource doesn't exist)
**Assertion**: Use `assert!(status == OK || status == NOT_FOUND)` for GET/POST endpoints
**Reason**: 404 proves auth passed (endpoint was reached, resource wasn't found)

**Example**:
```rust
let response = router.oneshot(session_request(method, path, &cookie)).await?;
assert!(
  response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
  "{role} should be allowed to {method} {path}, got {}", response.status()
);
```

### Rule 4: Handler Test Migration is Optional
**When**: Auth tier tests use `build_test_router()`
**Action**: Handler tests can keep focused helpers (e.g., `app()`)
**Reason**: Auth coverage is what matters; handler tests focus on business logic

### Rule 5: Streaming Tests are Exempt
**When**: Tests use `MockSharedContext.expect_forward_request()` for SSE
**Action**: Document as violation, keep as-is
**Reason**: Complex mock setup required; migrating is out of scope

## Canonical Test Templates

### 401 Test (Unauthenticated Rejection)
```rust
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
```

**For POST endpoints needing body**:
```rust
use crate::test_utils::{build_test_router, unauth_request_with_body};
use axum::body::Body;
let response = router.oneshot(unauth_request_with_body(method, path, Body::empty())).await?;
```

### 403 Test (Insufficient Role)
```rust
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_endpoints_reject_insufficient_role(
  #[values("resource_user")] role: &str,
  #[values(("METHOD", "/path"))] endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());
  Ok(())
}
```

### Allow Test (All Roles Permitted)
```rust
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_endpoints_allow_all_roles(
  #[values("resource_user", "resource_power_user", "resource_manager", "resource_admin")] role: &str,
  #[values(("METHOD", "/path"))] endpoint: (&str, &str),
) -> anyhow::Result<()> {
  use crate::test_utils::{build_test_router, create_authenticated_session, session_request};
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let (method, path) = endpoint;
  let response = router.oneshot(session_request(method, path, &cookie)).await?;
  // May return 200 or 404 depending on resource existence
  assert!(
    response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND,
    "{role} should be allowed to {method} {path}, got {}", response.status()
  );
  Ok(())
}
```

**For POST endpoints requiring body**:
```rust
use crate::test_utils::{build_test_router, create_authenticated_session, session_request_with_body};
use axum::body::Body;
use serde_json::json;

let body = Body::from(serde_json::to_string(&json!({
  "field": "value"
}))?);

let response = router.oneshot(
  session_request_with_body(method, path, &cookie, body)
).await?;
```

## Common Mistakes to Avoid

### ❌ Don't: Use strict 200 OK assertions for all endpoints
```rust
assert_eq!(StatusCode::OK, response.status()); // Fails if endpoint returns 404
```

### ✅ Do: Use flexible assertions for real services
```rust
assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
```

### ❌ Don't: Try to add allow tests for MockService endpoints
```rust
// This will panic without mock expectations
let response = router.oneshot(session_request("GET", "/toolsets", &cookie)).await?;
```

### ✅ Do: Document the violation instead
```rust
// ============================================================================
// VIOLATION DOCUMENTATION:
// Endpoints use MockToolService. Allow tests omitted (would require expectations).
// ============================================================================
```

### ❌ Don't: Add POST 401 tests without request body
```rust
let response = router.oneshot(unauth_request("POST", "/api/show")).await?; // Wrong
```

### ✅ Do: Use `unauth_request_with_body()` for POST
```rust
let response = router.oneshot(unauth_request_with_body("POST", "/api/show", Body::empty())).await?;
```

## Test Utilities Reference

### From `crates/routes_app/src/test_utils.rs`
- `build_test_router()` - Creates router with real services (DataService, DbService, etc.)
- `unauth_request(method, path)` - Unauthenticated GET/POST request
- `unauth_request_with_body(method, path, body)` - Unauthenticated request with body
- `session_request(method, path, cookie)` - Authenticated request
- `session_request_with_body(method, path, cookie, body)` - Authenticated request with body
- `create_authenticated_session(session_service, roles)` - Create session with roles

### Session Cookie Format
```rust
let cookie = create_authenticated_session(
  app_service.session_service().as_ref(),
  &["resource_user", "resource_power_user"] // Multiple roles possible
).await?;
```

## Violation Documentation Template

```rust
// ============================================================================
// VIOLATION DOCUMENTATION:
// [Module/endpoints affected]
// Pattern: [MockService name or streaming pattern]
// Reason: [Why allow tests cannot be added]
// Coverage: [What auth tests exist]
// Status: Acceptable violation
// ============================================================================
```

## Phase Completion Checklist

For each module:
- [ ] Identify auth tier for all endpoints
- [ ] Add 401 test (if not Public tier)
- [ ] Add 403 test (if PowerUser tier or higher)
- [ ] Add allow test (if not Public, not MockService)
- [ ] Document violations (if MockService or streaming)
- [ ] Verify no regressions (`cargo test -p routes_app --lib`)
- [ ] Update phase summary document

## Auth Tier Quick Lookup

| Module | Auth Tier | Notes |
|--------|-----------|-------|
| routes_settings | Public | No auth tests needed |
| routes_models | User | All 4 roles |
| routes_toolsets | User (filtered by OAuth) | MockToolService violation |
| routes_api_token | PowerUser+ | Admin endpoints are Admin tier |
| routes_users/info | User | All 4 roles |
| routes_users/management | Manager+ | MockAuthService violation |
| routes_users/access_request | Mixed | User (request), Manager+ (approve/reject) |
| routes_api_models | PowerUser+ | All 3 roles (power_user+) |
| routes_oai/models | User | All 4 roles |
| routes_oai/chat | User | MockSharedContext violation (streaming) |
| routes_ollama | User | Partial MockSharedContext violation |

## References
- Full phase summaries: `ai-docs/claude-plans/phase[0-4]-*.md`
- Violations registry: `ai-docs/claude-plans/routes-app-test-violations-registry.md`
- Integration tests plan: `ai-docs/claude-plans/20260209-prompt-routes-app-test-uniform.md`
