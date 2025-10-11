use crate::db::DbService;
use async_trait::async_trait;
use axum::body::Body;
use axum::response::Response;
use derive_new::new;
use objs::{impl_error_from, ApiAlias, AppError, ErrorType, ReqwestError};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const TEST_PROMPT_MAX_LENGTH: usize = 30;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AiApiServiceError {
  #[error(transparent)]
  Reqwest(#[from] ReqwestError),

  #[error("api_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ApiError(String),

  #[error("unauthorized")]
  #[error_meta(error_type = ErrorType::Authentication)]
  Unauthorized(String),

  #[error("not_found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),

  #[error("rate_limit")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  RateLimit(String),

  #[error("prompt_too_long")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  PromptTooLong {
    max_length: usize,
    actual_length: usize,
  },

  #[error("model_not_found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ModelNotFound(String),
}

impl_error_from!(
  reqwest::Error,
  AiApiServiceError::Reqwest,
  ::objs::ReqwestError
);

type Result<T> = std::result::Result<T, AiApiServiceError>;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AiApiService: Send + Sync + std::fmt::Debug {
  /// Test connectivity with a short prompt (max 30 chars for cost control)
  /// API key is optional - if None, requests without authentication (API may return 401)
  async fn test_prompt(
    &self,
    api_key: Option<String>,
    base_url: &str,
    model: &str,
    prompt: &str,
  ) -> Result<String>;

  /// Fetch available models from provider API
  /// API key is optional - if None, requests without authentication (API may return 401)
  async fn fetch_models(&self, api_key: Option<String>, base_url: &str) -> Result<Vec<String>>;

  /// Forward request to remote API
  async fn forward_request(&self, api_path: &str, id: &str, request: Value) -> Result<Response>;
}

#[derive(Debug, Clone, new)]
pub struct DefaultAiApiService {
  client: Client,
  db_service: Arc<dyn DbService>,
}

impl DefaultAiApiService {
  /// Create a new AI API service with default client
  pub fn with_db_service(db_service: Arc<dyn DbService>) -> Self {
    let client = Client::builder()
      .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
      .build()
      .expect("Failed to create HTTP client");

    Self::new(client, db_service)
  }

  /// Get API configuration for an id
  /// API key is optional - returns None if not configured
  async fn get_api_config(&self, id: &str) -> Result<(ApiAlias, Option<String>)> {
    // Get the API model alias configuration
    let api_alias = self
      .db_service
      .get_api_model_alias(id)
      .await
      .map_err(|e| AiApiServiceError::ApiError(e.to_string()))?
      .ok_or_else(|| AiApiServiceError::ModelNotFound(id.to_string()))?;

    // Get the decrypted API key (optional - may not be configured)
    let api_key = self
      .db_service
      .get_api_key_for_alias(id)
      .await
      .map_err(|e| AiApiServiceError::ApiError(e.to_string()))?;

    Ok((api_alias, api_key))
  }

  /// Convert HTTP status to appropriate error
  fn status_to_error(status: reqwest::StatusCode, body: String) -> AiApiServiceError {
    match status {
      reqwest::StatusCode::UNAUTHORIZED => AiApiServiceError::Unauthorized(body),
      reqwest::StatusCode::NOT_FOUND => AiApiServiceError::NotFound(body),
      reqwest::StatusCode::TOO_MANY_REQUESTS => AiApiServiceError::RateLimit(body),
      _ => AiApiServiceError::ApiError(format!("Status {}: {}", status, body)),
    }
  }
}

#[async_trait]
impl AiApiService for DefaultAiApiService {
  async fn test_prompt(
    &self,
    api_key: Option<String>,
    base_url: &str,
    model: &str,
    prompt: &str,
  ) -> Result<String> {
    if prompt.len() > TEST_PROMPT_MAX_LENGTH {
      return Err(AiApiServiceError::PromptTooLong {
        max_length: TEST_PROMPT_MAX_LENGTH,
        actual_length: prompt.len(),
      });
    }

    let request_body = serde_json::json!({
      "model": model,
      "messages": [
        {
          "role": "user",
          "content": prompt
        }
      ],
      "max_tokens": 50,
      "temperature": 0.7
    });

    let url = format!("{}/chat/completions", base_url);

    let mut request = self
      .client
      .post(&url)
      .header("Content-Type", "application/json")
      .json(&request_body);

    // Only add Authorization header if API key is provided
    if let Some(key) = api_key {
      request = request.header("Authorization", format!("Bearer {}", key));
    }

    let response = request.send().await?;

    let status = response.status();
    if !status.is_success() {
      let body = response.text().await.unwrap_or_default();
      return Err(Self::status_to_error(status, body));
    }

    let response_body: serde_json::Value = response.json().await?;

    let content = response_body
      .get("choices")
      .and_then(|c| c.get(0))
      .and_then(|choice| choice.get("message"))
      .and_then(|msg| msg.get("content"))
      .and_then(|c| c.as_str())
      .unwrap_or("No response")
      .to_string();

    Ok(content)
  }

  async fn fetch_models(&self, api_key: Option<String>, base_url: &str) -> Result<Vec<String>> {
    let url = format!("{}/models", base_url);

    let mut request = self.client.get(&url);

    // Only add Authorization header if API key is provided
    if let Some(key) = api_key {
      request = request.header("Authorization", format!("Bearer {}", key));
    }

    let response = request.send().await?;

    let status = response.status();
    if !status.is_success() {
      let body = response.text().await.unwrap_or_default();
      return Err(Self::status_to_error(status, body));
    }

    let response_body: serde_json::Value = response.json().await?;

    let models = response_body
      .get("data")
      .and_then(|data| data.as_array())
      .map(|models| {
        models
          .iter()
          .filter_map(|model| model.get("id").and_then(|id| id.as_str()).map(String::from))
          .collect()
      })
      .unwrap_or_default();

    Ok(models)
  }

  async fn forward_request(
    &self,
    api_path: &str,
    id: &str,
    mut request: Value,
  ) -> Result<Response> {
    let (api_alias, api_key) = self.get_api_config(id).await?;
    let url = format!("{}{}", api_alias.base_url, api_path);

    // Handle prefix stripping if configured
    if let Some(ref prefix) = api_alias.prefix {
      if let Some(model_str) = request.get("model").and_then(|v| v.as_str()) {
        if model_str.starts_with(prefix) {
          let stripped_model = model_str
            .strip_prefix(prefix)
            .unwrap_or(model_str)
            .to_string();
          if let Some(obj) = request.as_object_mut() {
            obj.insert(
              "model".to_string(),
              serde_json::Value::String(stripped_model),
            );
          }
        }
      }
    }

    // Forward the request to the remote API
    let mut http_request = self
      .client
      .post(&url)
      .header("Content-Type", "application/json");

    // Only add Authorization header if API key is provided
    if let Some(key) = api_key {
      http_request = http_request.header("Authorization", format!("Bearer {}", key));
    }

    let response = http_request.json(&request).send().await?;

    let status = response.status();

    // Convert reqwest::Response to axum::Response for streaming support
    let mut builder = Response::builder().status(status.as_u16());

    // Copy headers
    for (key, value) in response.headers() {
      if let Ok(value_str) = value.to_str() {
        builder = builder.header(key.as_str(), value_str);
      }
    }

    // Stream the body
    let body_stream = response.bytes_stream();
    let body = Body::from_stream(body_stream);

    let axum_response = builder
      .body(body)
      .map_err(|e| AiApiServiceError::ApiError(e.to_string()))?;

    Ok(axum_response)
  }
}

#[cfg(test)]
mod tests {
  use crate::ai_api_service::{AiApiService, AiApiServiceError, DefaultAiApiService};
  use crate::db::MockDbService;
  use axum::http::StatusCode;
  use chrono::Utc;
  use mockito::Server;
  use objs::{ApiAlias, ApiFormat};
  use rstest::rstest;
  use serde_json::json;
  use std::sync::Arc;

  #[rstest]
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
    let api_alias = ApiAlias::new(api_id, api_format, &url, models, prefix, Utc::now());

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
      Utc::now(),
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
}
