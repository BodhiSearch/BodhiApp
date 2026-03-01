use crate::db::DbError;
use errmeta::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ExaError {
  #[error("Search request failed: {0}.")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  RequestFailed(String),

  #[error("Search rate limit exceeded. Please wait and try again.")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  RateLimited,

  #[error("Search API key is invalid or missing.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidApiKey,

  #[error("Search request timed out.")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  Timeout,
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolsetError {
  #[error("Toolset '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  ToolsetNotFound(String),

  #[error("Toolset method '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  MethodNotFound(String),

  #[error("Toolset is not configured for this user.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ToolsetNotConfigured,

  #[error("Toolset is disabled.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ToolsetDisabled,

  #[error("Toolset execution failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ExecutionFailed(String),

  #[error("Toolset application is disabled.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ToolsetAppDisabled,

  #[error("Toolset slug '{0}' already exists.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  SlugExists(String),

  #[error("Invalid toolset slug: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidSlug(String),

  #[error("Invalid toolset description: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidDescription(String),

  #[error("Invalid toolset type: {0}.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  InvalidToolsetType(String),

  #[error(transparent)]
  DbError(#[from] DbError),

  #[error(transparent)]
  ExaError(#[from] ExaError),
}
