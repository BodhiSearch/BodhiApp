use crate::ai_apis::clients::liberty_codex::LibertyCodexClient;
use crate::ai_apis::error::AiApiClientFactoryError;
use crate::ai_apis::llm_liberty::refresh::MockLlmLibertyRefresh;
use crate::ai_apis::llm_liberty::LlmLibertyRefreshError;
use crate::ai_apis::AiApiClient;
use crate::models::llm_liberty_envelope::{LlmLibertyEnvelope, ResolvedLlmLibertyCredentials};
use crate::test_utils::{
  test_llm_liberty_envelope_codex, test_resolved_llm_liberty_credentials_codex,
};
use crate::SafeReqwest;
use anyhow_trace::anyhow_trace;
use axum::http::Method;
use mockito::{Matcher, Server};
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
  let mut env = test_llm_liberty_envelope_codex();
  env.api.chat_url = chat_url.to_string();
  env.api.models_url = models_url.map(String::from);
  env
}

fn make_creds(
  chat_url: &str,
  models_url: Option<&str>,
  access_token: &str,
) -> ResolvedLlmLibertyCredentials {
  let mut creds = test_resolved_llm_liberty_credentials_codex();
  creds.access_token = access_token.to_string();
  creds.refresh_token = "refresh-token".to_string();
  creds.api_chat_url = chat_url.to_string();
  creds.api_models_url = models_url.map(String::from);
  creds
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_prompt_success_from_envelope() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/codex/responses", server.url());

  let sse_body = concat!(
    "event: response.output_text.delta\n",
    "data: {\"type\":\"response.output_text.delta\",\"delta\":\"pong\"}\n\n",
    "event: response.output_text.done\n",
    "data: {\"type\":\"response.output_text.done\",\"text\":\"pong\"}\n\n",
  );

  let _mock = server
    .mock("POST", "/codex/responses")
    .match_header("authorization", "Bearer test-access-token")
    .match_header("ChatGPT-Account-ID", "test-account-id")
    .with_status(200)
    .with_header("content-type", "text/event-stream")
    .with_body(sse_body)
    .create_async()
    .await;

  let envelope = make_envelope(&chat_url, None);
  let client = LibertyCodexClient::from_envelope(&envelope, safe_http());
  let result = client.test_prompt("gpt-5.2", "ping").await?;
  assert_eq!("pong", result);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn fetch_models_success_from_envelope() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/codex/responses", server.url());
  let models_url = format!("{}/codex/models", server.url());

  let _mock = server
    .mock("GET", "/codex/models?client_version=0.0.1")
    .match_header("authorization", "Bearer test-access-token")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      serde_json::json!({
        "models": [
          {"slug": "gpt-5.2", "display_name": "GPT-5.2"},
          {"slug": "gpt-5.2-codex", "display_name": "GPT-5.2 Codex"}
        ]
      })
      .to_string(),
    )
    .create_async()
    .await;

  let envelope = make_envelope(&chat_url, Some(&models_url));
  let client = LibertyCodexClient::from_envelope(&envelope, safe_http());
  let models = client.fetch_models().await?;
  assert_eq!(2, models.len());
  assert_eq!("gpt-5.2", models[0].id());
  assert_eq!("gpt-5.2-codex", models[1].id());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn fetch_models_returns_empty_when_no_models_url() -> anyhow::Result<()> {
  let server = Server::new_async().await;
  let chat_url = format!("{}/codex/responses", server.url());

  let envelope = make_envelope(&chat_url, None);
  let client = LibertyCodexClient::from_envelope(&envelope, safe_http());
  let models = client.fetch_models().await?;
  assert!(models.is_empty());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn forward_post_responses_merges_envelope_body() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/codex/responses", server.url());

  let _mock = server
    .mock("POST", "/codex/responses")
    .match_header("authorization", "Bearer test-access-token")
    .match_header("ChatGPT-Account-ID", "test-account-id")
    .with_status(200)
    .with_header("content-type", "text/event-stream")
    .with_body("")
    .create_async()
    .await;

  let envelope = make_envelope(&chat_url, None);
  let client = LibertyCodexClient::from_envelope(&envelope, safe_http());
  let result = client
    .forward_request_with_method(
      &Method::POST,
      "/responses",
      Some(serde_json::json!({"model": "gpt-5.2", "input": []})),
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
async fn forward_returns_not_found_for_unknown_path() -> anyhow::Result<()> {
  let server = Server::new_async().await;
  let chat_url = format!("{}/codex/responses", server.url());

  let envelope = make_envelope(&chat_url, None);
  let client = LibertyCodexClient::from_envelope(&envelope, safe_http());

  let result = client
    .forward_request_with_method(
      &Method::POST,
      "/foo",
      Some(serde_json::json!({"any": "body"})),
      None,
      None,
    )
    .await;

  match result {
    Err(AiApiClientFactoryError::NotFound(path)) => assert_eq!("/foo", path),
    other => panic!(
      "expected NotFound for unknown path, got: {:?}",
      other.is_err()
    ),
  }
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn forward_retries_on_401_with_force_refresh() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/codex/responses", server.url());

  let _first_call = server
    .mock("POST", "/codex/responses")
    .match_header("authorization", "Bearer old-token")
    .with_status(401)
    .with_body(r#"{"error": "unauthorized"}"#)
    .create_async()
    .await;

  let _second_call = server
    .mock("POST", "/codex/responses")
    .match_header("authorization", "Bearer new-token")
    .with_status(200)
    .with_header("content-type", "text/event-stream")
    .with_body("")
    .create_async()
    .await;

  let mut mock_refresh = MockLlmLibertyRefresh::new();
  let fresh_creds = make_creds(&chat_url, None, "new-token");
  mock_refresh
    .expect_force_refresh()
    .times(1)
    .returning(move |_, _, _| Ok(fresh_creds.clone()));

  let creds = make_creds(&chat_url, None, "old-token");
  let client = LibertyCodexClient::from_credentials(
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
      "/responses",
      Some(serde_json::json!({"model": "gpt-5.2", "input": []})),
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
  let chat_url = format!("{}/codex/responses", server.url());

  let _first_call = server
    .mock("POST", "/codex/responses")
    .with_status(401)
    .with_body(r#"{"error": "unauthorized"}"#)
    .create_async()
    .await;

  let _second_call = server
    .mock("POST", "/codex/responses")
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
  let client = LibertyCodexClient::from_credentials(
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
      "/responses",
      Some(serde_json::json!({"model": "gpt-5.2", "input": []})),
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
  let chat_url = format!("{}/codex/responses", server.url());

  let _mock = server
    .mock("POST", "/codex/responses")
    .with_status(401)
    .with_body(r#"{"error": "unauthorized"}"#)
    .create_async()
    .await;

  let envelope = make_envelope(&chat_url, None);
  let client = LibertyCodexClient::from_envelope(&envelope, safe_http());

  let result = client
    .forward_request_with_method(
      &Method::POST,
      "/responses",
      Some(serde_json::json!({"model": "gpt-5.2", "input": []})),
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
  let chat_url = format!("{}/codex/responses", server.url());

  let _mock = server
    .mock("POST", "/codex/responses")
    .with_status(401)
    .create_async()
    .await;

  let mut mock_refresh = MockLlmLibertyRefresh::new();
  mock_refresh
    .expect_force_refresh()
    .times(1)
    .returning(|_, _, _| Err(LlmLibertyRefreshError::NotFound("alias gone".to_string())));

  let creds = make_creds(&chat_url, None, "old-token");
  let client = LibertyCodexClient::from_credentials(
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
      "/responses",
      Some(serde_json::json!({"model": "gpt-5.2", "input": []})),
      None,
      None,
    )
    .await;

  assert!(result.is_err());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn forward_passes_max_output_tokens_through_unchanged() -> anyhow::Result<()> {
  // Proxy must not mutate request bodies; codex quirks belong in the caller.
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/codex/responses", server.url());

  let expected = serde_json::json!({
    "model": "gpt-5.2",
    "input": [],
    "max_output_tokens": 2048,
    "instructions": "You are Codex, OpenAI's coding agent.",
    "store": false,
    "stream": true
  });
  let _mock = server
    .mock("POST", "/codex/responses")
    .match_body(Matcher::Json(expected))
    .with_status(200)
    .with_header("content-type", "text/event-stream")
    .with_body("")
    .create_async()
    .await;

  let envelope = make_envelope(&chat_url, None);
  let client = LibertyCodexClient::from_envelope(&envelope, safe_http());
  let result = client
    .forward_request_with_method(
      &Method::POST,
      "/responses",
      Some(serde_json::json!({"model": "gpt-5.2", "input": [], "max_output_tokens": 2048})),
      None,
      None,
    )
    .await?;

  assert_eq!(200, result.status().as_u16());
  Ok(())
}

#[rstest]
#[case::get("GET", "/responses/resp-abc-123", "/codex/responses/resp-abc-123")]
#[case::delete("DELETE", "/responses/resp-abc-123", "/codex/responses/resp-abc-123")]
#[case::input_items(
  "GET",
  "/responses/resp-abc-123/input_items",
  "/codex/responses/resp-abc-123/input_items"
)]
#[case::cancel(
  "POST",
  "/responses/resp-abc-123/cancel",
  "/codex/responses/resp-abc-123/cancel"
)]
#[anyhow_trace]
#[tokio::test]
async fn forward_responses_crud_paths_append_to_chat_url(
  #[case] method: &str,
  #[case] api_path: &str,
  #[case] expected_upstream_path: &str,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let chat_url = format!("{}/codex/responses", server.url());

  let _mock = server
    .mock(method, expected_upstream_path)
    .match_header("authorization", "Bearer test-access-token")
    .with_status(200)
    .with_body("{}")
    .create_async()
    .await;

  let envelope = make_envelope(&chat_url, None);
  let client = LibertyCodexClient::from_envelope(&envelope, safe_http());
  let http_method = Method::from_bytes(method.as_bytes())?;
  let result = client
    .forward_request_with_method(&http_method, api_path, None, None, None)
    .await?;

  assert_eq!(200, result.status().as_u16());
  Ok(())
}
