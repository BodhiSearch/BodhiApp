use crate::db::DbService;
use async_openai::types::CreateChatCompletionRequest;
use async_trait::async_trait;
use axum::response::Response;
use derive_new::new;
use objs::{impl_error_from, ApiAlias, AppError, ErrorType, ReqwestError};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const TEST_PROMPT_MAX_LENGTH: usize = 30;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AiApiServiceError {
  #[error(transparent)]
  Reqwest(#[from] ReqwestError),

  #[error("ai_api_service_api_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ApiError(String),

  #[error("ai_api_service_unauthorized")]
  #[error_meta(error_type = ErrorType::Authentication)]
  Unauthorized(String),

  #[error("ai_api_service_not_found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),

  #[error("ai_api_service_rate_limit")]
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

  #[error("api_key_not_found")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ApiKeyNotFound(String),
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
  async fn test_prompt(
    &self,
    api_key: &str,
    base_url: &str,
    model: &str,
    prompt: &str,
  ) -> Result<String>;

  /// Fetch available models from provider API
  async fn fetch_models(&self, api_key: &str, base_url: &str) -> Result<Vec<String>>;

  /// Forward chat completion request to remote API
  async fn forward_chat_completion(
    &self,
    id: &str,
    request: CreateChatCompletionRequest,
  ) -> Result<Response>;
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
  async fn get_api_config(&self, id: &str) -> Result<(ApiAlias, String)> {
    // Get the API model alias configuration
    let api_alias = self
      .db_service
      .get_api_model_alias(id)
      .await
      .map_err(|e| AiApiServiceError::ApiError(e.to_string()))?
      .ok_or_else(|| AiApiServiceError::ModelNotFound(id.to_string()))?;

    // Get the decrypted API key
    let api_key = self
      .db_service
      .get_api_key_for_alias(id)
      .await
      .map_err(|e| AiApiServiceError::ApiError(e.to_string()))?
      .ok_or_else(|| AiApiServiceError::ApiKeyNotFound(id.to_string()))?;

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
    api_key: &str,
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

    let response = self
      .client
      .post(&url)
      .header("Authorization", format!("Bearer {}", api_key))
      .header("Content-Type", "application/json")
      .json(&request_body)
      .send()
      .await?;

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

  async fn fetch_models(&self, api_key: &str, base_url: &str) -> Result<Vec<String>> {
    let url = format!("{}/models", base_url);

    let response = self
      .client
      .get(&url)
      .header("Authorization", format!("Bearer {}", api_key))
      .send()
      .await?;

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

  async fn forward_chat_completion(
    &self,
    id: &str,
    request: CreateChatCompletionRequest,
  ) -> Result<Response> {
    let (api_config, api_key) = self.get_api_config(id).await?;

    let url = format!("{}/chat/completions", api_config.base_url);

    // Forward the request to the remote API
    let response = self
      .client
      .post(&url)
      .header("Authorization", format!("Bearer {}", api_key))
      .header("Content-Type", "application/json")
      .json(&request)
      .send()
      .await?;

    let status = response.status();

    // Convert reqwest::Response to axum::Response for streaming support
    let mut builder = axum::response::Response::builder().status(status.as_u16());

    // Copy headers
    for (key, value) in response.headers() {
      if let Ok(value_str) = value.to_str() {
        builder = builder.header(key.as_str(), value_str);
      }
    }

    // Stream the body
    let body_stream = response.bytes_stream();
    let body = axum::body::Body::from_stream(body_stream);

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
  use mockito::Server;
  use rstest::rstest;
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
      .test_prompt("test-key", &url, "gpt-3.5-turbo", "Hello")
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
      .test_prompt("test-key", &url, "gpt-3.5-turbo", &long_prompt)
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

    let models = service.fetch_models("test-key", &url).await?;
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
      .test_prompt("test-key", &url, "gpt-3.5-turbo", "Hello")
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
      .test_prompt("invalid-key", &url, "unknown-model", "Hello")
      .await;

    assert!(matches!(result, Err(AiApiServiceError::NotFound(_))));

    Ok(())
  }
}
