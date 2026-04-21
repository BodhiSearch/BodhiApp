//! End-to-end live tests for the Anthropic Messages API proxy endpoints at
//! `/anthropic/v1/*`. Uses a mockito server as the upstream Anthropic provider
//! and verifies:
//! - request headers injected by `AiApiService::forward_request_with_method`
//!   match the Anthropic spec (`x-api-key` + `anthropic-version`)
//! - client-sent `anthropic-*` headers are extracted by the route handler and
//!   forwarded to upstream
//! - the `anthropic_auth_middleware` accepts `x-api-key: bodhiapp_<token>` as
//!   an alternative to `Authorization: Bearer`
//! - `/anthropic/v1/models` aggregates models from DB without calling upstream

mod utils;

use anyhow_trace::anyhow_trace;
use mockito::Server as MockServer;
use pretty_assertions::assert_eq;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use utils::{create_test_session_for_live_server, start_test_live_server};

/// Create an Anthropic-format API alias via the real REST endpoint.
/// Returns the alias id from the create response.
async fn create_anthropic_alias(
  client: &Client,
  base_url: &str,
  cookie: &str,
  upstream_url: &str,
  models: Vec<&str>,
) -> anyhow::Result<String> {
  let resp = client
    .post(format!("{}/bodhi/v1/models/api", base_url))
    .header("Cookie", cookie)
    .json(&json!({
      "api_format": "anthropic",
      "base_url": upstream_url,
      "api_key": {"action": "set", "value": "sk-ant-test-key"},
      "models": models,
    }))
    .send()
    .await?;
  assert_eq!(
    StatusCode::CREATED,
    resp.status(),
    "Failed to create Anthropic API alias: {}",
    resp.text().await?
  );
  let body: Value = resp.json().await?;
  Ok(body["id"].as_str().unwrap().to_string())
}

/// POST /anthropic/v1/messages successfully proxies to upstream with
/// `x-api-key` + `anthropic-version` headers injected from the stored alias.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_anthropic_messages_proxy_injects_auth_headers() -> anyhow::Result<()> {
  let mut mock_server = MockServer::new_async().await;
  mock_server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"data":[{"id":"claude-sonnet-4-5-20250929","display_name":"Claude 3.5 Sonnet","created_at":"2024-10-22T00:00:00Z","type":"model"}],"has_more":false}"#)
    .create_async()
    .await;
  let mock = mock_server
    .mock("POST", "/messages")
    .match_header("x-api-key", "sk-ant-test-key")
    .match_header("anthropic-version", "2023-06-01")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"id":"msg_123","content":[{"type":"text","text":"Hi!"}]}"#)
    .create_async()
    .await;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();
  let (cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  let _alias_id = create_anthropic_alias(
    &client,
    &server.base_url,
    &cookie,
    &mock_server.url(),
    vec!["claude-sonnet-4-5-20250929"],
  )
  .await?;

  let resp = client
    .post(format!("{}/anthropic/v1/messages", server.base_url))
    .header("Cookie", &cookie)
    .json(&json!({
      "model": "claude-sonnet-4-5-20250929",
      "max_tokens": 50,
      "messages": [{"role": "user", "content": "Hello"}]
    }))
    .send()
    .await?;

  assert_eq!(StatusCode::OK, resp.status());
  let body: Value = resp.json().await?;
  assert_eq!("msg_123", body["id"].as_str().unwrap());
  assert_eq!("Hi!", body["content"][0]["text"].as_str().unwrap());

  mock.assert_async().await;

  server.handle.shutdown().await?;
  Ok(())
}

/// Client-sent `anthropic-beta` header is forwarded to the upstream.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_anthropic_messages_proxy_forwards_anthropic_beta_header() -> anyhow::Result<()> {
  let mut mock_server = MockServer::new_async().await;
  mock_server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"data":[{"id":"claude-sonnet-4-5-20250929","display_name":"Claude 3.5 Sonnet","created_at":"2024-10-22T00:00:00Z","type":"model"}],"has_more":false}"#)
    .create_async()
    .await;
  let mock = mock_server
    .mock("POST", "/messages")
    .match_header("x-api-key", "sk-ant-test-key")
    .match_header("anthropic-beta", "token-counting-2024-11-01")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"id":"msg_456","content":[{"type":"text","text":"ok"}]}"#)
    .create_async()
    .await;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();
  let (cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  let _alias_id = create_anthropic_alias(
    &client,
    &server.base_url,
    &cookie,
    &mock_server.url(),
    vec!["claude-sonnet-4-5-20250929"],
  )
  .await?;

  let resp = client
    .post(format!("{}/anthropic/v1/messages", server.base_url))
    .header("Cookie", &cookie)
    .header("anthropic-beta", "token-counting-2024-11-01")
    .json(&json!({
      "model": "claude-sonnet-4-5-20250929",
      "max_tokens": 50,
      "messages": [{"role": "user", "content": "Hello"}]
    }))
    .send()
    .await?;

  assert_eq!(StatusCode::OK, resp.status());
  mock.assert_async().await;

  server.handle.shutdown().await?;
  Ok(())
}

/// Non-Anthropic aliases are rejected with 400.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_anthropic_messages_proxy_rejects_wrong_format() -> anyhow::Result<()> {
  let mut mock_openai_server = MockServer::new_async().await;
  mock_openai_server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"object":"list","data":[{"id":"gpt-4o","object":"model","created":0,"owned_by":"openai"}]}"#)
    .create_async()
    .await;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();
  let (cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  // Create an OpenAI-format alias (wrong format for /anthropic/v1/messages).
  let resp = client
    .post(format!("{}/bodhi/v1/models/api", server.base_url))
    .header("Cookie", &cookie)
    .json(&json!({
      "api_format": "openai",
      "base_url": mock_openai_server.url(),
      "api_key": {"action": "set", "value": "sk-test"},
      "models": ["gpt-4o"],
    }))
    .send()
    .await?;
  assert_eq!(StatusCode::CREATED, resp.status());

  // Try to use it via the Anthropic endpoint.
  let resp = client
    .post(format!("{}/anthropic/v1/messages", server.base_url))
    .header("Cookie", &cookie)
    .json(&json!({
      "model": "gpt-4o",
      "max_tokens": 50,
      "messages": [{"role": "user", "content": "hi"}]
    }))
    .send()
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, resp.status());

  server.handle.shutdown().await?;
  Ok(())
}

/// GET /anthropic/v1/models aggregates the alias's models from DB.
/// Does NOT call upstream.
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_anthropic_models_list_aggregates_from_db() -> anyhow::Result<()> {
  let mut mock_server = MockServer::new_async().await;
  // Mock the /models endpoint called during create (fetch_models validates model IDs).
  // Use expect(2) because two aliases are created.
  mock_server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"data":[{"id":"claude-sonnet-4-5-20250929","display_name":"Claude 3.5 Sonnet","created_at":"2024-10-22T00:00:00Z","type":"model"},{"id":"claude-opus-4-5-20251101","display_name":"Claude 3 Opus","created_at":"2024-02-29T00:00:00Z","type":"model"},{"id":"claude-haiku-4-5-20251001","display_name":"Claude 3 Haiku","created_at":"2024-03-07T00:00:00Z","type":"model"}],"has_more":false}"#)
    .expect(2)
    .create_async()
    .await;

  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();
  let (cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  // Two Anthropic aliases with overlapping models to exercise dedup.
  let _a = create_anthropic_alias(
    &client,
    &server.base_url,
    &cookie,
    &mock_server.url(),
    vec!["claude-sonnet-4-5-20250929", "claude-opus-4-5-20251101"],
  )
  .await?;
  let _b = create_anthropic_alias(
    &client,
    &server.base_url,
    &cookie,
    &mock_server.url(),
    vec!["claude-sonnet-4-5-20250929", "claude-haiku-4-5-20251001"],
  )
  .await?;

  let resp = client
    .get(format!("{}/anthropic/v1/models", server.base_url))
    .header("Cookie", &cookie)
    .send()
    .await?;
  assert_eq!(StatusCode::OK, resp.status());
  let body: Value = resp.json().await?;
  let data = body["data"].as_array().unwrap();
  assert_eq!(3, data.len(), "expected 3 unique models, got {:?}", data);
  let ids: Vec<&str> = data.iter().map(|m| m["id"].as_str().unwrap()).collect();
  assert!(ids.contains(&"claude-sonnet-4-5-20250929"));
  assert!(ids.contains(&"claude-opus-4-5-20251101"));
  assert!(ids.contains(&"claude-haiku-4-5-20251001"));
  assert_eq!(false, body["has_more"].as_bool().unwrap());

  server.handle.shutdown().await?;
  Ok(())
}

/// Missing model field returns 400 (validation).
#[anyhow_trace]
#[tokio::test]
#[serial_test::serial(live)]
async fn test_anthropic_messages_proxy_missing_model() -> anyhow::Result<()> {
  let server = start_test_live_server().await?;
  let client = reqwest::Client::new();
  let (cookie, _user_id) =
    create_test_session_for_live_server(&server.app_service, &["resource_user"]).await?;

  let resp = client
    .post(format!("{}/anthropic/v1/messages", server.base_url))
    .header("Cookie", &cookie)
    .json(&json!({
      "max_tokens": 50,
      "messages": [{"role": "user", "content": "hi"}]
    }))
    .send()
    .await?;
  assert_eq!(StatusCode::BAD_REQUEST, resp.status());

  server.handle.shutdown().await?;
  Ok(())
}
