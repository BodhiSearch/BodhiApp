use derive_builder::UninitializedFieldError;
use std::{
  error::Error,
  fmt::{self, Display},
};

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
