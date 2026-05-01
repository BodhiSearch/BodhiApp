use crate::ai_apis::ai_api_client::AiApiClient;
use crate::ai_apis::error::{AiApiClientFactoryError, Result};
use crate::ai_apis::provider_shared::convert_reqwest_to_axum;
use crate::inference::{LlmEndpoint, LocalLlama};
use crate::models::{Alias, ApiModel};
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use serde_json::Value;
use std::sync::Arc;

pub(crate) struct LocalLlamaClient {
  local_llama: Arc<dyn LocalLlama>,
  alias: Alias,
}

impl LocalLlamaClient {
  pub(crate) fn new(local_llama: Arc<dyn LocalLlama>, alias: Alias) -> Self {
    Self { local_llama, alias }
  }
}

#[async_trait]
impl AiApiClient for LocalLlamaClient {
  async fn test_prompt(&self, _model: &str, _prompt: &str) -> Result<String> {
    Err(AiApiClientFactoryError::ApiError(
      "test_prompt is not supported for local model aliases".to_string(),
    ))
  }

  async fn fetch_models(&self) -> Result<Vec<ApiModel>> {
    Err(AiApiClientFactoryError::ApiError(
      "fetch_models is not supported for local model aliases".to_string(),
    ))
  }

  async fn forward_request_with_method(
    &self,
    _method: &Method,
    api_path: &str,
    request: Option<Value>,
    _query_params: Option<Vec<(String, String)>>,
    _client_headers: Option<Vec<(String, String)>>,
  ) -> Result<Response> {
    let endpoint = match api_path {
      "/chat/completions" => LlmEndpoint::ChatCompletions,
      "/embeddings" => LlmEndpoint::Embeddings,
      other => {
        return Err(AiApiClientFactoryError::NotFound(format!(
          "endpoint '{}' is not supported for local model aliases",
          other
        )))
      }
    };
    let body = request.unwrap_or(Value::Null);
    let reqwest_response = self
      .local_llama
      .forward_request(endpoint, body, self.alias.clone())
      .await?;
    convert_reqwest_to_axum(reqwest_response)
  }
}
