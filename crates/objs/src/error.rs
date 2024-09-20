use crate::builder::BuilderError;
use llama_server_bindings::GptParamsBuilderError;
use std::{io, path::PathBuf};
use thiserror::Error;
use validator::{ValidationError, ValidationErrors};

#[derive(Debug, Error)]
pub enum ObjError {
  #[error(transparent)]
  Validation(#[from] ValidationErrors),
  #[error("cannot convert '{from}' to '{to}', error: \"{error}\"")]
  Conversion {
    from: String,
    to: String,
    error: String,
  },
  #[error("io error: {source}\npath: {path}")]
  IoWithDetail {
    #[source]
    source: io::Error,
    path: PathBuf,
  },
  #[error(transparent)]
  SerdeJson(#[from] serde_json::Error),
  #[error(transparent)]
  Builder(#[from] BuilderError),
  #[error(transparent)]
  GptBuilder(#[from] GptParamsBuilderError),
}

#[allow(unused)]
pub type Result<T> = std::result::Result<T, ObjError>;

pub fn validation_errors(field: &'static str, error: ValidationError) -> ValidationErrors {
  let mut errs = ValidationErrors::new();
  errs.add(field, error);
  errs
}
