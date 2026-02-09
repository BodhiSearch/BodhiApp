# Advanced Patterns

## Auth Test Organization

Auth tests are now **inline in module test files** at `crates/routes_app/src/<module>/tests/*_test.rs`:

- Each module file contains both handler tests AND auth tests
- Auth tests placed at the bottom of the file after handler tests
- Example: `src/routes_oai/tests/chat_completions_test.rs` contains both handler logic tests and auth tier tests for OpenAI endpoints

### Auth Test Template

Auth tests follow three patterns, all using `#[anyhow_trace]` and `anyhow::Result<()>`:

1. **Unauthenticated rejection (401)** -- `#[case]` per endpoint
2. **Insufficient role rejection (403)** -- `#[values]` cartesian product (roles × endpoints)
3. **Authorized access (200/OK)** -- `#[values]` cartesian product (eligible roles × safe endpoints)

```rust
// 1. Unauthenticated rejection
#[rstest]
#[case::endpoint_a("GET", "/path/a")]
#[case::endpoint_b("POST", "/path/b")]
#[tokio::test]
#[anyhow_trace]
async fn test_endpoints_reject_unauthenticated(#[case] method: &str, #[case] path: &str) -> anyhow::Result<()> {
  let (router, _, _temp) = build_test_router().await?;
  let response = router
    .oneshot(unauth_request(method, path))
    .await?;
  assert_eq!(StatusCode::UNAUTHORIZED, response.status());
  Ok(())
}

// 2. Insufficient role rejection (cartesian product)
#[rstest]
#[case::user("resource_user")]
#[case::power_user("resource_power_user")]
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
  assert_eq!(StatusCode::FORBIDDEN, response.status(),
    "{role} should be forbidden from {method} {path}");
  Ok(())
}

// 3. Authorized access (safe endpoints only - see "Safe Endpoint Identification" below)
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_endpoints_allow_sufficient_role(
  #[values("resource_admin", "resource_manager")] role: &str,
  #[values("GET")] method: &str,
  #[values("/path/a")] path: &str,  // Only safe endpoints - see comment below
) -> anyhow::Result<()> {
  let (router, app_service, _temp) = build_test_router().await?;
  let cookie = create_authenticated_session(
    app_service.session_service().as_ref(), &[role]
  ).await?;

  let response = router
    .oneshot(session_request(method, path, &cookie))
    .await?;
  assert_eq!(StatusCode::OK, response.status());
  Ok(())
}
```

## Safe Endpoint Identification Pattern

When writing "allowed" tests with `build_test_router()`, you must identify **safe endpoints** that won't panic:

**Safe endpoints** use real services in `build_test_router()`:
- `DbService` (real SQLite)
- `DataService` (real file-based)
- `SessionService` (real SQLite)

**Unsafe endpoints** call mock services that panic without expectations:
- `MockAuthService`
- `MockToolService`
- `MockSharedContext`

**Rule of thumb:**
- GET list endpoints typically safe (return empty list/OK)
- Mutating endpoints or those calling external services typically unsafe

**Always document why endpoints are excluded:**

```rust
// Only testing GET /bodhi/v1/users (safe - uses real DbService)
// Excluding POST /bodhi/v1/users/*/role (calls MockAuthService - would panic)
#[rstest]
#[tokio::test]
#[anyhow_trace]
async fn test_endpoints_allow_manager(
  #[values("resource_manager", "resource_admin")] role: &str,
  #[values("GET")] method: &str,
  #[values("/bodhi/v1/users")] path: &str,
) -> anyhow::Result<()> {
  // ...
}
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

- **Auth tests**: Inline in `src/<module>/tests/*_test.rs` (bottom of file, after handler tests)
- **Handler-level inline** (`mod tests` in source file): Only for <5 simple tests
- **Handler-level separate** (`<module>/tests/<name>_test.rs`): For modules with 5+ tests

## Coverage Commands

```bash
cargo test -p routes_app                    # All tests
cargo test -p routes_app --lib -- routes_oai  # Module filter (includes auth + handler tests)
cargo check -p routes_app                   # Quick compile check
```
