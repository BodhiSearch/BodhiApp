use crate::inference::InferenceError;
use crate::models::{Alias, ApiAlias};
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

/// Clone (not Copy) because response ID variants contain owned String.
#[derive(Debug, Clone, PartialEq)]
pub enum LlmEndpoint {
  ChatCompletions,
  Embeddings,
  Responses,
  ResponsesGet(String),
  ResponsesDelete(String),
  ResponsesInputItems(String),
  ResponsesCancel(String),
  AnthropicMessages,
  AnthropicModels,
  AnthropicModel(String),
}

impl LlmEndpoint {
  pub fn api_path(&self) -> String {
    match self {
      Self::ChatCompletions => "/chat/completions".to_string(),
      Self::Embeddings => "/embeddings".to_string(),
      Self::Responses => "/responses".to_string(),
      Self::ResponsesGet(id) | Self::ResponsesDelete(id) => format!("/responses/{}", id),
      Self::ResponsesInputItems(id) => format!("/responses/{}/input_items", id),
      Self::ResponsesCancel(id) => format!("/responses/{}/cancel", id),
      Self::AnthropicMessages => "/messages".to_string(),
      Self::AnthropicModels => "/models".to_string(),
      Self::AnthropicModel(id) => format!("/models/{}", id),
    }
  }

  pub fn http_method(&self) -> &'static Method {
    match self {
      Self::ResponsesGet(_) | Self::ResponsesInputItems(_) => &Method::GET,
      Self::AnthropicModels | Self::AnthropicModel(_) => &Method::GET,
      Self::ResponsesDelete(_) => &Method::DELETE,
      _ => &Method::POST,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  #[rstest]
  #[case(LlmEndpoint::AnthropicMessages, "/messages", &Method::POST)]
  #[case(LlmEndpoint::AnthropicModels, "/models", &Method::GET)]
  #[case(
    LlmEndpoint::AnthropicModel("claude-3-5-sonnet".to_string()),
    "/models/claude-3-5-sonnet",
    &Method::GET
  )]
  fn test_anthropic_endpoint_paths(
    #[case] endpoint: LlmEndpoint,
    #[case] expected_path: &str,
    #[case] expected_method: &Method,
  ) {
    assert_eq!(expected_path, endpoint.api_path());
    assert_eq!(expected_method, endpoint.http_method());
  }
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait InferenceService: Send + Sync + std::fmt::Debug {
  async fn forward_local(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
    alias: Alias,
  ) -> Result<Response, InferenceError>;

  async fn forward_remote(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
    api_alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<Response, InferenceError> {
    self
      .forward_remote_with_params(endpoint, request, api_alias, api_key, None, None)
      .await
  }

  async fn forward_remote_with_params(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
    api_alias: &ApiAlias,
    api_key: Option<String>,
    query_params: Option<Vec<(String, String)>>,
    client_headers: Option<Vec<(String, String)>>,
  ) -> Result<Response, InferenceError>;

  /// Stop the LLM server process (no-op for multi-tenant/remote deployments)
  async fn stop(&self) -> Result<(), InferenceError>;

  /// Update the execution variant (e.g., cpu, cuda) — no-op for remote deployments
  async fn set_variant(&self, variant: &str) -> Result<(), InferenceError>;

  /// Update the keep-alive timeout in seconds — no-op for remote deployments
  async fn set_keep_alive(&self, secs: i64);

  /// Returns true if the local LLM server is currently loaded/running
  async fn is_loaded(&self) -> bool;
}
