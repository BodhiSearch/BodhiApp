use oauth2::url::ParseError;
use objs::{AppError, BadRequestError, ErrorType};
use services::{AppStatus, AuthServiceError, JsonWebTokenError, SecretServiceError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LoginError {
  #[error("app_reg_info_not_found")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  AppRegInfoNotFound,
  #[error("app_status_invalid")]
  #[error_meta(error_type = ErrorType::InvalidAppState)]
  AppStatusInvalid(AppStatus),
  #[error(transparent)]
  SecretServiceError(#[from] SecretServiceError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::Authentication, code = "login_error-session_error", args_delegate = false)]
  SessionError(#[from] tower_sessions::session::Error),
  #[error("session_info_not_found")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  SessionInfoNotFound,
  #[error("oauth_error")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  OAuthError(String),
  #[error(transparent)]
  AuthServiceError(#[from] AuthServiceError),
  #[error(transparent)]
  BadRequest(#[from] BadRequestError),
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, code = "login_error-parse_error", args_delegate = false)]
  ParseError(#[from] ParseError),
  #[error(transparent)]
  JsonWebToken(#[from] JsonWebTokenError),
  #[error("state_digest_mismatch")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  StateDigestMismatch,
}

#[cfg(test)]
mod tests {
  use crate::{AppServiceError, CreateAliasError, LoginError, LogoutError, PullError};
  use oauth2::url::ParseError;
  use objs::{
    test_utils::{assert_error_message, setup_l10n},
    AppError, FluentLocalizationService,
  };
  use rstest::rstest;
  use services::AppStatus;
  use std::sync::Arc;

  #[rstest]
  #[case(&AppServiceError::AlreadySetup, "app is already setup")]
  #[case(&LoginError::AppRegInfoNotFound, "app is not registered, need to register app first")]
  #[case(&LoginError::SessionInfoNotFound, "login info not found in session, are cookies enabled?")]
  #[case(&LoginError::OAuthError("access_denied".to_string()), "OAuth authentication failed: access_denied")]
  #[case(&LoginError::AppStatusInvalid(AppStatus::Setup), "app status is invalid for this operation, status: setup")]
  #[case(&LoginError::ParseError(ParseError::EmptyHost), "failed to parse url, error: empty host")]
  #[case(&LoginError::SessionError(serde_json::from_str::<String>("{invalid").unwrap_err().into()), "failed to access session information, try again, or try logging out and login again, error: invalid type: map, expected a string at line 1 column 0")]
  #[case(&CreateAliasError::AliasNotPresent, "alias is not present in request")]
  #[case(&CreateAliasError::AliasMismatch { path: "pathalias".to_string(), request: "requestalias".to_string() }, "alias in path 'pathalias' does not match alias in request 'requestalias'")]
  #[case(&LogoutError::SessionDelete(serde_json::from_str::<String>("{invalid").unwrap_err().into()), "failed to delete session, error: invalid type: map, expected a string at line 1 column 0")]
  #[case(&PullError::FileAlreadyExists { repo: "test/repo".to_string(), filename: "filename.gguf".to_string(), snapshot: "main".to_string() }, "file filename.gguf already exists in repo test/repo with snapshot main")]
  #[case(&LoginError::StateDigestMismatch, "state parameter in callback does not match with the one sent in login request")]
  fn test_error_messages_routes_app(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected: &str,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), expected);
  }
}
