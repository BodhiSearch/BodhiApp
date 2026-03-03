use crate::{AiApiServiceError, ErrorType};
use errmeta_derive::ErrorMeta;

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error_meta(trait_to_impl = crate::AppError)]
pub enum InferenceError {
  #[error("inference not supported in current deployment mode")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Unsupported,

  #[error("model not found: {0}")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ModelNotFound(String),

  #[error(transparent)]
  AiApi(#[from] AiApiServiceError),

  #[error("inference internal error: {0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Internal(String),
}
