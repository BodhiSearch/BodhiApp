use services::{AppError, ErrorType};
use services::{RoleError, TokenScopeError, UserScopeError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ApiAuthError {
  #[error("Insufficient permissions for this resource.")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  Forbidden,
  #[error("Authentication required. Provide an API key or log in.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingAuth,
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidRole(#[from] RoleError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidScope(#[from] TokenScopeError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidUserScope(#[from] UserScopeError),
}
