use crate::db::DbError;
use crate::exa_service::ExaError;
use objs::{AppError, ErrorType};

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

  #[error("Toolset name '{0}' already exists.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  NameExists(String),

  #[error("Invalid toolset name: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidName(String),

  #[error("Invalid toolset type: {0}.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  InvalidToolsetType(String),

  #[error(transparent)]
  DbError(#[from] DbError),

  #[error(transparent)]
  ExaError(#[from] ExaError),
}
