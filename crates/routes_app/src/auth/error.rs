use oauth2::url::ParseError;
use services::{AppError, ErrorType};
use services::{AppStatus, AuthServiceError, TenantError, TokenError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AuthRouteError {
  #[error("Tenant not found.")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  TenantNotFound,
  #[error("Application status is invalid for this operation. Current status: {0}.")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  AppStatusInvalid(AppStatus),
  #[error(transparent)]
  TenantError(#[from] TenantError),
  #[error("{0}")]
  #[error_meta(error_type = ErrorType::Authentication, args_delegate = false)]
  SessionError(#[from] tower_sessions::session::Error),
  #[error("Login session not found. Are cookies enabled?")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  SessionInfoNotFound,
  #[error("Login failed: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  OAuthError(String),
  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),
  #[error("{0}")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  ParseError(#[from] ParseError),
  #[error(transparent)]
  TokenError(#[from] TokenError),
  #[error("State parameter in callback does not match the one sent in login request.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  StateDigestMismatch,
  #[error("Missing state parameter in OAuth callback.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingState,
  #[error("Missing code parameter in OAuth callback.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingCode,
  #[error("Failed to delete session: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  SessionDelete(tower_sessions::session::Error),
}
