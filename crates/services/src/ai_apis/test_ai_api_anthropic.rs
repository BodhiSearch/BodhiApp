use super::{AiApiService, DefaultAiApiService};
use crate::models::ApiFormat;
use anyhow_trace::anyhow_trace;
use mockito::Server;
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_anthropic_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let expected_body = serde_json::json!({
    "model": "claude-3-5-sonnet-20241022",
    "max_tokens": 50,
    "messages": [{"role": "user", "content": "Hello"}]
  });

  let _mock = server
    .mock("POST", "/messages")
    .match_header("x-api-key", "test-key")
    .match_header("anthropic-version", "2023-06-01")
    .match_body(mockito::Matcher::JsonString(serde_json::to_string(
      &expected_body,
    )?))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"content": [{"type": "text", "text": "Hi there!"}]}"#)
    .create_async()
    .await;

  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "claude-3-5-sonnet-20241022",
      "Hello",
      &ApiFormat::Anthropic,
      None,
      None,
    )
    .await?;
  assert_eq!("Hi there!", result);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_anthropic_malformed_response() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("POST", "/messages")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"content": []}"#)
    .create_async()
    .await;

  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "claude-3-5-sonnet-20241022",
      "Hello",
      &ApiFormat::Anthropic,
      None,
      None,
    )
    .await?;
  assert_eq!("No response", result);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_anthropic_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("GET", "/models")
    .match_header("x-api-key", "test-key")
    .match_header("anthropic-version", "2023-06-01")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{
        "data": [
          {"id": "claude-3-5-sonnet-20241022", "display_name": "Claude 3.5 Sonnet", "created_at": "2024-10-22T00:00:00Z", "type": "model", "capabilities": null, "max_input_tokens": null, "max_tokens": null},
          {"id": "claude-3-opus-20240229", "display_name": "Claude 3 Opus", "created_at": "2024-02-29T00:00:00Z", "type": "model", "capabilities": null, "max_input_tokens": null, "max_tokens": null}
        ],
        "has_more": false
      }"#,
    )
    .create_async()
    .await;

  let models = service
    .fetch_models(
      Some("test-key".to_string()),
      &url,
      &ApiFormat::Anthropic,
      None,
      None,
    )
    .await?;
  let model_ids: Vec<&str> = models.iter().map(|m| m.id()).collect();
  assert_eq!(
    vec!["claude-3-5-sonnet-20241022", "claude-3-opus-20240229"],
    model_ids
  );

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_anthropic_pagination() -> anyhow::Result<()> {
  // Two-page response: first page has has_more=true, second has has_more=false.
  // All IDs across both pages must be returned.
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock_page1 = server
    .mock("GET", "/models")
    .match_header("x-api-key", "test-key")
    .match_header("anthropic-version", "2023-06-01")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{
        "data": [
          {"id": "claude-3-5-sonnet-20241022", "display_name": "Claude 3.5 Sonnet", "created_at": "2024-10-22T00:00:00Z", "type": "model", "capabilities": null, "max_input_tokens": null, "max_tokens": null},
          {"id": "claude-3-opus-20240229", "display_name": "Claude 3 Opus", "created_at": "2024-02-29T00:00:00Z", "type": "model", "capabilities": null, "max_input_tokens": null, "max_tokens": null}
        ],
        "has_more": true
      }"#,
    )
    .create_async()
    .await;

  let _mock_page2 = server
    .mock("GET", "/models?before_id=claude-3-opus-20240229")
    .match_header("x-api-key", "test-key")
    .match_header("anthropic-version", "2023-06-01")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{
        "data": [
          {"id": "claude-3-haiku-20240307", "display_name": "Claude 3 Haiku", "created_at": "2024-03-07T00:00:00Z", "type": "model", "capabilities": null, "max_input_tokens": null, "max_tokens": null}
        ],
        "has_more": false
      }"#,
    )
    .create_async()
    .await;

  let models = service
    .fetch_models(
      Some("test-key".to_string()),
      &url,
      &ApiFormat::Anthropic,
      None,
      None,
    )
    .await?;
  let model_ids: Vec<&str> = models.iter().map(|m| m.id()).collect();
  assert_eq!(
    vec![
      "claude-3-5-sonnet-20241022",
      "claude-3-opus-20240229",
      "claude-3-haiku-20240307"
    ],
    model_ids
  );

  Ok(())
}
