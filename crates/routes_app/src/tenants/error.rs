use oauth2::url::ParseError;
use services::{AppError, AuthServiceError, ErrorType, TokenError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum DashboardAuthRouteError {
  #[error("Dashboard auth is only available in multi-tenant mode.")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  NotMultiTenant,
  #[error("Multi-tenant client configuration is missing.")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  MissingClientConfig,
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
  #[error("Must be logged in to the tenant before activating.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  TenantNotLoggedIn,
}
