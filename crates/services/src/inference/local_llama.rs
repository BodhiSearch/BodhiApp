use crate::models::Alias;
use crate::{AppError, ErrorType};
use errmeta_derive::ErrorMeta;
use serde_json::Value;

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LocalLlamaError {
  #[error("model not found: {0}")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ModelNotFound(String),

  #[error("llama executable not found: {0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ExecNotFound(String),

  #[error("local inference error: {0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Internal(String),
}

#[cfg_attr(any(test, feature = "test-utils"), mockall::automock)]
#[async_trait::async_trait]
pub trait LocalLlama: Send + Sync + std::fmt::Debug {
  async fn forward_request(
    &self,
    api_path: &str,
    request: Value,
    alias: Alias,
  ) -> Result<reqwest::Response, LocalLlamaError>;

  async fn stop(&self) -> Result<(), LocalLlamaError>;
  async fn set_variant(&self, variant: &str) -> Result<(), LocalLlamaError>;
  async fn set_keep_alive(&self, secs: i64);
  async fn is_loaded(&self) -> bool;
}
