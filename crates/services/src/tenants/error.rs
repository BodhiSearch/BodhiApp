use crate::auth::AuthContextError;
use crate::db::DbError;
use errmeta::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum TenantError {
  #[error("Tenant not found.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  NotFound,
  #[error("User already has a tenant.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  UserAlreadyHasTenant,
  #[error(transparent)]
  Db(#[from] DbError),
  #[error(transparent)]
  AuthContext(#[from] AuthContextError),
}

pub(crate) type Result<T> = std::result::Result<T, TenantError>;
