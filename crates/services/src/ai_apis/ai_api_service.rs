use super::ai_provider_client::AIProviderClient;
use super::error::{AiApiServiceError, Result};
use super::provider_anthropic::AnthropicProviderClient;
use super::provider_anthropic_oauth::AnthropicOAuthProviderClient;
use super::provider_gemini::GeminiProviderClient;
use super::provider_openai::OpenAIProviderClient;
use super::provider_openai_responses::OpenAIResponsesProviderClient;
use crate::models::{ApiAlias, ApiFormat, ApiModel};
use crate::SafeReqwest;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;
use std::time::Duration;

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const TEST_PROMPT_MAX_LENGTH: usize = 30;

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AiApiService: Send + Sync + std::fmt::Debug {
  async fn test_prompt(
    &self,
    api_key: Option<String>,
    base_url: &str,
    model: &str,
    prompt: &str,
    api_format: &ApiFormat,
    extra_headers: Option<Value>,
    extra_body: Option<Value>,
  ) -> Result<String>;

  async fn fetch_models(
    &self,
    api_key: Option<String>,
    base_url: &str,
    api_format: &ApiFormat,
    extra_headers: Option<Value>,
    extra_body: Option<Value>,
  ) -> Result<Vec<ApiModel>>;

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

  /// Shared `SafeReqwest` instance used by all upstream calls. Cheap to clone
  /// (Arc-shared `reqwest::Client` underneath) — callers should clone, not
  /// build their own. Used by routes_app/providers/* for the OAuth refresh path.
  fn safe_http_client(&self) -> SafeReqwest;
}

#[derive(Debug, Clone)]
pub struct DefaultAiApiService {
  client: SafeReqwest,
}

impl DefaultAiApiService {
  pub fn new() -> Result<Self> {
    let client = SafeReqwest::builder()
      .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
      .allow_private_ips()
      .build()?;
    Ok(Self { client })
  }
}

#[async_trait]
impl AiApiService for DefaultAiApiService {
  fn safe_http_client(&self) -> SafeReqwest {
    self.client.clone()
  }

  async fn test_prompt(
    &self,
    api_key: Option<String>,
    base_url: &str,
    model: &str,
    prompt: &str,
    api_format: &ApiFormat,
    extra_headers: Option<Value>,
    extra_body: Option<Value>,
  ) -> Result<String> {
    if prompt.len() > TEST_PROMPT_MAX_LENGTH {
      return Err(AiApiServiceError::PromptTooLong {
        max_length: TEST_PROMPT_MAX_LENGTH,
        actual_length: prompt.len(),
      });
    }
    match api_format {
      ApiFormat::OpenAI => {
        OpenAIProviderClient::new(api_key, base_url.to_string(), self.client.clone())
          .test_connection(model, prompt)
          .await
      }
      ApiFormat::OpenAIResponses => {
        OpenAIResponsesProviderClient::new(api_key, base_url.to_string(), self.client.clone())
          .test_connection(model, prompt)
          .await
      }
      ApiFormat::Anthropic => {
        AnthropicProviderClient::new(api_key, base_url.to_string(), self.client.clone())
          .test_connection(model, prompt)
          .await
      }
      // For LlmLibertyOauth, the route handler resolves credentials and passes
      // the access_token as api_key + envelope headers/body as extra_headers/body,
      // so the upstream call is identical to AnthropicOAuth.
      ApiFormat::AnthropicOAuth | ApiFormat::LlmLibertyOauth => {
        AnthropicOAuthProviderClient::new(
          api_key,
          base_url.to_string(),
          self.client.clone(),
          extra_headers,
          extra_body,
        )
        .test_connection(model, prompt)
        .await
      }
      ApiFormat::Gemini => {
        GeminiProviderClient::new(api_key, base_url.to_string(), self.client.clone())
          .test_connection(model, prompt)
          .await
      }
    }
  }

  async fn fetch_models(
    &self,
    api_key: Option<String>,
    base_url: &str,
    api_format: &ApiFormat,
    extra_headers: Option<Value>,
    extra_body: Option<Value>,
  ) -> Result<Vec<ApiModel>> {
    match api_format {
      ApiFormat::OpenAI => {
        OpenAIProviderClient::new(api_key, base_url.to_string(), self.client.clone())
          .models()
          .await
      }
      ApiFormat::OpenAIResponses => {
        OpenAIResponsesProviderClient::new(api_key, base_url.to_string(), self.client.clone())
          .models()
          .await
      }
      ApiFormat::Anthropic => {
        AnthropicProviderClient::new(api_key, base_url.to_string(), self.client.clone())
          .models()
          .await
      }
      ApiFormat::AnthropicOAuth | ApiFormat::LlmLibertyOauth => {
        AnthropicOAuthProviderClient::new(
          api_key,
          base_url.to_string(),
          self.client.clone(),
          extra_headers,
          extra_body,
        )
        .models()
        .await
      }
      ApiFormat::Gemini => {
        GeminiProviderClient::new(api_key, base_url.to_string(), self.client.clone())
          .models()
          .await
      }
    }
  }

  async fn forward_request_with_method(
    &self,
    method: &Method,
    api_path: &str,
    api_alias: &ApiAlias,
    api_key: Option<String>,
    request: Option<Value>,
    query_params: Option<Vec<(String, String)>>,
    client_headers: Option<Vec<(String, String)>>,
  ) -> Result<Response> {
    let prefix = api_alias.prefix.as_deref();
    let qp = query_params.as_deref();
    let ch = client_headers.as_deref();
    match api_alias.api_format {
      ApiFormat::OpenAI => {
        OpenAIProviderClient::new(api_key, api_alias.base_url.clone(), self.client.clone())
          .forward(method, api_path, prefix, request, qp, ch)
          .await
      }
      ApiFormat::OpenAIResponses => {
        OpenAIResponsesProviderClient::new(api_key, api_alias.base_url.clone(), self.client.clone())
          .forward(method, api_path, prefix, request, qp, ch)
          .await
      }
      ApiFormat::Anthropic => {
        AnthropicProviderClient::new(api_key, api_alias.base_url.clone(), self.client.clone())
          .forward(method, api_path, prefix, request, qp, ch)
          .await
      }
      // For LlmLibertyOauth, the route handler resolves credentials, patches
      // alias.extra_headers/extra_body from the envelope, and passes the
      // access_token as api_key before reaching here — identical to AnthropicOAuth.
      ApiFormat::AnthropicOAuth | ApiFormat::LlmLibertyOauth => {
        AnthropicOAuthProviderClient::new(
          api_key,
          api_alias.base_url.clone(),
          self.client.clone(),
          api_alias.extra_headers.clone(),
          api_alias.extra_body.clone(),
        )
        .forward(method, api_path, prefix, request, qp, ch)
        .await
      }
      ApiFormat::Gemini => {
        GeminiProviderClient::new(api_key, api_alias.base_url.clone(), self.client.clone())
          .forward(method, api_path, prefix, request, qp, ch)
          .await
      }
    }
  }
}

#[cfg(test)]
#[path = "test_ai_api_anthropic.rs"]
mod test_ai_api_anthropic;
#[cfg(test)]
#[path = "test_ai_api_anthropic_oauth.rs"]
mod test_ai_api_anthropic_oauth;
#[cfg(test)]
#[path = "test_ai_api_forward.rs"]
mod test_ai_api_forward;
#[cfg(test)]
#[path = "test_ai_api_gemini.rs"]
mod test_ai_api_gemini;
#[cfg(test)]
#[path = "test_ai_api_openai.rs"]
mod test_ai_api_openai;
#[cfg(test)]
#[path = "test_ai_api_provider_matrix.rs"]
mod test_ai_api_provider_matrix;
#[cfg(test)]
#[path = "test_merge_extra_body.rs"]
mod test_merge_extra_body;
