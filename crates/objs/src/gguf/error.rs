use crate::{impl_error_from, AppError, ErrorType, IoError};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum GGUFMetadataError {
  #[error(transparent)]
  #[error_meta(error_type = ErrorType::InternalServer)]
  FileOpenError(#[from] IoError),

  #[error("Invalid model file format: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  InvalidMagic(u32),

  #[error("Invalid model version: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  MalformedVersion(u32),

  #[error("Model file appears truncated.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  UnexpectedEOF,

  #[error("Model contains invalid text: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer, args_delegate = false)]
  InvalidString(#[from] std::string::FromUtf8Error),

  #[error("Unsupported model version: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  UnsupportedVersion(u32),

  #[error("Invalid model metadata type: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  InvalidValueType(u32),

  #[error("Invalid model metadata array type: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  InvalidArrayValueType(u32),

  #[error("Model metadata type mismatch: expected {expected}, got {actual}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  TypeMismatch { expected: String, actual: String },
}

impl_error_from!(
  ::std::io::Error,
  GGUFMetadataError::FileOpenError,
  crate::IoError
);
