use services::{AccessRequestError, AppScopeError, AuthServiceError, TenantError};
use services::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppsRouteError {
  #[error("Invalid consent request ({error}): {error_description}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ConsentRejected {
    error: String,
    error_description: String,
  },

  #[error("Consent field '{0}' is required to approve.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ConsentFieldMissing(String),

  #[error("Created access request is missing its scope.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  MissingAccessRequestScope,

  #[error("MCP instance not owned by user: {0}.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  McpInstanceNotOwned(String),

  #[error("MCP instance not configured properly: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  McpInstanceNotConfigured(String),

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
  TenantError(#[from] TenantError),

  #[error(transparent)]
  ScopeError(#[from] AppScopeError),
}
