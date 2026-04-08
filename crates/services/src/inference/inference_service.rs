use crate::inference::InferenceError;
use crate::models::{Alias, ApiAlias};
use axum::response::Response;
use serde_json::Value;

/// The LLM endpoint type for routing inference requests
#[derive(Debug, Clone, PartialEq)]
pub enum LlmEndpoint {
  ChatCompletions,
  Embeddings,
  /// POST /v1/responses — create a response
  Responses,
  /// GET /v1/responses/{id} — retrieve a response
  ResponsesGet(String),
  /// DELETE /v1/responses/{id} — delete a response
  ResponsesDelete(String),
  /// GET /v1/responses/{id}/input_items — list input items
  ResponsesInputItems(String),
  /// POST /v1/responses/{id}/cancel — cancel a background response
  ResponsesCancel(String),
}

impl LlmEndpoint {
  pub fn api_path(&self) -> String {
    match self {
      Self::ChatCompletions => "/chat/completions".to_string(),
      Self::Embeddings => "/embeddings".to_string(),
      Self::Responses => "/responses".to_string(),
      Self::ResponsesGet(id) => format!("/responses/{}", id),
      Self::ResponsesDelete(id) => format!("/responses/{}", id),
      Self::ResponsesInputItems(id) => format!("/responses/{}/input_items", id),
      Self::ResponsesCancel(id) => format!("/responses/{}/cancel", id),
    }
  }

  pub fn http_method(&self) -> &'static str {
    match self {
      Self::ResponsesGet(_) | Self::ResponsesInputItems(_) => "GET",
      Self::ResponsesDelete(_) => "DELETE",
      _ => "POST",
    }
  }
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait InferenceService: Send + Sync + std::fmt::Debug {
  /// Forward a request to a local model via SharedContext
  async fn forward_local(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
    alias: Alias,
  ) -> Result<Response, InferenceError>;

  /// Forward a request to a remote API provider
  async fn forward_remote(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
    api_alias: &ApiAlias,
    api_key: Option<String>,
  ) -> Result<Response, InferenceError>;

  /// Forward a request to a remote API provider with optional query parameters.
  /// Used by Responses API GET endpoints that need to forward query params upstream.
  async fn forward_remote_with_params(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
    api_alias: &ApiAlias,
    api_key: Option<String>,
    query_params: Option<Vec<(String, String)>>,
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
