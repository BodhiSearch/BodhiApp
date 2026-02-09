# Advanced Patterns

## Auth Test Organization

Auth tests are organized as **per-module test files** in `crates/routes_app/tests/`:

```
tests/
  test_router_smoke.rs          # 3 basic smoke tests
  routes_setup_auth_test.rs     # Public endpoint auth
  routes_settings_auth_test.rs  # Admin tier auth
  routes_models_auth_test.rs    # User + PowerUser tiers
  routes_models_metadata_auth_test.rs
  routes_models_pull_auth_test.rs
  routes_users_info_auth_test.rs          # Optional auth
  routes_users_access_request_auth_test.rs # Manager tier
  routes_users_management_auth_test.rs     # Manager tier
  routes_api_token_auth_test.rs  # PowerUser session tier
  routes_auth_auth_test.rs       # Optional auth (OAuth)
  routes_toolsets_auth_test.rs   # User session/OAuth tiers
  routes_oai_auth_test.rs        # User tier
  routes_ollama_auth_test.rs     # User tier
  routes_api_models_auth_test.rs # PowerUser tier
```

### Auth Test Template

Each auth test file tests three categories:

1. **Unauthenticated rejection (401)** -- rstest with endpoint cases
2. **Insufficient role rejection (403)** -- roles below the tier's minimum
3. **Authorized access (not 401/403)** -- only for endpoints with real services

```rust
// 1. Unauthenticated
#[rstest]
#[case::endpoint_a("GET", "/path/a")]
#[case::endpoint_b("POST", "/path/b")]
#[tokio::test]
async fn test_endpoints_reject_unauthenticated(#[case] method: &str, #[case] path: &str) {
  let (router, _, _temp) = build_test_router().await.unwrap();
  let response = router
    .oneshot(unauth_request(method, path))
    .await
    .unwrap();
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

// 2. Insufficient role
#[rstest]
#[case::user("resource_user")]
#[tokio::test]
async fn test_endpoints_reject_insufficient_role(#[case] role: &str) {
  let endpoints = vec![("GET", "/path/a"), ("POST", "/path/b")];
  let (router, app_service, _temp) = build_test_router().await.unwrap();
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(), &[role]
  ).await.unwrap();

  for (method, path) in endpoints {
    let response = router.clone()
      .oneshot(session_request(method, path, &cookie))
      .await.unwrap();
    assert_eq!(StatusCode::FORBIDDEN, response.status(),
      "{role} should be forbidden from {method} {path}");
  }
}

// 3. Authorized
#[rstest]
#[case::safe_endpoint("GET", "/path/a")]
#[tokio::test]
async fn test_endpoints_allow_role(#[case] method: &str, #[case] path: &str) {
  let (router, app_service, _temp) = build_test_router().await.unwrap();
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(), &["resource_admin"]
  ).await.unwrap();
  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await.unwrap();
  assert_ne!(StatusCode::UNAUTHORIZED, response.status());
  assert_ne!(StatusCode::FORBIDDEN, response.status());
}
```

### Skipping "Allowed" Tests

When a handler calls a mock service (MockAuthService, MockToolService, MockSharedContext), the "allowed" test would panic. Skip these with a comment:

```rust
// Manager-allowed tests skipped: all endpoints call MockAuthService which panics
// without expectations. Auth middleware is proven by the access-request tests
// which share the same route_layer.
```

## Background Task Testing (wait_for_event!)

For handlers that spawn background tasks (e.g. model pull/download):

```rust
// 1. Subscribe BEFORE the request
let mut rx = db_service.subscribe();

// 2. Send request (triggers background task)
let response = router.oneshot(Request::post("/modelfiles/pull").json(&payload)?).await?;
assert_eq!(StatusCode::CREATED, response.status());

// 3. Wait for background task completion
let received = wait_for_event!(rx, "update_download_request", Duration::from_millis(500));
assert!(received, "Timed out waiting for update_download_request");

// 4. Assert final DB state
let final_status = db_service.get_download_request(&id).await?.unwrap();
assert_eq!(DownloadStatus::Completed, final_status.status);
```

## Session/Cookie Tests (TestServer)

For multi-request flows needing cookies (OAuth login/callback):

```rust
let router = Router::new()
  .route("/auth/initiate", post(auth_initiate_handler))
  .route("/auth/callback", post(auth_callback_handler))
  .layer(app_service.session_service().session_layer())
  .with_state(state);

let mut client = TestServer::new(router)?;
client.save_cookies();

let login_resp = client.post("/auth/initiate").await;
login_resp.assert_status(StatusCode::CREATED);

let callback_resp = client
  .post("/auth/callback")
  .json(&json!({"code": "test_code", "state": state_value}))
  .await;
callback_resp.assert_status(StatusCode::OK);
```

**When to use TestServer vs oneshot:**
- `oneshot` -- single request, no session state (vast majority of tests)
- `TestServer` -- multi-request flows with cookies (login, OAuth callback)

## Parameterized Tests (#[case])

```rust
#[rstest]
#[case::user_scope("resource_user", TokenScope::User, "scope_token_user")]
#[case::admin_scope("resource_admin", TokenScope::User, "scope_token_user")]
#[awt]
#[tokio::test]
#[anyhow_trace]
async fn test_create_token_role_scope_mapping(
  #[case] role: &str,
  #[case] requested_scope: TokenScope,
  #[case] expected_scope: &str,
  #[future] test_db_service: TestDbService,
) -> anyhow::Result<()> {
  // ... shared test body ...
  Ok(())
}
```

## Mock HTTP Server (mockito)

For testing handlers that call external APIs (Keycloak, HuggingFace):

```rust
let mut server = Server::new_async().await;
server
  .mock("POST", "/realms/test-realm/protocol/openid-connect/token")
  .match_body(Matcher::AllOf(vec![
    Matcher::UrlEncoded("grant_type".into(), "authorization_code".into()),
  ]))
  .with_status(200)
  .with_body(json!({"access_token": "tok"}).to_string())
  .create_async()
  .await;
```

## SSE Stream Construction (for mocking)

```rust
fn streamed_response() -> Result<reqwest::Response, ContextError> {
  let stream = futures_util::stream::iter(["chunk1", "chunk2"])
    .enumerate()
    .map(|(i, value)| {
      let chunk = json!({
        "id": format!("chatcmpl-{i}"),
        "object": "chat.completion.chunk",
        "choices": [{"index": 0, "delta": {"content": value}}]
      });
      let response: CreateChatCompletionStreamResponse =
        serde_json::from_value(chunk).expect("valid chunk");
      format!("data: {}\n\n", serde_json::to_string(&response).expect("serialize"))
    })
    .then(|chunk| async move {
      tokio::time::sleep(Duration::from_millis(1)).await;
      Ok::<_, std::io::Error>(chunk)
    });
  let body = reqwest::Body::wrap_stream(stream);
  Ok(reqwest::Response::from(
    http::Response::builder()
      .status(200)
      .header("content-type", "text/event-stream")
      .body(body)?,
  ))
}
```

## Test File Organization

- **Router-level auth tests**: `crates/routes_app/tests/<module>_auth_test.rs`
- **Handler-level inline** (`mod tests` in source file): Only for <5 simple tests
- **Handler-level separate** (`<module>/tests/<name>_test.rs`): For modules with 5+ tests

## Coverage Commands

```bash
cargo test -p routes_app                    # All tests
cargo test -p routes_app -- routes_oai      # Module filter
cargo test -p routes_app --test routes_oai_auth_test  # Specific auth test file
cargo check -p routes_app                   # Quick compile check
```
