use super::ai_provider_client::AIProviderClient;
use super::error::{AiApiServiceError, Result};
use super::provider_shared::forward_to_upstream;
use crate::models::{AnthropicModel, ApiModel};
use crate::SafeReqwest;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

pub struct AnthropicProviderClient {
  client: SafeReqwest,
  api_key: Option<String>,
  base_url: String,
}

impl AnthropicProviderClient {
  pub fn new(api_key: Option<String>, base_url: String, client: SafeReqwest) -> Self {
    Self {
      client,
      api_key,
      base_url,
    }
  }

  fn apply_auth(
    &self,
    mut request: reqwest::RequestBuilder,
    client_headers: Option<&[(String, String)]>,
  ) -> reqwest::RequestBuilder {
    if let Some(ref key) = self.api_key {
      request = request.header("x-api-key", key);
    }
    let client_has_version = client_headers
      .map(|hdrs| {
        hdrs
          .iter()
          .any(|(k, _)| k.eq_ignore_ascii_case("anthropic-version"))
      })
      .unwrap_or(false);
    if !client_has_version {
      request = request.header("anthropic-version", "2023-06-01");
    }
    request
  }
}

#[async_trait]
impl AIProviderClient for AnthropicProviderClient {
  type CompletionResponse = String;

  async fn models(&self) -> Result<Vec<ApiModel>> {
    let mut all_models: Vec<ApiModel> = Vec::new();
    let mut before_id: Option<String> = None;

    loop {
      let url = match &before_id {
        Some(bid) => format!("{}/models?before_id={}", self.base_url, bid),
        None => format!("{}/models", self.base_url),
      };

      let mut request = self.client.get(&url)?;
      request = self.apply_auth(request, None);

      let response = request.send().await?;
      let status = response.status();
      if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AiApiServiceError::status_to_error(status, body));
      }

      let body: Value = response.json().await?;

      let page_models: Vec<ApiModel> = body
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
          arr
            .iter()
            .filter_map(|v| serde_json::from_value::<AnthropicModel>(v.clone()).ok())
            .map(ApiModel::Anthropic)
            .collect()
        })
        .unwrap_or_default();

      before_id = page_models.last().map(|m| m.id().to_string());
      all_models.extend(page_models);

      let has_more = body
        .get("has_more")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
      if !has_more {
        break;
      }
    }

    Ok(all_models)
  }

  async fn test_connection(&self, model: &str, prompt: &str) -> Result<String> {
    let request_body = serde_json::json!({
      "model": model,
      "max_tokens": 50,
      "messages": [{"role": "user", "content": prompt}]
    });
    let url = format!("{}/messages", self.base_url);

    let mut request = self
      .client
      .post(&url)?
      .header("Content-Type", "application/json")
      .json(&request_body);
    request = self.apply_auth(request, None);

    let response = request.send().await?;
    let status = response.status();
    if !status.is_success() {
      let body = response.text().await.unwrap_or_default();
      return Err(AiApiServiceError::status_to_error(status, body));
    }

    let body: Value = response.json().await?;
    Ok(
      body
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|b| b.get("text"))
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
      |rb| self.apply_auth(rb, client_headers),
      client_headers,
    )
    .await
  }
}
