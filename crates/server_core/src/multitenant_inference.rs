use crate::standalone_inference::proxy_to_remote;
use axum::response::Response;
use serde_json::Value;
use services::inference::{InferenceError, InferenceService, LlmEndpoint};
use services::{AiApiService, Alias, ApiAlias};
use std::sync::Arc;

#[derive(Debug)]
pub struct MultitenantInferenceService {
  ai_api_service: Arc<dyn AiApiService>,
}

impl MultitenantInferenceService {
  pub fn new(ai_api_service: Arc<dyn AiApiService>) -> Self {
    Self { ai_api_service }
  }
}

#[async_trait::async_trait]
impl InferenceService for MultitenantInferenceService {
  async fn forward_local(
    &self,
    _endpoint: LlmEndpoint,
    _request: Value,
    _alias: Alias,
  ) -> Result<Response, InferenceError> {
    Err(InferenceError::Unsupported)
  }

  async fn forward_remote_with_params(
    &self,
    endpoint: LlmEndpoint,
    request: Value,
    api_alias: &ApiAlias,
    api_key: Option<String>,
    query_params: Option<Vec<(String, String)>>,
    client_headers: Option<Vec<(String, String)>>,
  ) -> Result<Response, InferenceError> {
    proxy_to_remote(
      &self.ai_api_service,
      endpoint,
      request,
      api_alias,
      api_key,
      query_params,
      client_headers,
    )
    .await
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
