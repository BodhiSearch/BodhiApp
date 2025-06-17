use objs::{AppError, ErrorMessage, ErrorType};
use std::io;

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppOptionsError {
  #[error("validation_error: required property '{0}' is not set")]
  #[error_meta(code = "app_options_error-validation_error", error_type = ErrorType::BadRequest)]
  ValidationError(String),
  #[error(transparent)]
  #[error_meta(code = "app_options_error-parse_error", error_type = ErrorType::BadRequest, args_delegate = false)]
  Parse(#[from] strum::ParseError),
  #[error("unknown_system_setting: {0}")]
  #[error_meta(code = "app_options_error-unknown_system_setting", error_type = ErrorType::BadRequest)]
  UnknownSystemSetting(String),
}

impl From<AppOptionsError> for ErrorMessage {
  fn from(value: AppOptionsError) -> Self {
    ErrorMessage::new(value.code(), value.error_type(), value.to_string())
  }
}
#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppDirsBuilderError {
  #[error("failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  BodhiHomeNotFound,
  #[error("failed to automatically set HF_HOME. Set it through environment variable $HF_HOME and try again.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  HfHomeNotFound,
  #[error("io_error: failed to create directory {path}, error: {source}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  DirCreate {
    #[source]
    source: io::Error,
    path: String,
  },
  #[error("io_error: failed to update the file {path}, error: {source}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  IoFileWrite {
    #[source]
    source: io::Error,
    path: String,
  },
  #[error("setting_error: failed to update the setting, check $BODHI_HOME/settings.yaml has write permission")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  SettingServiceError,
}

impl From<AppDirsBuilderError> for ErrorMessage {
  fn from(value: AppDirsBuilderError) -> Self {
    ErrorMessage::new(value.code(), value.error_type(), value.to_string())
  }
}

impl From<AppServiceBuilderError> for ErrorMessage {
  fn from(value: AppServiceBuilderError) -> Self {
    ErrorMessage::new(value.code(), value.error_type(), value.to_string())
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum AppServiceBuilderError {
  #[error("Service already set: {0}")]
  #[error_meta(error_type=ErrorType::InternalServer)]
  ServiceAlreadySet(String),
}

#[cfg(test)]
mod tests {
  use crate::AppDirsBuilderError;
  use rstest::rstest;

  #[rstest]
  #[case(AppDirsBuilderError::BodhiHomeNotFound,
    "failed to automatically set BODHI_HOME. Set it through environment variable $BODHI_HOME and try again.")]
  #[case(AppDirsBuilderError::HfHomeNotFound,
    "failed to automatically set HF_HOME. Set it through environment variable $HF_HOME and try again.")]
  fn test_app_dirs_builder_error_messages(
    #[case] error: AppDirsBuilderError,
    #[case] message: String,
  ) {
    assert_eq!(message, error.to_string());
  }
}
