use crate::ai_apis::ai_api_client::AiApiClient;
use crate::ai_apis::error::{AiApiClientFactoryError, Result};
use crate::ai_apis::provider_shared::{fetch_openai_models, forward_to_upstream};
use crate::models::ApiModel;
use crate::SafeReqwest;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

pub(crate) struct OpenAiResponsesClient {
  client: SafeReqwest,
  api_key: Option<String>,
  base_url: String,
  prefix: Option<String>,
}

impl OpenAiResponsesClient {
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
      request = request.header("Authorization", format!("Bearer {}", key));
    }
    request
  }
}

#[async_trait]
impl AiApiClient for OpenAiResponsesClient {
  async fn test_prompt(&self, model: &str, prompt: &str) -> Result<String> {
    let request_body = serde_json::json!({
      "model": model,
      "input": prompt,
      "max_output_tokens": 50,
      "store": false
    });
    let url = format!("{}/responses", self.base_url);
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
    )
  }

  async fn fetch_models(&self) -> Result<Vec<ApiModel>> {
    fetch_openai_models(&self.client, self.api_key.as_deref(), &self.base_url).await
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
