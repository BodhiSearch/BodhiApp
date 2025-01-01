use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ErrorMetas {
  pub message: String,
  pub code: String,
  pub error_type: String,
  pub args: HashMap<String, String>,
}
