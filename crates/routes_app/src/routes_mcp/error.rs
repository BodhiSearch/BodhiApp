use objs::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum McpValidationError {
  #[error("Validation error: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Validation(String),

  #[error("OAuth state mismatch (CSRF protection).")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  CsrfStateMismatch,

  #[error("OAuth state expired. Please initiate login again.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  CsrfStateExpired,

  #[error("OAuth session data not found. Initiate login first.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  SessionDataMissing,

  #[error("Token exchange failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  TokenExchangeFailed(String),

  #[error("Invalid URL: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidUrl(String),

  #[error("Invalid redirect_uri: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidRedirectUri(String),
}
