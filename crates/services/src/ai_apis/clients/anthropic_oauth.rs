use super::anthropic_shared::{
  apply_bearer_auth_and_version, extract_anthropic_completion_text, parse_anthropic_models_page,
};
use crate::ai_apis::ai_api_client::AiApiClient;
use crate::ai_apis::error::{AiApiClientFactoryError, Result};
use crate::ai_apis::provider_shared::{forward_to_upstream, merge_extra_body};
use crate::models::ApiModel;
use crate::SafeReqwest;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

pub(crate) struct AnthropicOauthClient {
  client: SafeReqwest,
  api_key: Option<String>,
  base_url: String,
  prefix: Option<String>,
  extra_headers: Option<Value>,
  extra_body: Option<Value>,
}

impl AnthropicOauthClient {
  pub(crate) fn new(
    api_key: Option<String>,
    base_url: String,
    prefix: Option<String>,
    extra_headers: Option<Value>,
    extra_body: Option<Value>,
    client: SafeReqwest,
  ) -> Self {
    Self {
      client,
      api_key,
      base_url,
      prefix,
      extra_headers,
      extra_body,
    }
  }

  fn apply_auth(
    &self,
    request: reqwest::RequestBuilder,
    client_headers: Option<&[(String, String)]>,
  ) -> reqwest::RequestBuilder {
    apply_bearer_auth_and_version(
      request,
      self.api_key.as_deref(),
      self.extra_headers.as_ref(),
      client_headers,
    )
  }
}

#[async_trait]
impl AiApiClient for AnthropicOauthClient {
  async fn test_prompt(&self, model: &str, prompt: &str) -> Result<String> {
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
      return Err(AiApiClientFactoryError::status_to_error(status, body));
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
        return Err(AiApiClientFactoryError::status_to_error(status, body));
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
    let is_messages = api_path == "/messages";
    let merged_request = match (request, &self.extra_body) {
      (Some(body), Some(extra)) if is_messages => Some(merge_extra_body(body, extra)),
      (req, _) => req,
    };
    forward_to_upstream(
      &self.client,
      &self.base_url,
      method,
      api_path,
      self.prefix.as_deref(),
      merged_request,
      query_params.as_deref(),
      |rb| self.apply_auth(rb, client_headers.as_deref()),
      client_headers.as_deref(),
    )
    .await
  }
}
