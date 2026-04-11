use super::{AiApiService, AiApiServiceError, DefaultAiApiService};
use crate::models::{ApiAlias, ApiFormat, ApiModel};
use crate::test_utils::{fixed_dt, openai_model};
use anyhow_trace::anyhow_trace;
use axum::http::{Method, StatusCode};
use mockito::Server;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};

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

  let expected_body = json!({
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
    .fetch_models(Some("test-key".to_string()), &url, &ApiFormat::OpenAI)
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
    )
    .await;

  assert!(matches!(result, Err(AiApiServiceError::NotFound(_))));

  Ok(())
}

#[rstest]
#[case::strips_prefix(
  "azure-openai",
  ApiFormat::OpenAI,
  vec![openai_model("gpt-4")],
  Some("azure/".to_string()),
  "azure/gpt-4",
  "gpt-4"
)]
#[case::no_prefix_unchanged(
  "openai-api",
  ApiFormat::OpenAI,
  vec![openai_model("gpt-4")],
  None,
  "gpt-4",
  "gpt-4"
)]
#[case::strips_nested_prefix(
  "openrouter-api",
  ApiFormat::OpenAI,
  vec![openai_model("openai/gpt-4")],
  Some("openrouter/".to_string()),
  "openrouter/openai/gpt-4",
  "openai/gpt-4"
)]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_chat_completion_model_prefix_handling(
  #[case] api_id: &str,
  #[case] api_format: ApiFormat,
  #[case] models: Vec<ApiModel>,
  #[case] prefix: Option<String>,
  #[case] input_model: &str,
  #[case] expected_model: &str,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let api_alias = ApiAlias::new(api_id, api_format, &url, models, prefix, false, fixed_dt());

  let incoming_request = json! {{
    "model": input_model,
    "messages": [
      {
        "role": "user",
        "content": "Hello"
      }
    ]
  }};
  let fwd_request = json! {{
    "model": expected_model,
    "messages": [
      {
        "role": "user",
        "content": "Hello"
      }
    ]
  }};
  let _mock = server
    .mock("POST", "/chat/completions")
    .match_body(mockito::Matcher::JsonString(serde_json::to_string(
      &fwd_request,
    )?))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"choices":[{"message":{"content":"Hi there!"}}]}"#)
    .create_async()
    .await;
  let service = DefaultAiApiService::new()?;
  let response = service
    .forward_request(
      "/chat/completions",
      &api_alias,
      Some("test-key".to_string()),
      serde_json::from_value(incoming_request)?,
    )
    .await?;
  assert_eq!(response.status(), StatusCode::OK);
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_request_without_api_key() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let api_id = "test-api-no-key";

  let api_alias = ApiAlias::new(
    api_id,
    ApiFormat::OpenAI,
    &url,
    vec![openai_model("gpt-4")],
    None,
    false,
    fixed_dt(),
  );

  let request = json! {{
    "model": "gpt-4",
    "messages": [
      {
        "role": "user",
        "content": "Hello"
      }
    ]
  }};

  let _mock = server
    .mock("POST", "/chat/completions")
    .match_header("content-type", "application/json")
    .match_body(mockito::Matcher::JsonString(serde_json::to_string(
      &request,
    )?))
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"choices":[{"message":{"content":"Response without auth"}}]}"#)
    .create_async()
    .await;

  let service = DefaultAiApiService::new()?;
  let response = service
    .forward_request(
      "/chat/completions",
      &api_alias,
      None,
      serde_json::from_value(request)?,
    )
    .await?;

  assert_eq!(response.status(), StatusCode::OK);
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
    .fetch_models(api_key.map(|s| s.to_string()), &url, &ApiFormat::OpenAI)
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
    .fetch_models(api_key.map(|s| s.to_string()), &url, &ApiFormat::OpenAI)
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
    )
    .await?;

  assert_eq!("No response", result);

  Ok(())
}

// =============================================================================
// forward_request_with_method — GET / DELETE / POST method dispatch
// =============================================================================

fn make_api_alias(url: &str) -> ApiAlias {
  ApiAlias::new(
    "test-api",
    ApiFormat::OpenAI,
    url,
    vec![openai_model("gpt-4")],
    None,
    false,
    fixed_dt(),
  )
}

#[rstest]
#[case::get_no_body(Method::GET, None, None, false)]
#[case::delete_no_body(Method::DELETE, None, None, false)]
#[case::post_with_body(Method::POST, Some(json!({"model": "gpt-4", "messages": []})), None, true)]
#[case::get_with_query_params(
  Method::GET,
  None,
  Some(vec![("after".to_string(), "ts_123".to_string())]),
  false
)]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_request_with_method_dispatch(
  #[case] method: Method,
  #[case] body: Option<Value>,
  #[case] query_params: Option<Vec<(String, String)>>,
  #[case] expect_content_type: bool,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let api_alias = make_api_alias(&url);
  let service = DefaultAiApiService::new()?;

  let path = if query_params.is_some() {
    "/responses?after=ts_123"
  } else {
    "/responses"
  };

  let mut mock = server.mock(method.as_str(), path);

  if expect_content_type {
    mock = mock.match_header("content-type", "application/json");
  }

  let _mock = mock
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"id":"resp-123"}"#)
    .create_async()
    .await;

  let response = service
    .forward_request_with_method(
      &method,
      "/responses",
      &api_alias,
      None,
      body,
      query_params,
      None,
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());

  Ok(())
}

// =============================================================================
// Anthropic format
// =============================================================================

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_anthropic_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let expected_body = json!({
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
    .fetch_models(Some("test-key".to_string()), &url, &ApiFormat::Anthropic)
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
    .fetch_models(Some("test-key".to_string()), &url, &ApiFormat::Anthropic)
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

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_request_with_method_anthropic_headers() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let api_alias = ApiAlias::new(
    "anthropic-api",
    ApiFormat::Anthropic,
    &url,
    vec![openai_model("claude-3-5-sonnet-20241022")],
    None,
    false,
    fixed_dt(),
  );

  let _mock = server
    .mock("POST", "/messages")
    .match_header("x-api-key", "test-key")
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
      Some("test-key".to_string()),
      Some(json!({"model":"claude-3-5-sonnet-20241022","max_tokens":1,"messages":[]})),
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
async fn test_forward_request_with_method_client_headers_forwarded() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let api_alias = ApiAlias::new(
    "anthropic-api",
    ApiFormat::Anthropic,
    &url,
    vec![openai_model("claude-3-5-sonnet-20241022")],
    None,
    false,
    fixed_dt(),
  );

  let _mock = server
    .mock("POST", "/messages")
    .match_header("x-api-key", "test-key")
    .match_header("anthropic-beta", "test-beta-flag")
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
      Some("test-key".to_string()),
      Some(json!({"model":"claude-3-5-sonnet-20241022","max_tokens":1,"messages":[]})),
      None,
      Some(vec![(
        "anthropic-beta".to_string(),
        "test-beta-flag".to_string(),
      )]),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_request_client_anthropic_version_used_not_default() -> anyhow::Result<()> {
  // When client supplies anthropic-version, it must appear exactly once upstream
  // (the default must NOT be injected — reqwest appends, not replaces).
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let api_alias = ApiAlias::new(
    "anthropic-api",
    ApiFormat::Anthropic,
    &url,
    vec![openai_model("claude-3-5-sonnet-20241022")],
    None,
    false,
    fixed_dt(),
  );

  let _mock = server
    .mock("POST", "/messages")
    .match_header("x-api-key", "test-key")
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
      Some("test-key".to_string()),
      Some(json!({"model":"claude-3-5-sonnet-20241022","max_tokens":1,"messages":[]})),
      None,
      Some(vec![(
        "anthropic-version".to_string(),
        "2023-06-01".to_string(),
      )]),
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_request_default_anthropic_version_injected_when_absent() -> anyhow::Result<()>
{
  // When client does not supply anthropic-version, BodhiApp injects the default.
  let mut server = Server::new_async().await;
  let url = server.url();
  let service = DefaultAiApiService::new()?;

  let api_alias = ApiAlias::new(
    "anthropic-api",
    ApiFormat::Anthropic,
    &url,
    vec![openai_model("claude-3-5-sonnet-20241022")],
    None,
    false,
    fixed_dt(),
  );

  let _mock = server
    .mock("POST", "/messages")
    .match_header("x-api-key", "test-key")
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
      Some("test-key".to_string()),
      Some(json!({"model":"claude-3-5-sonnet-20241022","max_tokens":1,"messages":[]})),
      None,
      None, // no client_headers — default must be injected
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  Ok(())
}
