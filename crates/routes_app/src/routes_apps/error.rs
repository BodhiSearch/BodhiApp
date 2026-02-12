use objs::{AppError, ErrorType};
use services::{AccessRequestError, AuthServiceError, ToolsetError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppAccessRequestError {
  #[error("Access request not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound,

  #[error("Access request has expired.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  Expired,

  #[error("Access request already processed.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  AlreadyProcessed,

  #[error("Invalid flow type: {0}. Must be 'redirect' or 'popup'.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidFlowType(String),

  #[error("Redirect URL required for redirect flow.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingRedirectUrl,

  #[error("App client not found: {0}.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  AppClientNotFound(String),

  #[error("Invalid tool type: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidToolType(String),

  #[error("Tool instance not owned by user: {0}.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  ToolInstanceNotOwned(String),

  #[error("Tool instance not configured properly: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ToolInstanceNotConfigured(String),

  #[error(transparent)]
  AccessRequestServiceError(#[from] AccessRequestError),

  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),

  #[error(transparent)]
  ToolServiceError(#[from] ToolsetError),
}
