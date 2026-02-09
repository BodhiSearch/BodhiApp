---
name: bodhi-app-e2e
description: >
  Use when writing end-to-end integration tests that require a real running
  server, OAuth2 authentication, or multi-step user flows. Covers live server
  setup, session management, chat completion testing, and model lifecycle.
  Examples: "write e2e test for chat completions", "add streaming test",
  "test OAuth flow end-to-end".
---

# BodhiApp End-to-End Test Skill

Write end-to-end tests using the `integration-tests` crate for full-stack validation.

## When to Use This vs Router-Level Tests

| Scenario | Use This Skill | Use test-routes-app |
|----------|---------------|---------------------|
| Real LLM inference (llama.cpp) | Yes | No |
| OAuth2 flow with real Keycloak | Yes | No |
| Streaming chat completions | Yes | No |
| Auth tier verification (401/403) | No | Yes |
| Handler business logic | No | Yes |
| Mock service expectations | No | Yes |

## Quick Start

```rust
#[rstest]
#[awt]
#[tokio::test]
#[serial_test::serial(live)]
#[timeout(Duration::from_secs(5 * 60))]
#[anyhow_trace::anyhow_trace]
async fn test_live_feature(
  #[future] live_server: TestServerHandle,
) -> anyhow::Result<()> {
  let server = live_server;
  let client = reqwest::Client::new();
  let url = format!("http://{}:{}/ping", server.host, server.port);
  let resp = client.get(&url).send().await?;
  assert_eq!(200, resp.status().as_u16());
  Ok(())
}
```

## Infrastructure

### Crate Location
`crates/integration-tests/`

### Test Files
```
tests/
  test_live_api_ping.rs                      # Basic connectivity
  test_live_chat_completions_non_streamed.rs  # Non-streaming chat
  test_live_chat_completions_streamed.rs      # Streaming chat (SSE)
  test_live_tool_calling_non_streamed.rs      # Tool calling
  test_live_tool_calling_streamed.rs          # Streamed tool calling
  test_live_thinking_disabled.rs             # Thinking mode
  test_live_agentic_chat_with_exa.rs         # Agentic with tools
  test_live_lib.rs                           # LLM server process
  utils/
    mod.rs
    live_server_utils.rs                     # TestServerHandle, fixtures
    tool_call.rs                             # Tool call helpers
```

### Key Fixtures

```rust
// Complete server with auth - most tests use this
#[fixture]
async fn live_server(
  #[future] llama2_7b_setup: (Arc<dyn AppService>, TempDir),
) -> TestServerHandle

// Service setup without server start
#[fixture]
async fn llama2_7b_setup() -> (Arc<dyn AppService>, TempDir)
```

### TestServerHandle

```rust
pub struct TestServerHandle {
  pub temp_cache_dir: TempDir,
  pub host: String,
  pub port: u16,
  pub handle: ServerShutdownHandle,
  pub app_service: Arc<dyn AppService>,
}
```

## Authentication in E2E Tests

### Session-Based Auth (Cookie)

```rust
// Get OAuth tokens from real Keycloak
let (access_token, _refresh_token) = get_oauth_tokens().await?;

// Create session in DB
let session_cookie = create_authenticated_session(
  &server.app_service, &access_token
).await?;

// Use cookie in requests
let resp = client
  .post(&url)
  .header("Cookie", &session_cookie)
  .header("Sec-Fetch-Site", "same-origin")
  .json(&body)
  .send()
  .await?;
```

### Environment Setup

Tests require `.env.test` at `tests/resources/.env.test` with:
- OAuth2 server URL and realm
- Client credentials (client_id, client_secret)
- Test user credentials

## Core Rules

1. **Serial execution**: Always use `#[serial_test::serial(live)]`
2. **Timeouts**: Always use `#[timeout(Duration::from_secs(5 * 60))]`
3. **Annotations**: `#[rstest]` + `#[awt]` + `#[tokio::test]` + `#[serial_test::serial(live)]` + `#[timeout]` + `#[anyhow_trace]`
4. **Cleanup**: TestServerHandle auto-cleans via Drop
5. **Small models**: Use Llama-68M variants for fast tests
6. **Error handling**: `-> anyhow::Result<()>` with `?` propagation

## Testing Patterns

### Non-Streaming Chat Completion

```rust
let body = json!({
  "model": "qwen3:1.7b-instruct",
  "messages": [{"role": "user", "content": "Say hello"}],
  "max_tokens": 50
});
let resp = client.post(&url)
  .header("Cookie", &cookie)
  .json(&body)
  .send().await?;
assert_eq!(200, resp.status().as_u16());
let result: Value = resp.json().await?;
assert!(result["choices"][0]["message"]["content"].is_string());
```

### Streaming Chat Completion

```rust
let body = json!({
  "model": "qwen3:1.7b-instruct",
  "messages": [{"role": "user", "content": "Hello"}],
  "stream": true,
  "max_tokens": 50
});
let resp = client.post(&url)
  .header("Cookie", &cookie)
  .json(&body)
  .send().await?;
assert_eq!(200, resp.status().as_u16());

// Parse SSE stream
let text = resp.text().await?;
let chunks: Vec<&str> = text.split("data: ")
  .filter(|s| !s.trim().is_empty() && *s != "[DONE]\n\n")
  .collect();
assert!(!chunks.is_empty());
```

## Running Tests

```bash
# All e2e tests (requires OAuth2 server + model files)
cargo test --package integration-tests

# Specific test
cargo test --package integration-tests test_live_api_ping

# With output for debugging
cargo test --package integration-tests -- --nocapture
```

## Adding New E2E Tests

1. Create `tests/test_live_<feature>.rs`
2. Use the `live_server` fixture for full server with auth
3. Add `#[serial_test::serial(live)]` and `#[timeout]`
4. Follow existing patterns for authentication
5. Clean up any created resources (tokens, sessions)
