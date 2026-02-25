use objs::{AppError, ErrorType, IoError};
use services::{db::DbError, AppInstanceError, KeyringError, SessionServiceError};
use std::io;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum BootstrapError {
  // --- existing BootstrapError variants ---
  #[error("failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  BodhiHomeNotResolved,

  #[error("io_error: failed to create directory {path}, error: {source}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  DirCreate {
    #[source]
    source: io::Error,
    path: String,
  },

  #[error("BODHI_HOME value must be set")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  BodhiHomeNotSet,

  // --- absorbed from AppOptionsError ---
  #[error("validation_error: required property '{0}' is not set")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ValidationError(String),

  #[error(transparent)]
  #[error_meta(code = "bootstrap_error-parse", error_type = ErrorType::BadRequest, args_delegate = false)]
  Parse(#[from] strum::ParseError),

  #[error("unknown_system_setting: {0}")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  UnknownSystemSetting(String),

  // --- absorbed from AppServiceBuilderError ---
  #[error("{0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  ServiceAlreadySet(String),

  #[error("Encryption key not properly configured.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  PlaceholderValue(String),

  // --- replace .expect() panic ---
  #[error("AppServiceBuilder::build() called without BootstrapParts.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  MissingBootstrapParts,

  // --- transparent service error variants ---
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Db(#[from] DbError),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  AppInstance(#[from] AppInstanceError),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  SessionService(#[from] SessionServiceError),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Keyring(#[from] KeyringError),

  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  Io(#[from] IoError),
}

#[cfg(test)]
mod tests {
  use crate::BootstrapError;
  use rstest::rstest;

  #[rstest]
  #[case(BootstrapError::BodhiHomeNotResolved,
    "failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.")]
  #[case(BootstrapError::BodhiHomeNotSet, "BODHI_HOME value must be set")]
  fn test_app_dirs_builder_error_messages(#[case] error: BootstrapError, #[case] message: String) {
    assert_eq!(message, error.to_string());
  }
}
