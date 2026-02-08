use objs::{impl_error_from, AppError, ErrorType, IoError, ReqwestError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ServerError {
  #[error("Model server is starting up. Please wait and try again.")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  ServerNotReady,

  #[error("Failed to start model server: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  StartupError(String),

  #[error(transparent)]
  IoError(#[from] IoError),

  #[error(transparent)]
  ClientError(#[from] ReqwestError),

  #[error("Model server health check failed: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  HealthCheckError(String),

  #[error("Model server did not respond within {0} seconds.")]
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
  use objs::AppError;
  use std::io::{Error as StdIoError, ErrorKind};

  #[test]
  fn test_error_types() {
    assert_eq!(
      ServerError::ServerNotReady.error_type(),
      ErrorType::ServiceUnavailable.to_string()
    );
    assert_eq!(
      ServerError::TimeoutError(30).error_type(),
      ErrorType::InternalServer.to_string()
    );
  }

  #[test]
  fn test_error_status_codes() {
    assert_eq!(
      StatusCode::SERVICE_UNAVAILABLE.as_u16(),
      ServerError::ServerNotReady.status()
    );
    assert_eq!(
      StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
      ServerError::StartupError("test".to_string()).status()
    );
  }

  #[test]
  fn test_error_display() {
    assert_eq!(
      "Model server is starting up. Please wait and try again.",
      ServerError::ServerNotReady.to_string()
    );
    assert_eq!(
      "Failed to start model server: failed to execute.",
      ServerError::StartupError("failed to execute".to_string()).to_string()
    );
    assert_eq!(
      "Model server health check failed: connection refused.",
      ServerError::HealthCheckError("connection refused".to_string()).to_string()
    );
    assert_eq!(
      "Model server did not respond within 30 seconds.",
      ServerError::TimeoutError(30).to_string()
    );
  }

  #[test]
  fn test_error_from_io() {
    let io_error = StdIoError::new(ErrorKind::NotFound, "process failed");
    let server_error: ServerError = io_error.into();
    match server_error {
      ServerError::IoError(_) => (),
      _ => panic!("expected IoError variant"),
    }
  }
}
