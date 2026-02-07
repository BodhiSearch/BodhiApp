use oauth2::url::ParseError;
use objs::{AppError, ErrorType};
use services::{AppStatus, AuthServiceError, JsonWebTokenError, SecretServiceError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LoginError {
  #[error("Application is not registered. Please register the application first.")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  AppRegInfoNotFound,
  #[error("Application status is invalid for this operation. Current status: {0}.")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  AppStatusInvalid(AppStatus),
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication, code = "login_error-session_error", args_delegate = false)]
  SessionError(#[from] tower_sessions::session::Error),
  #[error("Login session not found. Are cookies enabled?")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  SessionInfoNotFound,
  #[error("Login failed: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  OAuthError(String),
  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, code = "login_error-parse_error", args_delegate = false)]
  ParseError(#[from] ParseError),
  #[error(transparent)]
  JsonWebToken(#[from] JsonWebTokenError),
  #[error("State parameter in callback does not match the one sent in login request.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  StateDigestMismatch,
  #[error("Missing state parameter in OAuth callback.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingState,
  #[error("Missing code parameter in OAuth callback.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  MissingCode,
}
