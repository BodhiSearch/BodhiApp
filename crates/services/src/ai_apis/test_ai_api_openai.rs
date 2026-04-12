use super::{AiApiService, AiApiServiceError, DefaultAiApiService};
use crate::models::{ApiFormat, ApiModel};
use crate::test_utils::openai_model;
use anyhow_trace::anyhow_trace;
use mockito::Server;
use pretty_assertions::assert_eq;
use rstest::rstest;

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("POST", "/chat/completions")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{
        "choices": [{
          "message": {
            "content": "Hello response"
          }
        }]
      }"#,
    )
    .create_async()
    .await;

  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "gpt-3.5-turbo",
      "Hello",
      &ApiFormat::OpenAI,
      None,
      None,
    )
    .await?;
  assert_eq!("Hello response", result);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_openai_responses_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let expected_body = serde_json::json!({
    "model": "gpt-4o",
    "input": "Hello",
    "max_output_tokens": 50,
    "store": false
  });
  let _mock = server
    .mock("POST", "/responses")
    .match_body(mockito::Matcher::JsonString(serde_json::to_string(
      &expected_body,
    )?))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{
        "output": [{
          "type": "message",
          "content": [{"type": "text", "text": "Hello response"}]
        }]
      }"#,
    )
    .create_async()
    .await;

  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "gpt-4o",
      "Hello",
      &ApiFormat::OpenAIResponses,
      None,
      None,
    )
    .await?;
  assert_eq!("Hello response", result);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_too_long() -> anyhow::Result<()> {
  let server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let long_prompt = "a".repeat(31);
  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "gpt-3.5-turbo",
      &long_prompt,
      &ApiFormat::OpenAI,
      None,
      None,
    )
    .await;

  assert!(matches!(
    result,
    Err(AiApiServiceError::PromptTooLong {
      max_length: 30,
      actual_length: 31
    })
  ));

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{
        "data": [
          {"id": "gpt-3.5-turbo", "object": "model", "created": 0, "owned_by": "openai"},
          {"id": "gpt-4", "object": "model", "created": 0, "owned_by": "openai"},
          {"id": "gpt-4-turbo", "object": "model", "created": 0, "owned_by": "openai"}
        ]
      }"#,
    )
    .create_async()
    .await;

  let models = service
    .fetch_models(
      Some("test-key".to_string()),
      &url,
      &ApiFormat::OpenAI,
      None,
      None,
    )
    .await?;
  assert_eq!(
    vec![
      openai_model("gpt-3.5-turbo"),
      openai_model("gpt-4"),
      openai_model("gpt-4-turbo"),
    ],
    models
  );

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_api_unauthorized_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("POST", "/chat/completions")
    .with_status(401)
    .with_body("Invalid API key")
    .create_async()
    .await;

  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "gpt-3.5-turbo",
      "Hello",
      &ApiFormat::OpenAI,
      None,
      None,
    )
    .await;

  assert!(matches!(result, Err(AiApiServiceError::Unauthorized(_))));

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_model_not_found() -> anyhow::Result<()> {
  let service = DefaultAiApiService::new()?;
  let mut server = Server::new_async().await;
  let url = server.url();

  let _mock = server
    .mock("POST", "/chat/completions")
    .with_status(404)
    .with_body("Model not found")
    .create_async()
    .await;

  let result = service
    .test_prompt(
      Some("invalid-key".to_string()),
      &url,
      "unknown-model",
      "Hello",
      &ApiFormat::OpenAI,
      None,
      None,
    )
    .await;

  assert!(matches!(result, Err(AiApiServiceError::NotFound(_))));

  Ok(())
}

#[rstest]
#[case::with_api_key(
  Some("test-key"),
  r#"{"choices":[{"message":{"content":"Response with auth"}}]}"#,
  "Response with auth"
)]
#[case::without_api_key(
  None,
  r#"{"choices":[{"message":{"content":"Response without auth"}}]}"#,
  "Response without auth"
)]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_success_parameterized(
  #[case] api_key: Option<&str>,
  #[case] response_body: &str,
  #[case] expected_response: &str,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("POST", "/chat/completions")
    .match_header("content-type", "application/json")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(response_body)
    .create_async()
    .await;

  let result = service
    .test_prompt(
      api_key.map(|s| s.to_string()),
      &url,
      "gpt-3.5-turbo",
      "Hello",
      &ApiFormat::OpenAI,
      None,
      None,
    )
    .await?;

  assert_eq!(expected_response, result);

  Ok(())
}

#[rstest]
#[case::with_api_key(Some("bad-key"), 401, "Unauthorized")]
#[case::without_api_key(None, 401, "Unauthorized")]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_failure_parameterized(
  #[case] api_key: Option<&str>,
  #[case] status_code: u16,
  #[case] response_body: &str,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("POST", "/chat/completions")
    .match_header("content-type", "application/json")
    .with_status(status_code as usize)
    .with_body(response_body)
    .create_async()
    .await;

  let result = service
    .test_prompt(
      api_key.map(|s| s.to_string()),
      &url,
      "gpt-3.5-turbo",
      "Hello",
      &ApiFormat::OpenAI,
      None,
      None,
    )
    .await;

  assert!(result.is_err());

  Ok(())
}

#[rstest]
#[case::with_api_key(
  Some("test-key"),
  r#"{"data": [{"id": "gpt-4", "object": "model", "created": 0, "owned_by": "openai"}, {"id": "gpt-3.5-turbo", "object": "model", "created": 0, "owned_by": "openai"}]}"#,
  vec!["gpt-4", "gpt-3.5-turbo"]
)]
#[case::without_api_key(
  None,
  r#"{"data": [{"id": "gpt-4", "object": "model", "created": 0, "owned_by": "openai"}, {"id": "gpt-3.5-turbo", "object": "model", "created": 0, "owned_by": "openai"}]}"#,
  vec!["gpt-4", "gpt-3.5-turbo"]
)]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_success_parameterized(
  #[case] api_key: Option<&str>,
  #[case] response_body: &str,
  #[case] expected_model_ids: Vec<&str>,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(response_body)
    .create_async()
    .await;

  let result = service
    .fetch_models(
      api_key.map(|s| s.to_string()),
      &url,
      &ApiFormat::OpenAI,
      None,
      None,
    )
    .await?;

  let result_ids: Vec<&str> = result.iter().map(|m| m.id()).collect();
  assert_eq!(expected_model_ids, result_ids);

  Ok(())
}

#[rstest]
#[case::with_api_key(Some("bad-key"), 401, "Unauthorized")]
#[case::without_api_key(None, 401, "Unauthorized")]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_failure_parameterized(
  #[case] api_key: Option<&str>,
  #[case] status_code: u16,
  #[case] response_body: &str,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("GET", "/models")
    .with_status(status_code as usize)
    .with_body(response_body)
    .create_async()
    .await;

  let result = service
    .fetch_models(
      api_key.map(|s| s.to_string()),
      &url,
      &ApiFormat::OpenAI,
      None,
      None,
    )
    .await;

  assert!(result.is_err());

  Ok(())
}

// =============================================================================
// test_prompt — OpenAIResponses format error paths
// =============================================================================

#[rstest]
#[case::unauthorized(401, "Unauthorized", AiApiServiceError::Unauthorized("".to_string()))]
#[case::not_found(404, "Not Found", AiApiServiceError::NotFound("".to_string()))]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_openai_responses_errors(
  #[case] status_code: u16,
  #[case] response_body: &str,
  #[case] expected_error: AiApiServiceError,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let _mock = server
    .mock("POST", "/responses")
    .with_status(status_code as usize)
    .with_body(response_body)
    .create_async()
    .await;

  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "gpt-4o",
      "Hello",
      &ApiFormat::OpenAIResponses,
      None,
      None,
    )
    .await;

  assert!(result.is_err());
  let err = result.unwrap_err();
  assert_eq!(
    std::mem::discriminant(&expected_error),
    std::mem::discriminant(&err)
  );

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_openai_responses_malformed_output() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  // output array has no item with type == "message" → falls back to "No response"
  let _mock = server
    .mock("POST", "/responses")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"output": [{"type": "tool_call", "content": []}]}"#)
    .create_async()
    .await;

  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "gpt-4o",
      "Hello",
      &ApiFormat::OpenAIResponses,
      None,
      None,
    )
    .await?;

  assert_eq!("No response", result);

  Ok(())
}

// Suppress unused import warnings — ApiModel is needed for the #[case] vec! literals
#[allow(unused_imports)]
use ApiModel as _;
