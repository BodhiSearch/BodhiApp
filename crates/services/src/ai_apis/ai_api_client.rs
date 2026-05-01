use super::error::Result;
use crate::models::ApiModel;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;

/// Per-request client returned by the `AiApiClientFactory` factory.
///
/// The implementation owns its bound credentials and URLs; method signatures
/// carry only per-call data. Constructed cheaply per request via
/// `AiApiClientFactory::for_alias` or `for_liberty`.
#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait]
pub trait AiApiClient: Send + Sync {
  async fn test_prompt(&self, model: &str, prompt: &str) -> Result<String>;
  async fn fetch_models(&self) -> Result<Vec<ApiModel>>;
  async fn forward_request_with_method(
    &self,
    method: &Method,
    api_path: &str,
    request: Option<Value>,
    query_params: Option<Vec<(String, String)>>,
    client_headers: Option<Vec<(String, String)>>,
  ) -> Result<Response>;
}
