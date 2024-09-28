use derive_builder::UninitializedFieldError;
use std::{
  error::Error,
  fmt::{self, Display},
};
use std::{io, path::PathBuf};
use validator::{ValidationError, ValidationErrors};

#[doc = "Common error type for derive_builder::Builder"]
#[derive(Debug)]
#[non_exhaustive]
pub enum BuilderError {
  #[doc = r" Uninitialized field"]
  UninitializedField(&'static str),
  #[doc = r" Custom validation error"]
  ValidationError(String),
}

impl From<UninitializedFieldError> for BuilderError {
  fn from(s: UninitializedFieldError) -> Self {
    Self::UninitializedField(s.field_name())
  }
}

impl From<String> for BuilderError {
  fn from(s: String) -> Self {
    Self::ValidationError(s)
  }
}

impl Display for BuilderError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::UninitializedField(ref field) => write!(f, "`{}` must be initialized", field),
      Self::ValidationError(ref error) => write!(f, "{}", error),
    }
  }
}

impl Error for BuilderError {}

#[derive(Debug, thiserror::Error)]
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
