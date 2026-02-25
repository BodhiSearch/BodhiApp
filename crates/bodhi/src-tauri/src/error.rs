use std::io;

use lib_bodhiserver::{
  services::SettingServiceError, AppError, BootstrapError, ErrorType, ServeError,
};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppSetupError {
  #[error(transparent)]
  Bootstrap(#[from] BootstrapError),

  #[error("Failed to start application: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  AsyncRuntime(#[from] io::Error),

  #[error(transparent)]
  Serve(#[from] ServeError),

  #[error(transparent)]
  SettingService(#[from] SettingServiceError),
}

#[cfg(test)]
mod tests {
  use std::io;

  use crate::error::AppSetupError;

  #[test]
  fn test_app_setup_error_async_runtime_display() {
    let io_err = io::Error::other("simulated async runtime failure");
    let app_setup_err = AppSetupError::AsyncRuntime(io_err);
    assert_eq!(
      "Failed to start application: simulated async runtime failure.",
      app_setup_err.to_string()
    );
  }
}
