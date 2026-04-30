use crate::ai_apis::ai_api_client::AiApiClient;
use crate::ai_apis::error::{AiApiClientFactoryError, Result};
use crate::ai_apis::provider_shared::forward_to_upstream;
use crate::models::{ApiModel, GeminiModel};
use crate::SafeReqwest;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

pub(crate) struct GeminiClient {
  client: SafeReqwest,
  api_key: Option<String>,
  base_url: String,
  prefix: Option<String>,
}

impl GeminiClient {
  pub(crate) fn new(
    api_key: Option<String>,
    base_url: String,
    prefix: Option<String>,
    client: SafeReqwest,
  ) -> Self {
    Self {
      client,
      api_key,
      base_url,
      prefix,
    }
  }

  fn apply_auth(&self, mut request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    if let Some(ref key) = self.api_key {
      request = request.header("x-goog-api-key", key);
    }
    request
  }
}

#[async_trait]
impl AiApiClient for GeminiClient {
  async fn test_prompt(&self, model: &str, prompt: &str) -> Result<String> {
    let request_body = serde_json::json!({
      "contents": [{"role": "user", "parts": [{"text": prompt}]}]
    });
    let url = format!("{}/models/{}:generateContent", self.base_url, model);
    let mut request = self
      .client
      .post(&url)?
      .header("Content-Type", "application/json")
      .json(&request_body);
    request = self.apply_auth(request);
    let response = request.send().await?;
    let status = response.status();
    if !status.is_success() {
      let body = response.text().await.unwrap_or_default();
      return Err(AiApiClientFactoryError::status_to_error(status, body));
    }
    let body: Value = response.json().await?;
    Ok(
      body
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|cand| cand.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(|parts| parts.as_array())
        .and_then(|arr| arr.first())
        .and_then(|part| part.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("No response")
        .to_string(),
    )
  }

  async fn fetch_models(&self) -> Result<Vec<ApiModel>> {
    let url = format!("{}/models", self.base_url);
    let mut request = self.client.get(&url)?;
    request = self.apply_auth(request);
    let response = request.send().await?;
    let status = response.status();
    if !status.is_success() {
      let body = response.text().await.unwrap_or_default();
      return Err(AiApiClientFactoryError::status_to_error(status, body));
    }
    let body: Value = response.json().await?;
    Ok(
      body
        .get("models")
        .and_then(|d| d.as_array())
        .map(|arr| {
          arr
            .iter()
            .filter_map(|v| serde_json::from_value::<GeminiModel>(v.clone()).ok())
            .map(ApiModel::Gemini)
            .collect()
        })
        .unwrap_or_default(),
    )
  }

  async fn forward_request_with_method(
    &self,
    method: &Method,
    api_path: &str,
    request: Option<Value>,
    query_params: Option<Vec<(String, String)>>,
    client_headers: Option<Vec<(String, String)>>,
  ) -> Result<Response> {
    forward_to_upstream(
      &self.client,
      &self.base_url,
      method,
      api_path,
      self.prefix.as_deref(),
      request,
      query_params.as_deref(),
      |rb| self.apply_auth(rb),
      client_headers.as_deref(),
    )
    .await
  }
}
