use crate::db::DbError;
use crate::{AuthServiceError, TenantError};
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

  #[error("Approved resources version '{approved_version}' does not match requested version '{requested_version}'.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  VersionMismatch {
    requested_version: String,
    approved_version: String,
  },

  #[error("Keycloak registration failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  KcRegistrationFailed(String),

  #[error(transparent)]
  Db(#[from] DbError),

  #[error(transparent)]
  Auth(#[from] AuthServiceError),

  #[error(transparent)]
  Tenant(#[from] TenantError),
}

pub(crate) type Result<T> = std::result::Result<T, AccessRequestError>;
