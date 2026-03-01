use super::error::{AiApiServiceError, Result};
use crate::db::DbService;
use crate::models::ApiAlias;
use async_trait::async_trait;
use axum::body::Body;
use axum::response::Response;
use derive_new::new;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const TEST_PROMPT_MAX_LENGTH: usize = 30;

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
#[path = "test_ai_api_service.rs"]
mod test_ai_api_service;
