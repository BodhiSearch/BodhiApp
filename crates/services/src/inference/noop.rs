use crate::inference::{InferenceError, InferenceService, LlmEndpoint};
use crate::models::{Alias, ApiAlias};
use axum::response::Response;
use serde_json::Value;

/// A no-op `InferenceService` used as a placeholder until a concrete implementation is wired.
/// All methods return `InferenceError::Unsupported`.
#[derive(Debug, Default, Clone)]
pub struct NoopInferenceService;

#[async_trait::async_trait]
impl InferenceService for NoopInferenceService {
  async fn forward_local(
    &self,
    _endpoint: LlmEndpoint,
    _request: Value,
    _alias: Alias,
  ) -> Result<Response, InferenceError> {
    Err(InferenceError::Unsupported)
  }

  async fn forward_remote(
    &self,
    _endpoint: LlmEndpoint,
    _request: Value,
    _api_alias: &ApiAlias,
    _api_key: Option<String>,
  ) -> Result<Response, InferenceError> {
    Err(InferenceError::Unsupported)
  }

  async fn stop(&self) -> Result<(), InferenceError> {
    Ok(())
  }

  async fn set_variant(&self, _variant: &str) -> Result<(), InferenceError> {
    Ok(())
  }

  async fn set_keep_alive(&self, _secs: i64) {}

  async fn is_loaded(&self) -> bool {
    false
  }
}
