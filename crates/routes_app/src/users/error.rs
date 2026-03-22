use services::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum UsersRouteError {
  // --- from UserRouteError ---
  #[error("Failed to list users: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ListFailed(String),
  #[error("Failed to change user role: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  RoleChangeFailed(String),
  #[error("Failed to remove user: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  RemoveFailed(String),

  // --- from AccessRequestError ---
  #[error("Access request already pending.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  AlreadyPending,

  #[error("User already has access.")]
  #[error_meta(error_type = ErrorType::UnprocessableEntity)]
  AlreadyHasAccess,

  #[error("Tenant selection required to request access.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  TenantRequired,

  #[error("Pending access request for user not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  PendingRequestNotFound,

  #[error("Access request {0} not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  RequestNotFound(String),

  #[error("Access request has already been processed.")]
  #[error_meta(error_type = ErrorType::Conflict)]
  AlreadyProcessed,

  #[error("Insufficient privileges to assign this role.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  InsufficientPrivileges,

  #[error("Failed to fetch access requests: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  FetchFailed(String),
}
