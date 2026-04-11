use super::error::{AiApiServiceError, Result};
use crate::models::{AnthropicModel, ApiModel};
use crate::SafeReqwest;
use async_openai::types::models::Model as OpenAIModel;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

/// Strategy trait for provider-specific API interactions.
///
/// The `CompletionResponse` associated type is intentional — it enables future providers
/// to return structured completion responses (e.g., parsed tool calls, streaming chunks)
/// rather than raw strings, without breaking existing implementations.
#[async_trait]
pub trait AIProviderClient: Send + Sync {
  type CompletionResponse: Send;

  async fn models(&self) -> Result<Vec<ApiModel>>;
  async fn test_connection(
    &self,
    model: &str,
    prompt: &str,
  ) -> Result<Self::CompletionResponse>;
  async fn forward(
    &self,
    method: &Method,
    api_path: &str,
    prefix: Option<&str>,
    request: Option<Value>,
    query_params: Option<&[(String, String)]>,
    client_headers: Option<&[(String, String)]>,
  ) -> Result<Response>;
}

// ---------------------------------------------------------------------------
// OpenAI
// ---------------------------------------------------------------------------

pub struct OpenAIProviderClient {
  client: SafeReqwest,
  api_key: Option<String>,
  base_url: String,
}

impl OpenAIProviderClient {
  pub fn new(api_key: Option<String>, base_url: String, client: SafeReqwest) -> Self {
    Self {
      client,
      api_key,
      base_url,
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
impl AIProviderClient for OpenAIProviderClient {
  type CompletionResponse = String;

  async fn models(&self) -> Result<Vec<ApiModel>> {
    fetch_openai_models(&self.client, self.api_key.as_deref(), &self.base_url).await
  }

  async fn test_connection(&self, model: &str, prompt: &str) -> Result<String> {
    let request_body = serde_json::json!({
      "model": model,
      "messages": [{"role": "user", "content": prompt}],
      "max_tokens": 50,
      "temperature": 0.7
    });
    let url = format!("{}/chat/completions", self.base_url);

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
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|msg| msg.get("content"))
        .and_then(|c| c.as_str())
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

// ---------------------------------------------------------------------------
// OpenAI Responses
// ---------------------------------------------------------------------------

pub struct OpenAIResponsesProviderClient {
  client: SafeReqwest,
  api_key: Option<String>,
  base_url: String,
}

impl OpenAIResponsesProviderClient {
  pub fn new(api_key: Option<String>, base_url: String, client: SafeReqwest) -> Self {
    Self {
      client,
      api_key,
      base_url,
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
impl AIProviderClient for OpenAIResponsesProviderClient {
  type CompletionResponse = String;

  async fn models(&self) -> Result<Vec<ApiModel>> {
    fetch_openai_models(&self.client, self.api_key.as_deref(), &self.base_url).await
  }

  async fn test_connection(&self, model: &str, prompt: &str) -> Result<String> {
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
      return Err(AiApiServiceError::status_to_error(status, body));
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

// ---------------------------------------------------------------------------
// Anthropic
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

async fn fetch_openai_models(
  client: &SafeReqwest,
  api_key: Option<&str>,
  base_url: &str,
) -> Result<Vec<ApiModel>> {
  let url = format!("{}/models", base_url);
  let mut request = client.get(&url)?;

  if let Some(key) = api_key {
    request = request.header("Authorization", format!("Bearer {}", key));
  }

  let response = request.send().await?;
  let status = response.status();
  if !status.is_success() {
    let body = response.text().await.unwrap_or_default();
    return Err(AiApiServiceError::status_to_error(status, body));
  }

  let body: Value = response.json().await?;
  let models: Vec<ApiModel> = body
    .get("data")
    .and_then(|d| d.as_array())
    .map(|arr| {
      arr
        .iter()
        .filter_map(|v| serde_json::from_value::<OpenAIModel>(v.clone()).ok())
        .map(ApiModel::OpenAI)
        .collect()
    })
    .unwrap_or_default();

  Ok(models)
}

async fn forward_to_upstream(
  client: &SafeReqwest,
  base_url: &str,
  method: &Method,
  api_path: &str,
  prefix: Option<&str>,
  mut request: Option<Value>,
  query_params: Option<&[(String, String)]>,
  apply_auth: impl FnOnce(reqwest::RequestBuilder) -> reqwest::RequestBuilder,
  client_headers: Option<&[(String, String)]>,
) -> Result<Response> {
  let url = format!("{}{}", base_url, api_path);

  if let Some(ref req) = request {
    if let Some(prefix) = prefix {
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

  let mut http_request = client.request(method.clone(), &url)?;
  if method == Method::POST {
    http_request = http_request.header("Content-Type", "application/json");
  }

  if let Some(params) = query_params {
    http_request = http_request.query(params);
  }

  http_request = apply_auth(http_request);

  if let Some(hdrs) = client_headers {
    for (k, v) in hdrs {
      http_request = http_request.header(k.as_str(), v.as_str());
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
