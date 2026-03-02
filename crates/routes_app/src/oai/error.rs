use services::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum OAIRouteError {
  #[error("Error constructing HTTP response: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Http(#[from] http::Error),

  #[error("Response serialization failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Serialization(#[from] serde_json::Error),

  #[error("{0}")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidRequest(String),
}
