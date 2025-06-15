use std::io;

use objs::{AppError, ErrorMessage, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppSetupError {
  #[error("io_error: error spawning async runtime: {0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  AsyncRuntime(#[from] io::Error),
}

impl From<AppSetupError> for ErrorMessage {
  fn from(value: AppSetupError) -> Self {
    ErrorMessage::new(value.code(), value.error_type(), value.to_string())
  }
}

#[cfg(test)]
mod tests {
  use objs::{ErrorMessage, ErrorType};
  use std::io;

  use crate::error::AppSetupError;

  #[test]
  fn test_app_setup_error_async_runtime_to_error_message() {
    // Simulate an io::Error
    let io_err = io::Error::other("simulated async runtime failure");
    // Convert to AppSetupError
    let app_setup_err = AppSetupError::AsyncRuntime(io_err);
    // Convert to ErrorMessage
    let err_msg: ErrorMessage = app_setup_err.into();
    // Check the error message fields using PartialEq
    let expected = ErrorMessage::new(
      "app_setup_error-async_runtime".to_string(),
      ErrorType::InternalServer.to_string(),
      "io_error: error spawning async runtime: simulated async runtime failure".to_string(),
    );
    assert_eq!(err_msg, expected);
  }
}
