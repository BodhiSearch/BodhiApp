use crate::inference::InferenceError;
use crate::models::{Alias, ApiAlias};
use axum::response::Response;
use serde_json::Value;

/// The LLM endpoint type for routing inference requests
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LlmEndpoint {
  ChatCompletions,
  Embeddings,
}

impl LlmEndpoint {
  pub fn api_path(&self) -> &str {
    match self {
      Self::ChatCompletions => "/chat/completions",
      Self::Embeddings => "/embeddings",
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

  /// Stop the LLM server process (no-op for multi-tenant/remote deployments)
  async fn stop(&self) -> Result<(), InferenceError>;

  /// Update the execution variant (e.g., cpu, cuda) — no-op for remote deployments
  async fn set_variant(&self, variant: &str) -> Result<(), InferenceError>;

  /// Update the keep-alive timeout in seconds — no-op for remote deployments
  async fn set_keep_alive(&self, secs: i64);

  /// Returns true if the local LLM server is currently loaded/running
  async fn is_loaded(&self) -> bool;
}
