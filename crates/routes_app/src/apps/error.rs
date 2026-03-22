use services::{AccessRequestError, AuthServiceError, TenantError, ToolsetError};
use services::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppsRouteError {
  #[error("Access request not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound,

  #[error("Access request has expired.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  Expired,

  #[error("Access request already processed.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  AlreadyProcessed,

  #[error("Redirect URL required for redirect flow.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingRedirectUrl,

  #[error("Invalid redirect URL '{0}': only http:// and https:// URLs are allowed.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InvalidRedirectUrl(String),

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

  #[error("Session role is required to approve access requests.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  InsufficientPrivileges,

  #[error("Approved role '{approved}' exceeds allowed maximum '{max_allowed}' for this user.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  PrivilegeEscalation {
    approved: String,
    max_allowed: String,
  },

  #[error(transparent)]
  AccessRequestServiceError(#[from] AccessRequestError),

  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),

  #[error(transparent)]
  ToolServiceError(#[from] ToolsetError),

  #[error(transparent)]
  TenantError(#[from] TenantError),
}
