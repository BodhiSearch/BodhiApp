use crate::db::DbError;
use crate::{AppInstanceError, AuthServiceError, ToolsetError};
use errmeta::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AccessRequestError {
  #[error("Access request '{0}' not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),

  #[error("Access request '{0}' has expired.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  Expired(String),

  #[error("Access request '{0}' has already been processed.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  AlreadyProcessed(String),

  #[error("Invalid status '{0}' for access request.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidStatus(String),

  #[error("Redirect URI is required for redirect flow.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingRedirectUri,

  #[error("Keycloak registration failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  KcRegistrationFailed(String),

  #[error(transparent)]
  DbError(#[from] DbError),

  #[error(transparent)]
  AuthError(#[from] AuthServiceError),

  #[error(transparent)]
  ToolError(#[from] ToolsetError),

  #[error(transparent)]
  AppInstance(#[from] AppInstanceError),
}

pub(crate) type Result<T> = std::result::Result<T, AccessRequestError>;
