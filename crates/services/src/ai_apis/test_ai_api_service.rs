use super::{AiApiService, AiApiServiceError, DefaultAiApiService};
use crate::models::{ApiAlias, ApiFormat};
use crate::test_utils::{fixed_dt, MockDbService};
use anyhow_trace::anyhow_trace;
use axum::http::StatusCode;
use mockito::Server;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;
use std::sync::Arc;

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_test_prompt_success() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let mock_db = MockDbService::new();
  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));

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
    .test_prompt(Some("test-key".to_string()), &url, "gpt-3.5-turbo", "Hello")
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
  let mock_db = MockDbService::new();
  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));

  let long_prompt = "a".repeat(31);
  let result = service
    .test_prompt(
      Some("test-key".to_string()),
      &url,
      "gpt-3.5-turbo",
      &long_prompt,
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
  let mock_db = MockDbService::new();
  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));

  let _mock = server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(
      r#"{
        "data": [
          {"id": "gpt-3.5-turbo"},
          {"id": "gpt-4"},
          {"id": "gpt-4-turbo"}
        ]
      }"#,
    )
    .create_async()
    .await;

  let models = service
    .fetch_models(Some("test-key".to_string()), &url)
    .await?;
  assert_eq!(vec!["gpt-3.5-turbo", "gpt-4", "gpt-4-turbo"], models);

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_api_unauthorized_error() -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let mock_db = MockDbService::new();
  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));

  let _mock = server
    .mock("POST", "/chat/completions")
    .with_status(401)
    .with_body("Invalid API key")
    .create_async()
    .await;

  let result = service
    .test_prompt(Some("test-key".to_string()), &url, "gpt-3.5-turbo", "Hello")
    .await;

  assert!(matches!(result, Err(AiApiServiceError::Unauthorized(_))));

  Ok(())
}

#[rstest]
#[anyhow_trace]
#[tokio::test]
async fn test_model_not_found() -> anyhow::Result<()> {
  let mock_db = MockDbService::new();
  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));
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
    )
    .await;

  assert!(matches!(result, Err(AiApiServiceError::NotFound(_))));

  Ok(())
}

#[rstest]
#[case::strips_prefix(
  "azure-openai",
  ApiFormat::OpenAI,
  vec!["gpt-4".to_string()],
  Some("azure/".to_string()),
  "azure/gpt-4",
  "gpt-4"
)]
#[case::no_prefix_unchanged(
  "openai-api",
  ApiFormat::OpenAI,
  vec!["gpt-4".to_string()],
  None,
  "gpt-4",
  "gpt-4"
)]
#[case::strips_nested_prefix(
  "openrouter-api",
  ApiFormat::OpenAI,
  vec!["openai/gpt-4".to_string()],
  Some("openrouter/".to_string()),
  "openrouter/openai/gpt-4",
  "openai/gpt-4"
)]
#[anyhow_trace]
#[tokio::test]
async fn test_forward_chat_completion_model_prefix_handling(
  #[case] api_id: &str,
  #[case] api_format: ApiFormat,
  #[case] models: Vec<String>,
  #[case] prefix: Option<String>,
  #[case] input_model: &str,
  #[case] expected_model: &str,
) -> anyhow::Result<()> {
  let mut mock_db = MockDbService::new();
  let mut server = Server::new_async().await;
  let url = server.url();

  // Create API alias with the provided parameters
  let api_alias = ApiAlias::new(api_id, api_format, &url, models, prefix, false, fixed_dt());

  // Setup mock expectations
  let api_id_owned = api_id.to_string();
  mock_db
    .expect_get_api_model_alias()
    .with(mockall::predicate::eq(api_id_owned.clone()))
    .returning(move |_| Ok(Some(api_alias.clone())));

  mock_db
    .expect_get_api_key_for_alias()
    .with(mockall::predicate::eq(api_id_owned))
    .returning(|_| Ok(Some("test-key".to_string())));

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
  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));
  let response = service
    .forward_request(
      "/chat/completions",
      api_id,
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
  let mut mock_db = MockDbService::new();
  let mut server = Server::new_async().await;
  let url = server.url();
  let api_id = "test-api-no-key";

  // Create API alias without API key
  let api_alias = ApiAlias::new(
    api_id,
    ApiFormat::OpenAI,
    &url,
    vec!["gpt-4".to_string()],
    None,
    false,
    fixed_dt(),
  );

  // Setup mock expectations - no API key
  let api_id_owned = api_id.to_string();
  mock_db
    .expect_get_api_model_alias()
    .with(mockall::predicate::eq(api_id_owned.clone()))
    .returning(move |_| Ok(Some(api_alias.clone())));

  mock_db
    .expect_get_api_key_for_alias()
    .with(mockall::predicate::eq(api_id_owned))
    .returning(|_| Ok(None)); // No API key configured

  let request = json! {{
    "model": "gpt-4",
    "messages": [
      {
        "role": "user",
        "content": "Hello"
      }
    ]
  }};

  // Mock server expects request WITHOUT Authorization header
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

  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));
  let response = service
    .forward_request(
      "/chat/completions",
      api_id,
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
  let mock_db = MockDbService::new();
  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));

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
  let mock_db = MockDbService::new();
  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));

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
    )
    .await;

  assert!(result.is_err());

  Ok(())
}

#[rstest]
#[case::with_api_key(Some("test-key"), r#"{"data": [{"id": "gpt-4"}, {"id": "gpt-3.5-turbo"}]}"#, vec!["gpt-4", "gpt-3.5-turbo"])]
#[case::without_api_key(None, r#"{"data": [{"id": "gpt-4"}, {"id": "gpt-3.5-turbo"}]}"#, vec!["gpt-4", "gpt-3.5-turbo"])]
#[anyhow_trace]
#[tokio::test]
async fn test_fetch_models_success_parameterized(
  #[case] api_key: Option<&str>,
  #[case] response_body: &str,
  #[case] expected_models: Vec<&str>,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();
  let mock_db = MockDbService::new();
  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));

  let _mock = server
    .mock("GET", "/models")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(response_body)
    .create_async()
    .await;

  let result = service
    .fetch_models(api_key.map(|s| s.to_string()), &url)
    .await?;

  assert_eq!(
    expected_models
      .iter()
      .map(|s| s.to_string())
      .collect::<Vec<String>>(),
    result
  );

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
  let mock_db = MockDbService::new();
  let service = DefaultAiApiService::with_db_service(Arc::new(mock_db));

  let _mock = server
    .mock("GET", "/models")
    .with_status(status_code as usize)
    .with_body(response_body)
    .create_async()
    .await;

  let result = service
    .fetch_models(api_key.map(|s| s.to_string()), &url)
    .await;

  assert!(result.is_err());

  Ok(())
}
