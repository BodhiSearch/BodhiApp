# Request Construction

## RequestTestExt -- JSON Body

Import: `use server_core::test_utils::RequestTestExt;`

### POST with typed body

```rust
Request::post("/v1/chat/completions").json(request)?
```

### POST with inline JSON

```rust
Request::post("/api/models").json(json!({
  "model": "test-model",
  "messages": [{"role": "user", "content": "Hello"}]
}))?
```

### POST with raw JSON string

```rust
Request::post("/setup").json_str(r#"{"invalid": json"#)?
```

### GET with empty body

```rust
Request::get("/api/models").body(Body::empty())?
```

### PUT/DELETE with method builder

```rust
Request::builder()
  .method(Method::PUT)
  .uri("/api/aliases/test-alias")
  .json(&update_payload)?
```

```rust
Request::builder()
  .method(Method::DELETE)
  .uri("/api/models/test-id")
  .body(Body::empty())?
```

## RequestAuthExt -- Auth Headers

Import: `use server_core::test_utils::RequestAuthExt;`

Sets `X-BodhiApp-Token` + `X-BodhiApp-Role` or `X-BodhiApp-Scope` headers.

### Session-based user auth

```rust
Request::post("/api/tokens")
  .with_user_auth(&token, "resource_admin")
  .json(&body)?
```

### API token auth

```rust
Request::get("/api/data")
  .with_api_token(&token, "scope_token_user")
  .body(Body::empty())?
```

### Token-only (no role -- testing missing role error)

```rust
Request::get("/api/tokens?page=1&page_size=10")
  .header("X-BodhiApp-Token", &token)
  .body(Body::empty())?
```

### Custom headers alongside auth

```rust
Request::post("/endpoint")
  .with_user_auth(&token, "resource_admin")
  .header("Host", "localhost:1135")
  .json(&body)?
```

### User identity headers (no token)

```rust
Request::post(ENDPOINT_USER_REQUEST_ACCESS)
  .header(KEY_HEADER_BODHIAPP_USERNAME, "user@example.com")
  .header(KEY_HEADER_BODHIAPP_USER_ID, "user-id-123")
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
