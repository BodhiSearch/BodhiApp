use crate::ErrorType;
use std::{collections::HashMap, str::FromStr};

pub trait AppError: std::error::Error + Send + Sync + 'static {
  fn error_type(&self) -> String;

  fn status(&self) -> u16 {
    let error_type: Result<ErrorType, _> = FromStr::from_str(self.error_type().as_str());
    error_type.unwrap_or_default().status()
  }

  fn code(&self) -> String;

  fn args(&self) -> HashMap<String, String>;
}

impl<T: AppError + 'static> From<T> for Box<dyn AppError> {
  fn from(error: T) -> Self {
    Box::new(error)
  }
}

impl std::error::Error for Box<dyn AppError> {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    (**self).source()
  }
}
