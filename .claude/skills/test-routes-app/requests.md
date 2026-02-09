# Request Construction

## Request Helpers for build_test_router()

### Unauthenticated request

```rust
let response = router
  .oneshot(unauth_request("GET", "/bodhi/v1/models"))
  .await
  .unwrap();
```

Sets `Host: localhost:1135` header only. No auth.

### Session-authenticated request

```rust
let cookie = create_authenticated_session(
  app_service.session_service().as_ref(),
  &["resource_user"],
).await.unwrap();

let response = router
  .oneshot(session_request("GET", "/bodhi/v1/models", &cookie))
  .await
  .unwrap();
```

Sets `Cookie`, `Sec-Fetch-Site: same-origin`, and `Host: localhost:1135` headers.

### Multiple requests with same router

Use `#[values]` for cartesian product testing (preferred):

```rust
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_endpoints_reject_insufficient_role(
  #[case] role: &str,
  #[values("GET", "POST")] method: &str,
  #[values("/path/a", "/path/b")] path: &str,
) -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(), &[role]
  ).await?;

  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  assert_eq!(StatusCode::FORBIDDEN, response.status());
  Ok(())
}
```

## Request Construction with AppServiceStubBuilder

### RequestTestExt -- JSON Body

```rust
// POST with typed body
Request::post("/v1/chat/completions").json(request)?

// POST with inline JSON
Request::post("/api/models").json(json!({
  "model": "test-model",
  "messages": [{"role": "user", "content": "Hello"}]
}))?

// POST with raw JSON string
Request::post("/setup").json_str(r#"{"invalid": json"#)?

// GET with empty body
Request::get("/api/models").body(Body::empty())?

// PUT/DELETE with method builder
Request::builder()
  .method(Method::PUT)
  .uri("/api/aliases/test-alias")
  .json(&update_payload)?
```

### RequestAuthExt -- Auth Headers

Sets `X-BodhiApp-Token` + `X-BodhiApp-Role` or `X-BodhiApp-Scope` headers.
**Only for isolated handler tests with AppServiceStubBuilder** -- auth tier tests use real session auth.

```rust
// Session-based user auth
Request::post("/api/tokens")
  .with_user_auth(&token, "resource_admin")
  .json(&body)?

// API token auth
Request::get("/api/data")
  .with_api_token(&token, "scope_token_user")
  .body(Body::empty())?

// Token-only (no role -- testing missing role error)
Request::get("/api/tokens?page=1&page_size=10")
  .header("X-BodhiApp-Token", &token)
  .body(Body::empty())?
```

## Pagination Requests

```rust
let response = router
  .oneshot(
    Request::get("/api/tokens?page=1&page_size=10")
      .header("X-BodhiApp-Token", &token)
      .body(Body::empty())?
  )
  .await?;
```

## Sending the Request

Always use `tower::ServiceExt::oneshot()` for single-request tests:

```rust
let response = router.oneshot(request).await?;
```

For multi-request flows with cookies, use `axum_test::TestServer` instead (see [advanced.md](advanced.md)).
