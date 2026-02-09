---
name: test-routes-app
description: >
  Use when writing or migrating unit tests for the routes_app crate.
  Covers canonical test patterns, fixture setup, request/response helpers,
  error assertions, SSE streaming, background tasks, session tests, and
  mock service injection. Examples: "write tests for a new handler",
  "migrate old tests to canonical pattern", "add coverage for error paths".
---

# routes_app Test Skill

Write and migrate tests for the `routes_app` crate using two complementary patterns:
**router-level** (auth + integration) and **handler-level** (isolated business logic).

## Two Test Patterns

### 1. Router-Level Tests (Auth Tier + Integration)

Use for: auth tier verification, end-to-end request flow through real middleware.

Located in: `crates/routes_app/tests/*_auth_test.rs`

```rust
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;

#[rstest]
#[case::list_models("GET", "/bodhi/v1/models")]
#[case::get_model("GET", "/bodhi/v1/models/some-id")]
#[tokio::test]
async fn test_endpoints_reject_unauthenticated(#[case] method: &str, #[case] path: &str) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request(method, path))
    .await
    .unwrap();
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}
```

### 2. Handler-Level Tests (Business Logic)

Use for: isolated handler testing with specific mock expectations, edge cases.

Located in: `crates/routes_app/src/<module>/tests/*_test.rs`

```rust
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_<handler>_<scenario>() -> anyhow::Result<()> {
  let app_service = AppServiceStubBuilder::default().build()?;
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));
  let router = Router::new()
    .route("/path", post(handler_under_test))
    .with_state(state);
  let response = router.oneshot(Request::post("/path").json(payload)?).await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
```

## When to Use Which

| Scenario | Pattern |
|----------|---------|
| Auth tier verification (401/403) | Router-level |
| Endpoint reachability with correct role | Router-level |
| Business logic with mock expectations | Handler-level |
| Edge cases needing MockAuthService/MockToolService/MockSharedContext | Handler-level |
| Error path testing with specific service failures | Handler-level |
| SSE streaming response validation | Handler-level |

## Core Rules

1. **Annotations (handler-level)**: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]`. Add `#[awt]` ONLY with `#[future]` params.
2. **Annotations (router-level)**: `#[rstest]` + `#[tokio::test]` (no `#[anyhow_trace]`, use `.unwrap()`)
3. **Naming**: `test_<handler_name>_<scenario>` (handler) or `test_<tier>_endpoints_<behavior>` (router)
4. **Assertions**: `assert_eq!(expected, actual)` with `use pretty_assertions::assert_eq;`
5. **Error codes**: Assert `body["error"]["code"]`, never message text
6. **Router-level "allowed" tests**: Only test endpoints using real services (db_service, data_service). Skip endpoints calling MockAuthService/MockToolService/MockSharedContext.

## Pattern Files

- **[fixtures.md](fixtures.md)** -- AppServiceStubBuilder, build_test_router, DB fixtures, mock injection
- **[requests.md](requests.md)** -- Request construction, auth headers, session helpers
- **[assertions.md](assertions.md)** -- Response parsing, error codes, SSE streams, DB verification
- **[advanced.md](advanced.md)** -- Background tasks, session/cookie tests, parameterized tests, mock servers, auth test organization

## Standard Imports

### Router-level tests
```rust
use axum::http::StatusCode;
use pretty_assertions::assert_eq;
use routes_app::test_utils::{
  build_test_router, create_authenticated_session, session_request, unauth_request,
};
use rstest::rstest;
use tower::ServiceExt;
```

### Handler-level tests
```rust
use axum::{body::Body, http::Request, routing::{get, post, put, delete}, Router};
use tower::ServiceExt;
use reqwest::StatusCode;
use rstest::rstest;
use anyhow_trace::anyhow_trace;
use pretty_assertions::assert_eq;
use serde_json::{json, Value};
use std::sync::Arc;
use server_core::{
  test_utils::{RequestTestExt, ResponseTestExt, RequestAuthExt},
  DefaultRouterState, MockSharedContext,
};
use services::test_utils::AppServiceStubBuilder;
```

## Auth Tier Reference

| Tier | Role | Endpoints |
|------|------|-----------|
| Public | None | /ping, /health, /app/info, /app/setup, /logout |
| Optional Auth | Any | /bodhi/v1/user, /bodhi/v1/auth/*, /bodhi/v1/user/request-* |
| User | resource_user | /v1/models, /v1/chat/completions, /api/tags, /bodhi/v1/models (read) |
| User Session | resource_user | /bodhi/v1/toolsets (CRUD) |
| User OAuth | resource_user | /bodhi/v1/toolsets (list) |
| PowerUser | resource_power_user | /bodhi/v1/models (write), /bodhi/v1/modelfiles/*, /bodhi/v1/api-models |
| PowerUser Session | resource_power_user | /bodhi/v1/tokens |
| Admin Session | resource_admin | /bodhi/v1/settings, /bodhi/v1/toolset_types |
| Manager Session | resource_manager | /bodhi/v1/access-requests/*, /bodhi/v1/users |
