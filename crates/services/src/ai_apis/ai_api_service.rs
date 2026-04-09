use super::error::{AiApiServiceError, Result};
use crate::models::{ApiAlias, ApiFormat};
use crate::SafeReqwest;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;
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
    api_format: &ApiFormat,
  ) -> Result<String>;

  /// Fetch available models from provider API
  /// API key is optional - if None, requests without authentication (API may return 401)
  async fn fetch_models(
    &self,
    api_key: Option<String>,
    base_url: &str,
    api_format: &ApiFormat,
  ) -> Result<Vec<String>>;

  /// Forward POST request to remote API using a pre-resolved alias.
  async fn forward_request(
    &self,
    api_path: &str,
    api_alias: &ApiAlias,
    api_key: Option<String>,
    request: Value,
  ) -> Result<Response> {
    self
      .forward_request_with_method(
        &Method::POST,
        api_path,
        api_alias,
        api_key,
        Some(request),
        None,
        None,
      )
      .await
  }

  /// Forward request to remote API with explicit HTTP method.
  async fn forward_request_with_method(
    &self,
    method: &Method,
    api_path: &str,
    api_alias: &ApiAlias,
    api_key: Option<String>,
    request: Option<Value>,
    query_params: Option<Vec<(String, String)>>,
    client_headers: Option<Vec<(String, String)>>,
  ) -> Result<Response>;
}

#[derive(Debug, Clone)]
pub struct DefaultAiApiService {
  client: SafeReqwest,
}

impl DefaultAiApiService {
  /// Create a new AI API service with default HTTP client settings
  pub fn new() -> Result<Self> {
    let client = SafeReqwest::builder()
      .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
      .allow_private_ips()
      .build()?;
    Ok(Self { client })
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
    api_format: &ApiFormat,
  ) -> Result<String> {
    if prompt.len() > TEST_PROMPT_MAX_LENGTH {
      return Err(AiApiServiceError::PromptTooLong {
        max_length: TEST_PROMPT_MAX_LENGTH,
        actual_length: prompt.len(),
      });
    }

    let (request_body, url) = match api_format {
      ApiFormat::OpenAIResponses => (
        serde_json::json!({
          "model": model,
          "input": prompt,
          "max_output_tokens": 50,
          "store": false
        }),
        format!("{}/responses", base_url),
      ),
      ApiFormat::Anthropic => (
        serde_json::json!({
          "model": model,
          "max_tokens": 50,
          "messages": [{"role": "user", "content": prompt}]
        }),
        format!("{}/messages", base_url),
      ),
      _ => (
        serde_json::json!({
          "model": model,
          "messages": [
            {
              "role": "user",
              "content": prompt
            }
          ],
          "max_tokens": 50,
          "temperature": 0.7
        }),
        format!("{}/chat/completions", base_url),
      ),
    };

    let mut request = self
      .client
      .post(&url)?
      .header("Content-Type", "application/json")
      .json(&request_body);

    match api_format {
      ApiFormat::Anthropic => {
        if let Some(key) = api_key {
          request = request.header("x-api-key", key);
        }
        request = request.header("anthropic-version", "2023-06-01");
      }
      _ => {
        if let Some(key) = api_key {
          request = request.header("Authorization", format!("Bearer {}", key));
        }
      }
    }

    let response = request.send().await?;

    let status = response.status();
    if !status.is_success() {
      let body = response.text().await.unwrap_or_default();
      return Err(Self::status_to_error(status, body));
    }

    let response_body: serde_json::Value = response.json().await?;

    let content = match api_format {
      ApiFormat::OpenAIResponses => response_body
        .get("output")
        .and_then(|o| o.as_array())
        .and_then(|items| {
          items
            .iter()
            .find(|item| item.get("type").and_then(|t| t.as_str()) == Some("message"))
        })
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|parts| parts.first())
        .and_then(|part| part.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("No response")
        .to_string(),
      ApiFormat::Anthropic => response_body
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|b| b.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("No response")
        .to_string(),
      _ => response_body
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("No response")
        .to_string(),
    };

    Ok(content)
  }

  async fn fetch_models(
    &self,
    api_key: Option<String>,
    base_url: &str,
    api_format: &ApiFormat,
  ) -> Result<Vec<String>> {
    let mut all_ids: Vec<String> = Vec::new();
    let mut before_id: Option<String> = None;

    loop {
      let url = match &before_id {
        Some(bid) => format!("{}/models?before_id={}", base_url, bid),
        None => format!("{}/models", base_url),
      };

      let mut request = self.client.get(&url)?;

      match api_format {
        ApiFormat::Anthropic => {
          if let Some(ref key) = api_key {
            request = request.header("x-api-key", key);
          }
          request = request.header("anthropic-version", "2023-06-01");
        }
        _ => {
          if let Some(ref key) = api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
          }
        }
      }

      let response = request.send().await?;

      let status = response.status();
      if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(Self::status_to_error(status, body));
      }

      let response_body: serde_json::Value = response.json().await?;

      let page_ids: Vec<String> = response_body
        .get("data")
        .and_then(|data| data.as_array())
        .map(|models| {
          models
            .iter()
            .filter_map(|model| model.get("id").and_then(|id| id.as_str()).map(String::from))
            .collect()
        })
        .unwrap_or_default();

      before_id = page_ids.last().cloned();
      all_ids.extend(page_ids);

      let has_more = response_body
        .get("has_more")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
      if !has_more {
        break;
      }
    }

    Ok(all_ids)
  }

  async fn forward_request_with_method(
    &self,
    method: &Method,
    api_path: &str,
    api_alias: &ApiAlias,
    api_key: Option<String>,
    mut request: Option<Value>,
    query_params: Option<Vec<(String, String)>>,
    client_headers: Option<Vec<(String, String)>>,
  ) -> Result<Response> {
    let url = format!("{}{}", api_alias.base_url, api_path);

    if let Some(ref req) = request {
      if let Some(ref prefix) = api_alias.prefix {
        if let Some(model_str) = req.get("model").and_then(|v| v.as_str()) {
          if model_str.starts_with(prefix) {
            let stripped_model = model_str
              .strip_prefix(prefix)
              .unwrap_or(model_str)
              .to_string();
            if let Some(obj) = request.as_mut().and_then(|r| r.as_object_mut()) {
              obj.insert(
                "model".to_string(),
                serde_json::Value::String(stripped_model),
              );
            }
          }
        }
      }
    }

    let mut http_request = self.client.request(method.clone(), &url)?;
    if method == Method::POST {
      http_request = http_request.header("Content-Type", "application/json");
    }

    if let Some(ref params) = query_params {
      http_request = http_request.query(params);
    }

    match api_alias.api_format {
      ApiFormat::Anthropic => {
        if let Some(key) = api_key {
          http_request = http_request.header("x-api-key", key);
        }
        // Inject default only when client didn't supply their own version;
        // client preference wins to avoid duplicate headers (reqwest appends, not replaces).
        let client_has_version = client_headers
          .as_ref()
          .map(|hdrs| hdrs.iter().any(|(k, _)| k.eq_ignore_ascii_case("anthropic-version")))
          .unwrap_or(false);
        if !client_has_version {
          http_request = http_request.header("anthropic-version", "2023-06-01");
        }
      }
      _ => {
        if let Some(key) = api_key {
          http_request = http_request.header("Authorization", format!("Bearer {}", key));
        }
      }
    }

    if let Some(hdrs) = client_headers {
      for (k, v) in hdrs {
        http_request = http_request.header(k, v);
      }
    }

    let response = if let Some(body) = request {
      http_request.json(&body).send().await?
    } else {
      http_request.send().await?
    };

    let status = response.status();
    let mut builder = Response::builder().status(status.as_u16());

    for (key, value) in response.headers() {
      if let Ok(value_str) = value.to_str() {
        builder = builder.header(key.as_str(), value_str);
      }
    }

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
