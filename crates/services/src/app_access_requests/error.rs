use crate::db::DbError;
use crate::{AuthServiceError, TenantError};
use errmeta::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AccessRequestError {
  #[error("Failed to serialize access request payload: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Serialization(String),

  #[error(transparent)]
  Db(#[from] DbError),

  #[error(transparent)]
  Auth(#[from] AuthServiceError),

  #[error(transparent)]
  Tenant(#[from] TenantError),

  #[error(transparent)]
  Scope(#[from] AppScopeError),
}

pub(crate) type Result<T> = std::result::Result<T, AccessRequestError>;

/// App-facing scope vocabulary violations. Route handlers map these onto the
/// OAuth error codes (`invalid_scope` / `invalid_request`) for the consent flow.
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppScopeError {
  #[error("Unrecognized scope token '{token}'.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MalformedScopeToken { token: String },

  #[error("Scope token '{token}' conflicts with an earlier token in the same request.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ConflictingScopeToken { token: String },

  #[error("Scope token '{token}' is reserved for server-side composition.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ReservedScopeToken { token: String },

  #[error("Grant for '{section}' exceeds what the requested scope allows.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  GrantExceedsScope { section: String },

  #[error("Cannot compose access-request scope from resource client '{resource_client_id}' and id '{access_request_id}'.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  InvalidScopeComposition {
    resource_client_id: String,
    access_request_id: String,
  },
}
