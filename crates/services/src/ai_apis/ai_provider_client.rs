use super::error::Result;
use crate::models::ApiModel;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

/// Strategy trait for provider-specific HTTP calls.
#[async_trait]
pub trait AIProviderClient: Send + Sync {
  type CompletionResponse: Send;

  async fn models(&self) -> Result<Vec<ApiModel>>;
  async fn test_connection(&self, model: &str, prompt: &str) -> Result<Self::CompletionResponse>;
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
