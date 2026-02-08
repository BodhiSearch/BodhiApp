# HTTP Service Testing with mockito

## Overview

Services that make HTTP calls (AuthService, AiApiService, ExaService) use `mockito::Server` to mock external endpoints. The pattern creates a temporary HTTP server and passes its URL to the service under test.

## Basic Pattern

```rust
use mockito::Server;
use rstest::rstest;
use serde_json::json;

#[rstest]
#[tokio::test]
async fn test_service_http_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
    .mock("POST", "/api/endpoint")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(json!({ "result": "success" }).to_string())
    .create_async()
    .await;

  // Create service pointing at mock server
  let service = MyService::new(&server.url());
  let result = service.call_api().await?;

  assert_eq!("success", result);
  Ok(())
}
```

## AuthService Testing

AuthService uses a factory function `test_auth_service(url)` that configures a `KeycloakAuthService` with the mock server URL:

```rust
use crate::test_utils::test_auth_service;
use mockito::{Matcher, Server};

#[rstest]
#[tokio::test]
async fn test_auth_register_client() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let mock_server = server
    .mock("POST", "/realms/test-realm/bodhi/resources")
    .with_status(201)
    .with_header("content-type", "application/json")
    .with_body(json!({
      "client_id": "test-client",
      "client_secret": "test-secret"
    }).to_string())
    .create();

  let service = test_auth_service(&url);
  let result = service.register_client(/* ... */).await;
  assert!(result.is_ok());

  mock_server.assert();  // Verify the mock was called
  Ok(())
}
```

## AiApiService Testing

AiApiService accepts the URL at call-time rather than construction time:

```rust
use crate::ai_api_service::{AiApiService, DefaultAiApiService};
use crate::test_utils::MockDbService;

#[rstest]
#[tokio::test]
async fn test_api_call_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let mock_db = MockDbService::new();
  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));

  let _mock = server
    .mock("POST", "/chat/completions")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"choices": [{"message": {"content": "Hello response"}}]}"#)
    .create_async()
    .await;

  let result = service
    .test_prompt(Some("test-key".to_string()), &url, "gpt-3.5-turbo", "Hello")
    .await?;
  assert_eq!("Hello response", result);
  Ok(())
}
```

## Request Matching

Use `mockito::Matcher` for precise request validation:

### Header matching
```rust
server
  .mock("POST", "/search")
  .match_header("x-api-key", "test-key")
  .match_header("content-type", "application/json")
```

### JSON body matching
```rust
use mockito::Matcher;

server
  .mock("POST", "/search")
  .match_body(Matcher::JsonString(
    json!({
      "query": "rust programming",
      "numResults": 5
    }).to_string(),
  ))
```

### Form-encoded body matching
```rust
server
  .mock("POST", "/token")
  .match_header("content-type", "application/x-www-form-urlencoded")
  .match_body(Matcher::AllOf(vec![
    Matcher::UrlEncoded("grant_type".into(), "refresh_token".into()),
    Matcher::UrlEncoded("client_id".into(), client_id.into()),
  ]))
```

## Error Response Testing

Test error handling by returning non-200 status codes:

```rust
#[rstest]
#[tokio::test]
async fn test_api_unauthorized() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;

  let _mock = server
    .mock("POST", "/chat/completions")
    .with_status(401)
    .with_body("Invalid API key")
    .create_async()
    .await;

  let result = service.test_prompt(/* ... */).await;
  assert!(result.is_err());
  // Assert specific error variant or code
  Ok(())
}
```

## Mock Assertion

Call `.assert()` on the mock to verify it was called the expected number of times:

```rust
let mock = server.mock("POST", "/endpoint").create();
// ... test logic ...
mock.assert();  // Panics if not called exactly once
```

For async mocks:
```rust
let mock = server.mock("POST", "/endpoint").create_async().await;
// ... test logic ...
mock.assert_async().await;
```

## Sync vs Async Mock Creation

- Use `.create()` for sync tests or when the mock server is created synchronously
- Use `.create_async().await` for async tests to avoid blocking the runtime
- Use `Server::new_async().await` in async tests (preferred)
