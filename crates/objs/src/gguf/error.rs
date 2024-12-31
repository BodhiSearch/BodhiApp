use crate::{impl_error_from, AppError, ErrorType, IoError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum GGUFMetadataError {
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  FileOpenError(#[from] IoError),

  #[error("invalid_magic")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  InvalidMagic(u32),

  #[error("malformed_version")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  MalformedVersion(u32),

  #[error("unexpected_eof")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  UnexpectedEOF,

  #[error("invalid_string")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500, args_delegate = false)]
  InvalidString(#[from] std::string::FromUtf8Error),

  #[error("unsupported_version")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  UnsupportedVersion(u32),

  #[error("invalid_value_type")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  InvalidValueType(u32),

  #[error("invalid_array_value_type")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  InvalidArrayValueType(u32),

  #[error("type_mismatch")]
  #[error_meta(error_type = ErrorType::InternalServer, status = 500)]
  TypeMismatch { expected: String, actual: String },
}

impl_error_from!(
  ::std::io::Error,
  GGUFMetadataError::FileOpenError,
  crate::IoError
);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    test_utils::{assert_error_message, setup_l10n},
    FluentLocalizationService,
  };
  use rstest::rstest;
  use std::sync::Arc;

  #[rstest]
  #[case(&GGUFMetadataError::InvalidMagic(123), "Invalid magic number in GGUF file: 123")]
  #[case(&GGUFMetadataError::MalformedVersion(123), "Malformed GGUF version: 123")]
  #[case(&GGUFMetadataError::UnexpectedEOF, "Encountered unexpected end of file")]
  #[case(&GGUFMetadataError::UnsupportedVersion(123), "Unsupported GGUF version: 123")]
  #[case(&GGUFMetadataError::InvalidString(String::from_utf8(vec![0xE0, 0x80]).unwrap_err()), "Error converting bytes to UTF-8: invalid utf-8 sequence of 1 bytes from index 0")]
  #[case(&GGUFMetadataError::InvalidValueType(123), "Invalid value type: 123")]
  #[case(&GGUFMetadataError::InvalidArrayValueType(123), "Invalid value type in array: 123")]
  #[case(&GGUFMetadataError::TypeMismatch { expected: "expected".to_string(), actual: "actual".to_string() }, "Type mismatch: expected expected, got actual")]
  fn test_error_messages(
    #[from(setup_l10n)] localization_service: &Arc<FluentLocalizationService>,
    #[case] error: &dyn AppError,
    #[case] expected_message: &str,
  ) {
    assert_error_message(
      localization_service,
      &error.code(),
      error.args(),
      expected_message,
    );
  }
}
