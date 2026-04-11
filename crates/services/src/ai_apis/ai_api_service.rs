use super::ai_provider_client::{
  AnthropicProviderClient, OpenAIProviderClient, OpenAIResponsesProviderClient, AIProviderClient,
};
use super::error::{AiApiServiceError, Result};
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
  ) -> Result<String>;

  async fn fetch_models(
    &self,
    api_key: Option<String>,
    base_url: &str,
    api_format: &ApiFormat,
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
    }
  }

  async fn fetch_models(
    &self,
    api_key: Option<String>,
    base_url: &str,
    api_format: &ApiFormat,
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
        OpenAIResponsesProviderClient::new(
          api_key,
          api_alias.base_url.clone(),
          self.client.clone(),
        )
        .forward(method, api_path, prefix, request, qp, ch)
        .await
      }
      ApiFormat::Anthropic => {
        AnthropicProviderClient::new(api_key, api_alias.base_url.clone(), self.client.clone())
          .forward(method, api_path, prefix, request, qp, ch)
          .await
      }
    }
  }
}

#[cfg(test)]
#[path = "test_ai_api_service.rs"]
mod test_ai_api_service;
