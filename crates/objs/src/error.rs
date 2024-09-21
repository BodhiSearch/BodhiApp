use crate::builder::BuilderError;
use std::{io, path::PathBuf};
use thiserror::Error;
use validator::{ValidationError, ValidationErrors};

#[derive(Debug, Error)]
pub enum ObjError {
  #[error("Validation failed: {0}")]
  Validation(#[from] ValidationErrors),

  #[error("Cannot convert '{from}' to '{to}'. Error: {error}")]
  Conversion {
    from: String,
    to: String,
    error: String,
  },

  #[error("IO error occurred while accessing '{path}': {source}")]
  IoWithDetail {
    #[source]
    source: io::Error,
    path: PathBuf,
  },

  #[error("JSON serialization/deserialization error: {0}")]
  SerdeJson(#[from] serde_json::Error),

  #[error("Builder error: {0}")]
  Builder(#[from] BuilderError),
}

#[allow(unused)]
pub type Result<T> = std::result::Result<T, ObjError>;

pub fn validation_errors(field: &'static str, error: ValidationError) -> ValidationErrors {
  let mut errs = ValidationErrors::new();
  errs.add(field, error);
  errs
}
