use services::{
  db::DbError, AppError, AuthServiceError, ErrorType, RoleError, TenantError, TokenError,
  TokenScopeError, UserScopeError,
};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AuthError {
  #[error(transparent)]
  Token(#[from] TokenError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  Role(#[from] RoleError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  TokenScope(#[from] TokenScopeError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication)]
  UserScope(#[from] UserScopeError),
  #[error("User has no valid access roles.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  MissingRoles,
  #[error("Access denied.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidAccess,
  #[error("API token is inactive.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  TokenInactive,
  #[error("API token not found.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  TokenNotFound,
  #[error(transparent)]
  AuthService(#[from] AuthServiceError),
  #[error(transparent)]
  Tenant(#[from] TenantError),
  #[error(transparent)]
  DbError(#[from] DbError),
  #[error("Session expired. Please log out and log in again.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  RefreshTokenNotFound,
  #[error("{0}")]
  #[error_meta(error_type = ErrorType::Authentication)]
  TowerSession(#[from] tower_sessions::session::Error),
  #[error("Invalid token: {0}.")]
  #[error_meta(error_type = ErrorType::Authentication)]
  InvalidToken(String),
}
