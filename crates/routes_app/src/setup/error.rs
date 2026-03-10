use services::{AppError, AuthServiceError, ErrorType, TenantError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum SetupRouteError {
  #[error("Application is already set up.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  AlreadySetup,
  #[error("Setup is only available in standalone deployment mode.")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  NotStandalone,
  #[error("Server name must be at least 10 characters long.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ServerNameTooShort,
  #[error(transparent)]
  TenantError(#[from] TenantError),
  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),
}
