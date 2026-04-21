use super::{AiApiService, DefaultAiApiService};
use crate::models::{ApiAlias, ApiFormat};
use crate::test_utils::{fixed_dt, openai_model};
use anyhow_trace::anyhow_trace;
use axum::http::Method;
use mockito::Server;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_anthropic_oauth_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let extra_headers = json!({"anthropic-beta": "oauth-2025-04-20"});
  let extra_body = json!({
    "system": [{"type": "text", "text": "You are Claude Code"}]
  });

  let expected_body = json!({
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 50,
    "messages": [{"role": "user", "content": "Hello"}],
    "system": [{"type": "text", "text": "You are Claude Code"}]
  });

  let _mock = server
    .mock("POST", "/messages")
    .match_header("authorization", "Bearer oauth-token-123")
    .match_header("anthropic-beta", "oauth-2025-04-20")
    .match_body(mockito::Matcher::JsonString(serde_json::to_string(
      &expected_body,
    )?))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"content": [{"type": "text", "text": "Hi from OAuth!"}]}"#)
    .create_async()
    .await;

  let result = service
    .test_prompt(
      Some("oauth-token-123".to_string()),
      &url,
      "claude-sonnet-4-5-20250929",
      "Hello",
      &ApiFormat::AnthropicOAuth,
      Some(extra_headers),
      Some(extra_body),
    )
    .await?;
  assert_eq!("Hi from OAuth!", result);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_anthropic_oauth_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let extra_headers = json!({
    "anthropic-beta": "oauth-2025-04-20",
    "user-agent": "claude-cli/2.1.80"
  });

  let _mock = server
    .mock("GET", "/models")
    .match_header("authorization", "Bearer oauth-token-123")
    .match_header("anthropic-beta", "oauth-2025-04-20")
    .match_header("anthropic-version", "2023-06-01")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{
      "data": [
        {"id": "claude-sonnet-4-5-20250929", "display_name": "Claude 3.5 Sonnet", "created_at": "2024-10-22T00:00:00Z", "type": "model", "capabilities": null, "max_input_tokens": null, "max_tokens": null}
      ],
      "has_more": false
    }"#)
    .create_async()
    .await;

  let models = service
    .fetch_models(
      Some("oauth-token-123".to_string()),
      &url,
      &ApiFormat::AnthropicOAuth,
      Some(extra_headers),
      None,
    )
    .await?;
  let model_ids: Vec<&str> = models.iter().map(|m| m.id()).collect();
  assert_eq!(vec!["claude-sonnet-4-5-20250929"], model_ids);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_request_anthropic_oauth_merges_body() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let extra_headers = json!({"anthropic-beta": "oauth-2025-04-20"});
  let extra_body = json!({
    "max_tokens": 32000,
    "system": [{"type": "text", "text": "ABC"}]
  });

  let api_alias = ApiAlias::new(
    "anthropic-oauth-api",
    ApiFormat::AnthropicOAuth,
    &url,
    vec![openai_model("claude-sonnet-4-5-20250929")],
    None,
    false,
    fixed_dt(),
    Some(extra_headers),
    Some(extra_body),
  );

  // Incoming has max_tokens: 10 (incoming wins) and no system (config applied)
  let expected_upstream_body = json!({
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 10,
    "messages": [{"role": "user", "content": "Hello"}],
    "system": [{"type": "text", "text": "ABC"}]
  });

  let _mock = server
    .mock("POST", "/messages")
    .match_header("authorization", "Bearer oauth-token-123")
    .match_header("anthropic-beta", "oauth-2025-04-20")
    .match_body(mockito::Matcher::JsonString(serde_json::to_string(
      &expected_upstream_body,
    )?))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"id":"msg_123","content":[{"type":"text","text":"Hi"}]}"#)
    .create_async()
    .await;

  let incoming = json!({
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 10,
    "messages": [{"role": "user", "content": "Hello"}]
  });

  let response = service
    .forward_request_with_method(
      &Method::POST,
      "/messages",
      &api_alias,
      Some("oauth-token-123".to_string()),
      Some(incoming),
      None,
      None,
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_request_anthropic_oauth_prepends_system() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let extra_headers = json!({"anthropic-beta": "oauth-2025-04-20"});
  let extra_body = json!({
    "system": [{"type": "text", "text": "CONFIG"}]
  });

  let api_alias = ApiAlias::new(
    "anthropic-oauth-api",
    ApiFormat::AnthropicOAuth,
    &url,
    vec![openai_model("claude-sonnet-4-5-20250929")],
    None,
    false,
    fixed_dt(),
    Some(extra_headers),
    Some(extra_body),
  );

  // Incoming has its own system — config must be prepended
  let expected_upstream_body = json!({
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 10,
    "messages": [{"role": "user", "content": "Hello"}],
    "system": [
      {"type": "text", "text": "CONFIG"},
      {"type": "text", "text": "USER"}
    ]
  });

  let _mock = server
    .mock("POST", "/messages")
    .match_header("authorization", "Bearer oauth-token-123")
    .match_body(mockito::Matcher::JsonString(serde_json::to_string(
      &expected_upstream_body,
    )?))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"id":"msg_123","content":[{"type":"text","text":"Hi"}]}"#)
    .create_async()
    .await;

  let incoming = json!({
    "model": "claude-sonnet-4-5-20250929",
    "max_tokens": 10,
    "messages": [{"role": "user", "content": "Hello"}],
    "system": [{"type": "text", "text": "USER"}]
  });

  let response = service
    .forward_request_with_method(
      &Method::POST,
      "/messages",
      &api_alias,
      Some("oauth-token-123".to_string()),
      Some(incoming),
      None,
      None,
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_request_anthropic_oauth_version_injected_when_absent() -> anyhow::Result<()> {
  // When extra_headers do not include anthropic-version and client does not either,
  // the default should be injected.
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let extra_headers = json!({"anthropic-beta": "oauth-2025-04-20"});

  let api_alias = ApiAlias::new(
    "anthropic-oauth-api",
    ApiFormat::AnthropicOAuth,
    &url,
    vec![openai_model("claude-sonnet-4-5-20250929")],
    None,
    false,
    fixed_dt(),
    Some(extra_headers),
    None,
  );

  let _mock = server
    .mock("POST", "/messages")
    .match_header("authorization", "Bearer oauth-token-123")
    .match_header("anthropic-version", "2023-06-01")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"id":"msg_123","content":[{"type":"text","text":"Hi"}]}"#)
    .create_async()
    .await;

  let response = service
    .forward_request_with_method(
      &Method::POST,
      "/messages",
      &api_alias,
      Some("oauth-token-123".to_string()),
      Some(json!({"model":"claude-sonnet-4-5-20250929","max_tokens":1,"messages":[]})),
      None,
      None,
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  Ok(())
}
