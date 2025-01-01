use objs::{impl_error_from, AppError, ErrorType, IoError, ReqwestError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ServerError {
  #[error("server_not_ready")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ServerNotReady,

  #[error("startup_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  StartupError(String),

  #[error(transparent)]
  IoError(#[from] IoError),

  #[error(transparent)]
  ClientError(#[from] ReqwestError),

  #[error("health_check_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  HealthCheckError(String),

  #[error("timeout_error")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  TimeoutError(u64),
}

impl_error_from!(::std::io::Error, ServerError::IoError, ::objs::IoError);
impl_error_from!(
  reqwest::Error,
  ServerError::ClientError,
  ::objs::ReqwestError
);
pub type Result<T> = std::result::Result<T, ServerError>;

#[cfg(test)]
mod tests {
  use super::*;
  use axum::http::StatusCode;
  use objs::{
    test_utils::assert_error_message, test_utils::setup_l10n, AppError, FluentLocalizationService,
  };
  use rstest::rstest;
  use std::io::{Error as StdIoError, ErrorKind};
  use std::sync::Arc;

  #[rstest]
  #[case::server_not_ready(
        &ServerError::ServerNotReady,
        "server not ready: the server process has not completed initialization"
    )]
  #[case::startup_error(
        &ServerError::StartupError("failed to execute".to_string()),
        "failed to start server: failed to execute"
    )]
  #[case::process_error(
        &ServerError::IoError(IoError::new(StdIoError::new(ErrorKind::NotFound, "process failed"))),
        "io_error: process failed"
    )]
  #[case::client_error(
        &ServerError::ClientError(ReqwestError::new("test_error".to_string())),
        "error connecting to internal service: test_error"
    )]
  #[case::health_check_error(
        &ServerError::HealthCheckError("connection refused".to_string()),
        "server health check failed: connection refused"
    )]
  #[case::timeout_error(
        &ServerError::TimeoutError(30),
        "server health check timed out after 30 seconds"
    )]
  fn test_error_messages(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected: &str,
  ) {
    assert_error_message(localization_service, &error.code(), error.args(), expected);
  }

  #[test]
  fn test_error_types() {
    assert_eq!(
      ServerError::ServerNotReady.error_type(),
      ErrorType::InternalServer.to_string()
    );
    assert_eq!(
      ServerError::TimeoutError(30).error_type(),
      ErrorType::InternalServer.to_string()
    );
  }

  #[test]
  fn test_error_status_codes() {
    assert_eq!(
      StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
      ServerError::ServerNotReady.status()
    );
    assert_eq!(
      StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
      ServerError::StartupError("test".to_string()).status()
    );
  }
}
