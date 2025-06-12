use thiserror::Error;

use crate::{AppError, ErrorMessage, ErrorType, RwLockReadError};

#[derive(Debug, PartialEq, Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LocalizationSetupError {
  #[error("concurrency error setting up localization resource: {0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  RwLockWrite(String),
  #[error("locale is not supported: {0}")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  LocaleNotSupported(String),
}

impl From<LocalizationSetupError> for ErrorMessage {
  fn from(value: LocalizationSetupError) -> Self {
    ErrorMessage::new(value.code(), value.error_type(), value.to_string())
  }
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("locale_not_supported")]
#[error_meta(trait_to_impl = AppError, error_type = ErrorType::BadRequest)]
pub struct LocaleNotSupportedError {
  locale: String,
}

#[derive(Debug, PartialEq, Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum LocalizationMessageError {
  #[error(transparent)]
  RwLockRead(#[from] RwLockReadError),
  #[error("message_not_found")]
  #[error_meta(error_type = ErrorType::InternalServer, code = "localization_error-message_not_found")]
  MessageNotFound(String),
  #[error("format_pattern")]
  #[error_meta(error_type = ErrorType::InternalServer, code = "localization_error-format_pattern")]
  FormatPattern(String),
  #[error(transparent)]
  LocaleNotSupported(#[from] LocaleNotSupportedError),
}
