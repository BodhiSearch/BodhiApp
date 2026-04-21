use super::{AiApiService, DefaultAiApiService};
use crate::models::{ApiAlias, ApiFormat};
use crate::test_utils::{fixed_dt, openai_model};
use anyhow_trace::anyhow_trace;
use axum::http::Method;
use mockito::Server;
use rstest::rstest;
use serde_json::{json, Value};

fn make_api_alias(url: &str) -> ApiAlias {
  ApiAlias::new(
    "test-api",
    ApiFormat::OpenAI,
    url,
    vec![openai_model("gpt-4")],
    None,
    false,
    fixed_dt(),
    None,
    None,
  )
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
  #[case] models: Vec<crate::models::ApiModel>,
  #[case] prefix: Option<String>,
  #[case] input_model: &str,
  #[case] expected_model: &str,
) -> anyhow::Result<()> {
  let mut server = Server::new_async().await;
  let url = server.url();

  let api_alias = ApiAlias::new(
    api_id,
    api_format,
    &url,
    models,
    prefix,
    false,
    fixed_dt(),
    None,
    None,
  );

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
  assert_eq!(response.status(), axum::http::StatusCode::OK);
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
    None,
    None,
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

  assert_eq!(response.status(), axum::http::StatusCode::OK);
  Ok(())
}

// =============================================================================
// forward_request_with_method — GET / DELETE / POST method dispatch
// =============================================================================

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
    vec![openai_model("claude-sonnet-4-5-20250929")],
    None,
    false,
    fixed_dt(),
    None,
    None,
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
      Some(json!({"model":"claude-sonnet-4-5-20250929","max_tokens":1,"messages":[]})),
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
    vec![openai_model("claude-sonnet-4-5-20250929")],
    None,
    false,
    fixed_dt(),
    None,
    None,
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
      Some(json!({"model":"claude-sonnet-4-5-20250929","max_tokens":1,"messages":[]})),
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
    vec![openai_model("claude-sonnet-4-5-20250929")],
    None,
    false,
    fixed_dt(),
    None,
    None,
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
      Some(json!({"model":"claude-sonnet-4-5-20250929","max_tokens":1,"messages":[]})),
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
    vec![openai_model("claude-sonnet-4-5-20250929")],
    None,
    false,
    fixed_dt(),
    None,
    None,
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
      Some(json!({"model":"claude-sonnet-4-5-20250929","max_tokens":1,"messages":[]})),
      None,
      None, // no client_headers — default must be injected
    )
    .await?;

  assert_eq!(axum::http::StatusCode::OK, response.status());
  Ok(())
}
