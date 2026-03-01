use crate::{AppError, ErrorType};

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum EntityError {
  #[error("{0} not found.")]
  #[error_meta(error_type = ErrorType::NotFound)]
  NotFound(String),
}

#[cfg(test)]
#[path = "test_entity_error.rs"]
mod test_entity_error;
