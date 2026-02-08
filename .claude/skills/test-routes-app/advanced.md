# Advanced Patterns

## Background Task Testing (wait_for_event!)

For handlers that spawn background tasks (e.g. model pull/download), subscribe to DB events before the request and wait after.

Import: `use crate::wait_for_event;`

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

For multi-request flows needing cookies (OAuth login/callback), use `axum_test::TestServer`:

```rust
let router = Router::new()
  .route("/auth/initiate", post(auth_initiate_handler))
  .route("/auth/callback", post(auth_callback_handler))
  .layer(app_service.session_service().session_layer())
  .with_state(state);

let mut client = TestServer::new(router)?;
client.save_cookies();

// First request
let login_resp = client.post("/auth/initiate").await;
login_resp.assert_status(StatusCode::CREATED);

// Second request (cookies auto-forwarded)
let callback_resp = client
  .post("/auth/callback")
  .json(&json!({"code": "test_code", "state": state_value}))
  .await;
callback_resp.assert_status(StatusCode::OK);
```

**When to use TestServer vs oneshot:**
- `oneshot` -- single request, no session state (vast majority of tests)
- `TestServer` -- multi-request flows with cookies (login, OAuth callback)

**Manual cookie injection with oneshot** (for tests that set up session state directly in DB):

```rust
Request::post("/auth/callback")
  .header("Cookie", format!("bodhiapp_session_id={}", session_id))
  .json(json!({"code": "test_code", "state": state_value}))?
```

## Parameterized Tests (#[case])

Use `#[case]` for testing multiple scenarios with the same test body:

```rust
#[rstest]
#[case::user_scope("resource_user", TokenScope::User, "scope_token_user")]
#[case::admin_scope("resource_admin", TokenScope::User, "scope_token_user")]
#[case::power_user("resource_admin", TokenScope::PowerUser, "scope_token_power_user")]
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

### Status code parameterization

```rust
#[rstest]
#[case::success("valid-name", StatusCode::CREATED, None)]
#[case::missing("", StatusCode::BAD_REQUEST, Some("validation_error"))]
#[case::conflict("existing", StatusCode::CONFLICT, Some("entity_error-conflict"))]
#[tokio::test]
#[anyhow_trace]
async fn test_create_handler(
  #[case] name: &str,
  #[case] expected_status: StatusCode,
  #[case] expected_error: Option<&str>,
) -> anyhow::Result<()> {
  // ...
  assert_eq!(expected_status, response.status());
  if let Some(code) = expected_error {
    let body = response.json::<Value>().await?;
    assert_eq!(code, body["error"]["code"].as_str().unwrap());
  }
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
    Matcher::UrlEncoded("code".into(), "test_code".into()),
  ]))
  .with_status(200)
  .with_body(json!({"access_token": "tok", "refresh_token": "ref"}).to_string())
  .create_async()
  .await;
```

## SSE Stream Construction (for mocking)

Build a mock SSE stream response for `MockSharedContext::forward_request`:

```rust
fn streamed_response() -> Result<reqwest::Response, ContextError> {
  let stream = futures_util::stream::iter(["chunk1", "chunk2", "chunk3"])
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

- **Inline** (`mod tests` in source file): Only for <5 simple tests
- **Separate file** (`<module>_test.rs`): For standalone modules with 5+ tests
- **Tests directory** (`<module>/tests/*.rs`): For domain modules with multiple test files (e.g. `routes_oai/tests/chat_test.rs`, `routes_oai/tests/models_test.rs`)

## Coverage Commands

```bash
# Per-crate coverage report
cargo llvm-cov --package routes_app --text

# Run only matching tests
cargo test -p routes_app -- routes_oai
cargo test -p routes_app -- test_create_token

# Quick compile check before full test
cargo check -p routes_app
```
