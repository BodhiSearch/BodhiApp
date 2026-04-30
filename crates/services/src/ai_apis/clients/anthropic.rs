use super::anthropic_shared::{extract_anthropic_completion_text, parse_anthropic_models_page};
use crate::ai_apis::ai_api_client::AiApiClient;
use crate::ai_apis::error::{AiApiServiceError, Result};
use crate::ai_apis::provider_shared::forward_to_upstream;
use crate::models::ApiModel;
use crate::SafeReqwest;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

pub(crate) struct AnthropicClient {
  client: SafeReqwest,
  api_key: Option<String>,
  base_url: String,
  prefix: Option<String>,
}

impl AnthropicClient {
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
impl AiApiClient for AnthropicClient {
  async fn test_prompt(&self, model: &str, prompt: &str) -> Result<String> {
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
    Ok(extract_anthropic_completion_text(&body))
  }

  async fn fetch_models(&self) -> Result<Vec<ApiModel>> {
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
      let page_models = parse_anthropic_models_page(&body);
      before_id = page_models.last().map(|m| m.id().to_string());
      all_models.extend(page_models);
      if !body
        .get("has_more")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
      {
        break;
      }
    }
    Ok(all_models)
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
      |rb| self.apply_auth(rb, client_headers.as_deref()),
      client_headers.as_deref(),
    )
    .await
  }
}
