# routes_app Test Patterns - Quick Reference

**Last Updated**: 2026-02-09 (Phase 5 completion)
**Test Count**: 423 passing tests
**Pattern Compliance**: 100% (with documented exemptions)

---

## When to Use Each Pattern

### 1. Standard Auth Tier Test → Use `build_test_router()`

```rust
use crate::test_utils::{build_test_router, unauth_request, session_request, create_authenticated_session};

// User tier (401 only)
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_endpoint_requires_auth() -> anyhow::Result<()> {
  let (router, _, _temp) = build_test_router().await?;
  let response = router.oneshot(unauth_request("GET", "/path")).await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

// PowerUser tier (401 + 403 + allow)
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_endpoint_forbids_resource_user() -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &["resource_user"]).await?;
  let response = router.oneshot(session_request("POST", "/path", &cookie)).await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());
  Ok(())
}

#[anyhow_trace]
#[rstest]
#[case("power_user")]
#[case("resource_manager")]
#[case("resource_admin")]
#[tokio::test]
async fn test_endpoint_allow_power_user_and_above(#[case] role: &str) -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(app_service.session_service().as_ref(), &[role]).await?;
  let response = router.oneshot(session_request("POST", "/path", &cookie)).await?;
  assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
  Ok(())
}
```

---

### 2. Handler-Focused Test → Use Local Router

**When**: Testing handler logic, validation, error cases
**Why**: Simpler setup, direct handler invocation, clearer failure messages

```rust
#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_handler_validation_error() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default()
    .with_db_service()
    .await
    .build()?;
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));
  let router = Router::new()
    .route("/path", post(handler_function))
    .with_state(state);

  let resp = router
    .oneshot(Request::post("/path").json(invalid_payload)?)
    .await?;

  assert_eq!(StatusCode::BAD_REQUEST, resp.status());
  Ok(())
}
```

---

### 3. Multi-Step Flow Test → Use TestServer (EXEMPT)

**When**: OAuth flows, session continuity, cookie-based workflows
**Why**: TestServer maintains state across requests

```rust
use axum_test::TestServer;

#[anyhow_trace]
#[rstest]
#[tokio::test]
async fn test_oauth_flow() -> anyhow::Result<()> {
  let router = Router::new()
    .route("/auth/initiate", post(auth_initiate_handler))
    .route("/auth/callback", post(auth_callback_handler))
    .layer(app_service.session_service().session_layer())
    .with_state(state);

  let mut client = TestServer::new(router)?;
  client.save_cookies(); // Enable cookie persistence

  // Step 1: Initiate auth
  let login_resp = client.post("/auth/initiate").await;
  login_resp.assert_status(StatusCode::CREATED);

  // Step 2: Callback (session cookie automatically included)
  let callback_resp = client
    .post("/auth/callback")
    .json(&json!({"code": "...", "state": "..."}))
    .await;
  callback_resp.assert_status(StatusCode::OK);

  Ok(())
}
```

---

### 4. MockService Dependency → Document Violation

**When**: Endpoint requires MockToolService, MockAuthService, or MockSharedContext
**Why**: Mocks panic without expectations, setting up expectations adds complexity

**Action**: Document in violations registry, use handler-focused tests

```rust
// routes_toolsets example - cannot migrate to build_test_router()
// Violation: MockToolService dependency
// Documented in: ai-docs/claude-plans/phase5-complete-summary.md
```

---

## Request Helper Quick Reference

### Unauthenticated Requests
```rust
use crate::test_utils::{unauth_request, unauth_request_with_body};

// GET/DELETE without body
let req = unauth_request("GET", "/bodhi/v1/info");

// POST/PUT with JSON body
let body = serde_json::json!({"key": "value"});
let req = unauth_request_with_body("POST", "/bodhi/v1/setup", &body)?;
```

### Authenticated Requests
```rust
use crate::test_utils::{
  create_authenticated_session,
  session_request,
  session_request_with_body
};

// Create session with roles
let cookie = create_authenticated_session(
  session_service.as_ref(),
  &["resource_user", "power_user"]
).await?;

// GET/DELETE with session
let req = session_request("GET", "/bodhi/v1/models", &cookie);

// POST/PUT with session and JSON body
let body = serde_json::json!({"key": "value"});
let req = session_request_with_body("POST", "/bodhi/v1/tokens", &cookie, &body)?;
```

---

## Auth Tier Decision Matrix

| Endpoint Auth | 401 Test | 403 Test | Allow Test | Pattern |
|---------------|----------|----------|------------|---------|
| Public | ❌ | ❌ | ✅ (verify public access) | `unauth_request()` |
| Optional Auth | ❌ | ❌ | ✅ (both modes) | Parameterized test |
| User | ✅ | ❌ | ✅ (all roles) | 401 + allow |
| PowerUser | ✅ | ✅ (user) | ✅ (power_user+) | 401 + 403 + allow |
| Manager | ✅ | ✅ (user + power) | ✅ (manager+) | 401 + 403 + allow |
| Admin | ✅ | ✅ (all but admin) | ✅ (admin only) | 401 + 403 + allow |

---

## Common Pitfalls

### ❌ Don't: Create session without roles
```rust
// This creates a session without any roles - will fail auth checks
let cookie = create_authenticated_session(session_service.as_ref(), &[]).await?;
```

### ✅ Do: Always specify at least one role
```rust
let cookie = create_authenticated_session(
  session_service.as_ref(),
  &["resource_user"]
).await?;
```

---

### ❌ Don't: Use `Utc::now()` directly in handlers
```rust
pub async fn handler(State(state): State<Arc<dyn RouterState>>) -> Result<Json<Response>> {
  let now = Utc::now(); // NOT TESTABLE
  // ...
}
```

### ✅ Do: Use `TimeService` from state
```rust
pub async fn handler(State(state): State<Arc<dyn RouterState>>) -> Result<Json<Response>> {
  let now = state.app_service().time_service().utc_now(); // Testable
  // ...
}
```

---

### ❌ Don't: Hardcode expected status for "allow" tests
```rust
assert_eq!(StatusCode::OK, response.status()); // Breaks if resource doesn't exist
```

### ✅ Do: Accept OK or NOT_FOUND
```rust
assert!(
  response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND
); // Auth passed, resource may not exist
```

---

## Module-Specific Notes

### routes_toolsets
- **Violation**: MockToolService dependencies
- **Pattern**: Handler-focused tests for allow cases
- **Location**: `crates/routes_app/src/routes_toolsets/tests/toolsets_test.rs`

### routes_users
- **Violation**: MockAuthService dependencies
- **Pattern**: TestServer for management endpoints
- **Location**: `crates/routes_app/src/routes_users/tests/management_test.rs`

### routes_oai + routes_ollama
- **Violation**: MockSharedContext for streaming
- **Pattern**: Streaming tests use MockSharedContext, auth tests use build_test_router()
- **Location**: `crates/routes_app/src/routes_oai/tests/chat_test.rs`, `routes_ollama/tests/`

### routes_auth
- **Exemption**: OAuth flow tests require TestServer
- **Pattern**: Multi-step flow tests use TestServer with save_cookies()
- **Location**: `crates/routes_app/src/routes_auth/tests/login_test.rs`

---

## File Locations

### Test Infrastructure
- **Main helpers**: `crates/routes_app/src/test_utils/mod.rs`
- **Router factory**: `build_test_router()`
- **Request builders**: `unauth_request()`, `session_request()`, etc.
- **Session helper**: `create_authenticated_session()`

### Complete Documentation
- **Full summary**: `ai-docs/claude-plans/phase5-complete-summary.md`
- **Violations registry**: Section in phase5-complete-summary.md
- **Pattern catalog**: Section in phase5-complete-summary.md

---

## Quick Checklist for New Endpoints

When adding a new endpoint:

1. ☑️ Determine auth tier (public, optional, user, power_user, manager, admin)
2. ☑️ Add 401 test if not public (use `build_test_router()`)
3. ☑️ Add 403 test if power_user+ tier (test lower roles)
4. ☑️ Add allow test (use parameterized test for multiple roles)
5. ☑️ Add handler-focused tests for validation/error cases
6. ☑️ Update OpenAPI annotations (`#[utoipa::path(...)]`)
7. ☑️ Run `cargo test -p routes_app --lib` to verify

---

## Quick Reference Commands

```bash
# Run all routes_app tests
cargo test -p routes_app --lib --no-fail-fast

# Run specific module tests
cargo test -p routes_app --lib routes_auth::tests
cargo test -p routes_app --lib routes_toolsets::tests

# Run single test
cargo test -p routes_app --lib test_endpoint_requires_auth

# Check compilation
cargo check -p routes_app

# Format code
cargo fmt -p routes_app

# Generate OpenAPI spec
cargo run --package xtask openapi
```

---

**End of Quick Reference** - For detailed explanations, see `ai-docs/claude-plans/phase5-complete-summary.md`
