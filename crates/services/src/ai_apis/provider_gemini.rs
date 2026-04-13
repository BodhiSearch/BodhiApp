use super::ai_provider_client::AIProviderClient;
use super::error::{AiApiServiceError, Result};
use super::provider_shared::forward_to_upstream;
use crate::models::{ApiModel, GeminiModel};
use crate::SafeReqwest;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

pub struct GeminiProviderClient {
  client: SafeReqwest,
  api_key: Option<String>,
  base_url: String,
}

impl GeminiProviderClient {
  pub fn new(api_key: Option<String>, base_url: String, client: SafeReqwest) -> Self {
    Self {
      client,
      api_key,
      base_url,
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
impl AIProviderClient for GeminiProviderClient {
  type CompletionResponse = String;

  async fn models(&self) -> Result<Vec<ApiModel>> {
    let url = format!("{}/models", self.base_url);
    let mut request = self.client.get(&url)?;
    request = self.apply_auth(request);

    let response = request.send().await?;
    let status = response.status();
    if !status.is_success() {
      let body = response.text().await.unwrap_or_default();
      return Err(AiApiServiceError::status_to_error(status, body));
    }

    let body: Value = response.json().await?;

    let models: Vec<ApiModel> = body
      .get("models")
      .and_then(|d| d.as_array())
      .map(|arr| {
        arr
          .iter()
          .filter_map(|v| serde_json::from_value::<GeminiModel>(v.clone()).ok())
          .map(ApiModel::Gemini)
          .collect()
      })
      .unwrap_or_default();

    Ok(models)
  }

  async fn test_connection(&self, model: &str, prompt: &str) -> Result<String> {
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
      return Err(AiApiServiceError::status_to_error(status, body));
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

  async fn forward(
    &self,
    method: &Method,
    api_path: &str,
    prefix: Option<&str>,
    request: Option<Value>,
    query_params: Option<&[(String, String)]>,
    client_headers: Option<&[(String, String)]>,
  ) -> Result<Response> {
    forward_to_upstream(
      &self.client,
      &self.base_url,
      method,
      api_path,
      prefix,
      request,
      query_params,
      |rb| self.apply_auth(rb),
      client_headers,
    )
    .await
  }
}
