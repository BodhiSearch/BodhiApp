use super::ai_provider_client::AIProviderClient;
use super::error::{AiApiServiceError, Result};
use super::provider_shared::{forward_to_upstream, merge_extra_body};
use crate::models::{AnthropicModel, ApiModel};
use crate::SafeReqwest;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

pub struct AnthropicOAuthProviderClient {
  client: SafeReqwest,
  api_key: Option<String>,
  base_url: String,
  extra_headers: Option<Value>,
  extra_body: Option<Value>,
}

impl AnthropicOAuthProviderClient {
  pub fn new(
    api_key: Option<String>,
    base_url: String,
    client: SafeReqwest,
    extra_headers: Option<Value>,
    extra_body: Option<Value>,
  ) -> Self {
    Self {
      client,
      api_key,
      base_url,
      extra_headers,
      extra_body,
    }
  }

  fn apply_auth(
    &self,
    mut request: reqwest::RequestBuilder,
    client_headers: Option<&[(String, String)]>,
  ) -> reqwest::RequestBuilder {
    if let Some(ref token) = self.api_key {
      request = request.header("Authorization", format!("Bearer {}", token));
    }

    if let Some(Value::Object(ref map)) = self.extra_headers {
      for (k, v) in map {
        if let Some(v_str) = v.as_str() {
          request = request.header(k.as_str(), v_str);
        }
      }
    }

    let client_has_version = client_headers
      .map(|hdrs| {
        hdrs
          .iter()
          .any(|(k, _)| k.eq_ignore_ascii_case("anthropic-version"))
      })
      .unwrap_or(false);
    let extra_has_version = self
      .extra_headers
      .as_ref()
      .and_then(|v| v.as_object())
      .map(|m| {
        m.keys()
          .any(|k| k.eq_ignore_ascii_case("anthropic-version"))
      })
      .unwrap_or(false);
    if !client_has_version && !extra_has_version {
      request = request.header("anthropic-version", "2023-06-01");
    }

    request
  }
}

#[async_trait]
impl AIProviderClient for AnthropicOAuthProviderClient {
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
    let base_body = serde_json::json!({
      "model": model,
      "max_tokens": 50,
      "messages": [{"role": "user", "content": prompt}]
    });

    let request_body = match &self.extra_body {
      Some(extra) => merge_extra_body(base_body, extra),
      None => base_body,
    };

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
    // Only merge extra_body for /messages endpoint — extra_body contains Anthropic-specific
    // fields (system prompt array, max_tokens) that are invalid in /chat/completions format.
    let is_messages_endpoint = api_path == "/messages";
    let merged_request = match (request, &self.extra_body) {
      (Some(body), Some(extra)) if is_messages_endpoint => Some(merge_extra_body(body, extra)),
      (req, _) => req,
    };

    forward_to_upstream(
      &self.client,
      &self.base_url,
      method,
      api_path,
      prefix,
      merged_request,
      query_params,
      |rb| self.apply_auth(rb, client_headers),
      client_headers,
    )
    .await
  }
}
