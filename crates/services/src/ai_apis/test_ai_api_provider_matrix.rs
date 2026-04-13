use super::{AiApiService, DefaultAiApiService};
use crate::models::{ApiAlias, ApiFormat};
use crate::test_utils::fixed_dt;
use anyhow_trace::anyhow_trace;
use axum::http::Method;
use mockito::Server;
use rstest::rstest;
use serde_json::json;

fn openai_chat_response() -> &'static str {
  r#"{"choices": [{"message": {"content": "ok"}, "index": 0, "finish_reason": "stop"}]}"#
}

fn openai_models_response() -> &'static str {
  r#"{"data": [{"id": "gpt-4", "object": "model", "created": 0, "owned_by": "openai"}]}"#
}

fn anthropic_message_response() -> &'static str {
  r#"{"content": [{"type": "text", "text": "ok"}]}"#
}

fn anthropic_models_response() -> &'static str {
  r#"{"data": [{"id": "claude-3", "display_name": "Claude 3", "created_at": "2024-01-01T00:00:00Z", "type": "model"}], "has_more": false}"#
}

fn gemini_generate_response() -> &'static str {
  r#"{"candidates": [{"content": {"parts": [{"text": "ok"}]}}]}"#
}

fn gemini_models_response() -> &'static str {
  r#"{"models": [{"name": "models/gemini-2.5-flash", "version": "001"}]}"#
}

fn echo_response() -> &'static str {
  r#"{"status": "echoed"}"#
}

fn make_alias(url: &str, format: ApiFormat) -> ApiAlias {
  ApiAlias::new(
    "test-alias",
    format,
    url,
    vec![],
    None,
    false,
    fixed_dt(),
    None,
    None,
  )
}

// === test_prompt_success ===

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_prompt_success_openai() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let _mock = server
    .mock("POST", "/chat/completions")
    .match_header("Authorization", "Bearer test-key")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(openai_chat_response())
    .create_async()
    .await;
  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "gpt-4",
      "hi",
      &ApiFormat::OpenAI,
      None,
      None,
    )
    .await?;
  assert_eq!("ok", result);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_prompt_success_openai_responses() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let _mock = server
    .mock("POST", "/responses")
    .match_header("Authorization", "Bearer test-key")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"output": [{"type": "message", "content": [{"text": "ok"}]}]}"#)
    .create_async()
    .await;
  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "gpt-4o",
      "hi",
      &ApiFormat::OpenAIResponses,
      None,
      None,
    )
    .await?;
  assert_eq!("ok", result);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_prompt_success_anthropic() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let _mock = server
    .mock("POST", "/messages")
    .match_header("x-api-key", "test-key")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(anthropic_message_response())
    .create_async()
    .await;
  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "claude-3",
      "hi",
      &ApiFormat::Anthropic,
      None,
      None,
    )
    .await?;
  assert_eq!("ok", result);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_prompt_success_anthropic_oauth() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let _mock = server
    .mock("POST", "/messages")
    .match_header("Authorization", "Bearer test-token")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(anthropic_message_response())
    .create_async()
    .await;
  let result = service
    .test_prompt(
      Some("test-token".to_string()),
      &url,
      "claude-3",
      "hi",
      &ApiFormat::AnthropicOAuth,
      None,
      None,
    )
    .await?;
  assert_eq!("ok", result);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_prompt_success_gemini() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let _mock = server
    .mock("POST", "/models/gemini-2.5-flash:generateContent")
    .match_header("x-goog-api-key", "test-key")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(gemini_generate_response())
    .create_async()
    .await;
  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "gemini-2.5-flash",
      "hi",
      &ApiFormat::Gemini,
      None,
      None,
    )
    .await?;
  assert_eq!("ok", result);
  Ok(())
}

// === test_prompt_401 ===

#[rstest]
#[case::openai(ApiFormat::OpenAI, "/chat/completions", "some-model")]
#[case::openai_responses(ApiFormat::OpenAIResponses, "/responses", "some-model")]
#[case::anthropic(ApiFormat::Anthropic, "/messages", "some-model")]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth, "/messages", "some-model")]
#[case::gemini(
  ApiFormat::Gemini,
  "/models/gemini-2.5-flash:generateContent",
  "gemini-2.5-flash"
)]
#[anyhow_trace]
#[tokio::test]
async fn test_prompt_401_unauthorized(
  #[case] api_format: ApiFormat,
  #[case] path: &str,
  #[case] model: &str,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let _mock = server
    .mock("POST", path)
    .with_status(401)
    .with_header("content-type", "application/json")
    .with_body(r#"{"error": "unauthorized"}"#)
    .create_async()
    .await;
  let err = service
    .test_prompt(
      Some("bad-key".to_string()),
      &url,
      model,
      "hi",
      &api_format,
      None,
      None,
    )
    .await
    .expect_err("should fail with 401");
  let msg = format!("{}", err);
  assert!(
    msg.contains("401") || msg.contains("nauthorized"),
    "expected 401 error, got: {}",
    msg
  );
  Ok(())
}

// === fetch_models_success ===

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_success_openai() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let _mock = server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(openai_models_response())
    .create_async()
    .await;
  let models = service
    .fetch_models(
      Some("key".to_string()),
      &url,
      &ApiFormat::OpenAI,
      None,
      None,
    )
    .await?;
  assert_eq!(1, models.len());
  assert_eq!("gpt-4", models[0].id());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_success_gemini() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let _mock = server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(gemini_models_response())
    .create_async()
    .await;
  let models = service
    .fetch_models(
      Some("key".to_string()),
      &url,
      &ApiFormat::Gemini,
      None,
      None,
    )
    .await?;
  assert_eq!(1, models.len());
  assert_eq!("gemini-2.5-flash", models[0].id());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_success_anthropic() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let _mock = server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(anthropic_models_response())
    .create_async()
    .await;
  let models = service
    .fetch_models(
      Some("key".to_string()),
      &url,
      &ApiFormat::Anthropic,
      None,
      None,
    )
    .await?;
  assert_eq!(1, models.len());
  assert_eq!("claude-3", models[0].id());
  Ok(())
}

// === fetch_models_401 ===

#[rstest]
#[case::openai(ApiFormat::OpenAI)]
#[case::openai_responses(ApiFormat::OpenAIResponses)]
#[case::anthropic(ApiFormat::Anthropic)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth)]
#[case::gemini(ApiFormat::Gemini)]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_401(#[case] api_format: ApiFormat) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let _mock = server
    .mock("GET", "/models")
    .with_status(401)
    .with_header("content-type", "application/json")
    .with_body(r#"{"error": "unauthorized"}"#)
    .create_async()
    .await;
  let err = service
    .fetch_models(Some("bad-key".to_string()), &url, &api_format, None, None)
    .await
    .expect_err("should fail with 401");
  let msg = format!("{}", err);
  assert!(
    msg.contains("401") || msg.contains("nauthorized"),
    "expected 401 error, got: {}",
    msg
  );
  Ok(())
}

// === forward_passthrough ===

#[rstest]
#[case::openai(ApiFormat::OpenAI)]
#[case::openai_responses(ApiFormat::OpenAIResponses)]
#[case::anthropic(ApiFormat::Anthropic)]
#[case::anthropic_oauth(ApiFormat::AnthropicOAuth)]
#[case::gemini(ApiFormat::Gemini)]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_passthrough(#[case] api_format: ApiFormat) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;
  let alias = make_alias(&url, api_format);

  let body = json!({"model": "some-model", "messages": []});
  let _mock = server
    .mock("POST", "/chat/completions")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(echo_response())
    .create_async()
    .await;

  let response = service
    .forward_request_with_method(
      &Method::POST,
      "/chat/completions",
      &alias,
      Some("key".to_string()),
      Some(body),
      None,
      None,
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  Ok(())
}
