use crate::ai_apis::clients::liberty_anthropic::LibertyAnthropicClient;
use crate::ai_apis::llm_liberty::refresh::MockLlmLibertyRefresh;
use crate::ai_apis::llm_liberty::LlmLibertyRefreshError;
use crate::ai_apis::AiApiClient;
use crate::models::llm_liberty_envelope::{LlmLibertyEnvelope, ResolvedLlmLibertyCredentials};
use crate::SafeReqwest;
use anyhow_trace::anyhow_trace;
use axum::http::Method;
use chrono::{Duration, Utc};
use mockito::Server;
use pretty_assertions::assert_eq;
use rstest::rstest;
use std::sync::Arc;

fn safe_http() -> SafeReqwest {
  SafeReqwest::builder()
    .allow_private_ips()
    .build()
    .expect("safe reqwest builder")
}

fn make_envelope(chat_url: &str, models_url: Option<&str>) -> LlmLibertyEnvelope {
  use crate::models::llm_liberty_envelope::{
    LlmLibertyApiEndpoints, LlmLibertyAuthSpec, LlmLibertyOauthEndpoints,
  };
  LlmLibertyEnvelope {
    version: "1.0.0".into(),
    provider: "anthropic".into(),
    access_token: "test-access-token".into(),
    refresh_token: "test-refresh-token".into(),
    expires_at: (Utc::now() + Duration::hours(1)).timestamp(),
    auth: LlmLibertyAuthSpec {
      location: "header".into(),
      key: "Authorization".into(),
      scheme: "Bearer".into(),
    },
    oauth: LlmLibertyOauthEndpoints {
      authorize_url: "https://oauth.example/authorize".into(),
      token_url: "https://oauth.example/token".into(),
      revoke_url: None,
      client_id: "client-id".into(),
      client_secret: None,
    },
    api: LlmLibertyApiEndpoints {
      base_url: "https://api.example.com".into(),
      chat_url: chat_url.to_string(),
      models_url: models_url.map(String::from),
    },
    headers: serde_json::json!({}),
    body: serde_json::json!({}),
    extra: None,
  }
}

fn make_creds(
  chat_url: &str,
  models_url: Option<&str>,
  access_token: &str,
) -> ResolvedLlmLibertyCredentials {
  ResolvedLlmLibertyCredentials {
    access_token: access_token.to_string(),
    refresh_token: "refresh-token".to_string(),
    expires_at: Utc::now() + Duration::hours(1),
    tenant_id: "tenant-a".to_string(),
    provider: "anthropic".to_string(),
    auth_scheme: "Bearer".to_string(),
    auth_key: "Authorization".to_string(),
    oauth_token_url: "https://oauth.example/token".to_string(),
    oauth_client_id: "client-id".to_string(),
    oauth_client_secret: None,
    api_base_url: "https://api.example.com".to_string(),
    api_chat_url: chat_url.to_string(),
    api_models_url: models_url.map(String::from),
    headers_json: serde_json::json!({}),
    body_json: serde_json::json!({}),
    extra_json: None,
  }
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_prompt_success_from_envelope() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/v1/messages", server.url());

  let _mock = server
    .mock("POST", "/v1/messages")
    .match_header("authorization", "Bearer test-access-token")
    .match_header("anthropic-version", "2023-06-01")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"content": [{"type": "text", "text": "pong"}]}"#)
    .create_async()
    .await;

  let envelope = make_envelope(&chat_url, None);
  let client = LibertyAnthropicClient::from_envelope(&envelope, safe_http());
  let result = client.test_prompt("claude-3-5-sonnet", "ping").await?;
  assert_eq!("pong", result);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn fetch_models_success_from_envelope() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/v1/messages", server.url());
  let models_url = format!("{}/v1/models", server.url());

  let _mock = server
    .mock("GET", "/v1/models")
    .match_header("authorization", "Bearer test-access-token")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(serde_json::json!({
      "data": [
        {"type": "model", "id": "claude-3-5-sonnet-20241022", "display_name": "Claude 3.5 Sonnet", "created_at": "2024-10-22T00:00:00Z"}
      ],
      "has_more": false
    }).to_string())
    .create_async()
    .await;

  let envelope = make_envelope(&chat_url, Some(&models_url));
  let client = LibertyAnthropicClient::from_envelope(&envelope, safe_http());
  let models = client.fetch_models().await?;
  assert_eq!(1, models.len());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn fetch_models_returns_empty_when_no_models_url() -> anyhow::Result<()> {
  let server = Server::new_async().await;
  let chat_url = format!("{}/v1/messages", server.url());

  let envelope = make_envelope(&chat_url, None);
  let client = LibertyAnthropicClient::from_envelope(&envelope, safe_http());
  let models = client.fetch_models().await?;
  assert!(models.is_empty());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn forward_retries_on_401_with_force_refresh() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/v1/messages", server.url());

  let _first_call = server
    .mock("POST", "/v1/messages")
    .match_header("authorization", "Bearer old-token")
    .with_status(401)
    .with_body(r#"{"error": "unauthorized"}"#)
    .create_async()
    .await;

  let _second_call = server
    .mock("POST", "/v1/messages")
    .match_header("authorization", "Bearer new-token")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"ok": true}"#)
    .create_async()
    .await;

  let mut mock_refresh = MockLlmLibertyRefresh::new();
  let fresh_creds = make_creds(&chat_url, None, "new-token");
  mock_refresh
    .expect_force_refresh()
    .times(1)
    .returning(move |_, _, _| Ok(fresh_creds.clone()));

  let creds = make_creds(&chat_url, None, "old-token");
  let client = LibertyAnthropicClient::from_credentials(
    &creds,
    "alias-1",
    None,
    "tenant-a",
    "user-a",
    Arc::new(mock_refresh),
    safe_http(),
  );

  let result = client
    .forward_request_with_method(
      &Method::POST,
      "/messages",
      Some(serde_json::json!({"model": "claude-3", "messages": []})),
      None,
      None,
    )
    .await?;

  assert_eq!(200, result.status().as_u16());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn forward_propagates_second_401() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/v1/messages", server.url());

  let _first_call = server
    .mock("POST", "/v1/messages")
    .with_status(401)
    .with_body(r#"{"error": "unauthorized"}"#)
    .create_async()
    .await;

  let _second_call = server
    .mock("POST", "/v1/messages")
    .with_status(401)
    .with_body(r#"{"error": "still unauthorized"}"#)
    .create_async()
    .await;

  let mut mock_refresh = MockLlmLibertyRefresh::new();
  let fresh_creds = make_creds(&chat_url, None, "new-token");
  mock_refresh
    .expect_force_refresh()
    .times(1)
    .returning(move |_, _, _| Ok(fresh_creds.clone()));

  let creds = make_creds(&chat_url, None, "old-token");
  let client = LibertyAnthropicClient::from_credentials(
    &creds,
    "alias-1",
    None,
    "tenant-a",
    "user-a",
    Arc::new(mock_refresh),
    safe_http(),
  );

  let result = client
    .forward_request_with_method(
      &Method::POST,
      "/messages",
      Some(serde_json::json!({"model": "claude-3", "messages": []})),
      None,
      None,
    )
    .await?;

  assert_eq!(401, result.status().as_u16());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn forward_skips_retry_when_no_alias_id() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/v1/messages", server.url());

  let _mock = server
    .mock("POST", "/v1/messages")
    .with_status(401)
    .with_body(r#"{"error": "unauthorized"}"#)
    .create_async()
    .await;

  let envelope = make_envelope(&chat_url, None);
  let client = LibertyAnthropicClient::from_envelope(&envelope, safe_http());

  let result = client
    .forward_request_with_method(
      &Method::POST,
      "/messages",
      Some(serde_json::json!({"model": "claude-3", "messages": []})),
      None,
      None,
    )
    .await?;

  assert_eq!(401, result.status().as_u16());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn forward_refresh_error_surfaced_as_api_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/v1/messages", server.url());

  let _mock = server
    .mock("POST", "/v1/messages")
    .with_status(401)
    .create_async()
    .await;

  let mut mock_refresh = MockLlmLibertyRefresh::new();
  mock_refresh
    .expect_force_refresh()
    .times(1)
    .returning(|_, _, _| Err(LlmLibertyRefreshError::NotFound("alias gone".to_string())));

  let creds = make_creds(&chat_url, None, "old-token");
  let client = LibertyAnthropicClient::from_credentials(
    &creds,
    "alias-1",
    None,
    "tenant-a",
    "user-a",
    Arc::new(mock_refresh),
    safe_http(),
  );

  let result = client
    .forward_request_with_method(
      &Method::POST,
      "/messages",
      Some(serde_json::json!({"model": "claude-3", "messages": []})),
      None,
      None,
    )
    .await;

  assert!(result.is_err());
  Ok(())
}
