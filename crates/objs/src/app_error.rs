use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait AppError: std::error::Error {
  fn error_type(&self) -> String;

  fn status(&self) -> i32;

  fn status_u16(&self) -> u16;

  fn code(&self) -> String;

  fn args(&self) -> HashMap<String, String>;
}

impl<T: AppError + 'static> From<T> for Box<dyn AppError> {
  fn from(error: T) -> Self {
    Box::new(error)
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new)]
struct ErrorBody {
  pub message: String,
  pub r#type: String,
  pub code: Option<String>,
  pub param: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new)]
struct OpenAIApiError {
  pub error: ErrorBody,
}
