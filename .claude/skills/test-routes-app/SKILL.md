---
name: test-routes-app
description: >
  Use when writing or migrating unit tests for the routes_app crate.
  Covers canonical test patterns, fixture setup, request/response helpers,
  error assertions, SSE streaming, background tasks, session tests, and
  mock service injection. Examples: "write tests for a new handler",
  "migrate old tests to canonical pattern", "add coverage for error paths".
---

# routes_app Unit Test Skill

Write and migrate unit tests for the `routes_app` crate following uniform conventions.

## Quick Reference

Every route test follows this shape:

```rust
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_<handler>_<scenario>() -> anyhow::Result<()> {
  // 1. Build service stub
  let app_service = AppServiceStubBuilder::default().build()?;
  // 2. Create single-handler router
  let state = Arc::new(DefaultRouterState::new(
    Arc::new(MockSharedContext::default()),
    Arc::new(app_service),
  ));
  let router = Router::new()
    .route("/path", post(handler_under_test))
    .with_state(state);
  // 3. Send request
  let response = router.oneshot(Request::post("/path").json(payload)?).await?;
  // 4. Assert
  assert_eq!(StatusCode::OK, response.status());
  let body: ExpectedType = response.json().await?;
  assert_eq!(expected, body);
  Ok(())
}
```

## Core Rules

1. **Annotations**: `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]` on every async test. Add `#[awt]` ONLY when `#[future]` fixture params are used.
2. **Naming**: `test_<handler_name>_<scenario>` (e.g. `test_create_token_handler_missing_role`)
3. **Module**: `mod tests` (not `mod test`)
4. **Return**: Always `-> anyhow::Result<()>` with `Ok(())` at end
5. **Errors**: Use `?` not `.unwrap()`. Use `.expect("msg")` only in non-`?` contexts (closures, Option chains)
6. **Assertions**: `assert_eq!(expected, actual)` with `use pretty_assertions::assert_eq;`
7. **Error codes**: Assert `body["error"]["code"]`, never message text. Codes are `enum_name-variant_name` in snake_case.
8. **Router scope**: Minimal single-handler router per test (pure unit test)
9. **Auth bypass**: Tests bypass auth middleware; use `RequestAuthExt` to set headers directly

## Pattern Files

For detailed patterns with full code examples, see:

- **[fixtures.md](fixtures.md)** -- AppServiceStubBuilder, DB fixtures, mock injection
- **[requests.md](requests.md)** -- Request construction, auth headers, pagination
- **[assertions.md](assertions.md)** -- Response parsing, error codes, SSE streams, DB verification
- **[advanced.md](advanced.md)** -- Background tasks, session/cookie tests, parameterized tests, mock servers

## Standard Imports

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

## Migration Checklist

When migrating existing tests to the canonical pattern:

- [ ] Add `use pretty_assertions::assert_eq;`
- [ ] Add `use anyhow_trace::anyhow_trace;`
- [ ] Ensure `#[rstest]` + `#[tokio::test]` + `#[anyhow_trace]` annotation order
- [ ] Add `#[awt]` only if `#[future]` fixture params exist
- [ ] Replace `.unwrap()` with `?` (or `.expect()` in closures)
- [ ] Replace manual `Content-Type` + `Body::from()` with `Request::post(uri).json(body)?`
- [ ] Replace manual auth headers with `.with_user_auth()` / `.with_api_token()`
- [ ] Convert error message assertions to error code assertions
- [ ] Remove unused imports (`json!`, auth constant imports, `Body` if no longer direct)
- [ ] Verify `assert_eq!(expected, actual)` order
- [ ] Remove `#[allow(unused)]` or dead helper code

## When NOT to Use This Skill

- `routes_ollama/` tests (separate conventions)
- Integration tests in `routes_all` (auth middleware integration)
- Frontend/UI tests (use the `playwright` skill instead)
- Service-layer tests in `services` crate (different patterns)
