use objs::{AppError, ErrorType};

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ToolsetValidationError {
  #[error("Validation error: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Validation(String),
}
