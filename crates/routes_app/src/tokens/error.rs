use services::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum TokenRouteError {
  #[error("Access token is missing.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  AccessTokenMissing,
  #[error("Privilege escalation not allowed.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  PrivilegeEscalation,
}
