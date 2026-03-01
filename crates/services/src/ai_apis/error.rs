use crate::ReqwestError;
use errmeta::{impl_error_from, AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AiApiServiceError {
  #[error(transparent)]
  Reqwest(#[from] ReqwestError),

  #[error("API error: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ApiError(String),

  #[error("API authentication failed: {0}.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  Unauthorized(String),

  #[error("Resource not found: {0}.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),

  #[error("Too many requests to API. Please wait and try again.")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  RateLimit(String),

  #[error("Message too long. Maximum length is {max_length} but received {actual_length}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  PromptTooLong {
    max_length: usize,
    actual_length: usize,
  },

  #[error("API model '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ModelNotFound(String),
}

impl_error_from!(
  reqwest::Error,
  AiApiServiceError::Reqwest,
  crate::ReqwestError
);

pub(crate) type Result<T> = std::result::Result<T, AiApiServiceError>;
