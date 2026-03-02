use errmeta::{AppError, ErrorType};
use validator::{ValidationError, ValidationErrors};

pub fn validation_errors(field: &'static str, error: ValidationError) -> ValidationErrors {
  let mut errs = ValidationErrors::new();
  errs.add(field, error);
  errs
}

#[derive(Debug, PartialEq, thiserror::Error, errmeta_derive::ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
pub enum ObjValidationError {
  #[error("{0}")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ValidationErrors(#[from] ValidationErrors),

  #[error("Invalid repository format '{0}'. Expected 'username/repo'.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  FilePatternMismatch(String),

  #[error("Prefix is required when forwarding all requests.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  ForwardAllRequiresPrefix,
}

#[derive(Debug)]
pub struct SerdeJsonError {
  source: serde_json::Error,
  path: Option<String>,
}

impl std::fmt::Display for SerdeJsonError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.path {
      Some(path) => write!(
        f,
        "Failed to process JSON file '{}': {}.",
        path, self.source
      ),
      None => write!(f, "Failed to process JSON data: {}.", self.source),
    }
  }
}

impl std::error::Error for SerdeJsonError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(&self.source)
  }
}

impl AppError for SerdeJsonError {
  fn error_type(&self) -> String {
    ErrorType::InternalServer.to_string()
  }

  fn code(&self) -> String {
    "serde_json_error".to_string()
  }

  fn args(&self) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    map.insert("source".to_string(), self.source.to_string());
    if let Some(path) = &self.path {
      map.insert("path".to_string(), path.clone());
    }
    map
  }
}

impl From<serde_json::Error> for SerdeJsonError {
  fn from(source: serde_json::Error) -> Self {
    Self { source, path: None }
  }
}

impl SerdeJsonError {
  pub fn new(source: serde_json::Error) -> Self {
    Self { source, path: None }
  }

  pub fn with_path(source: serde_json::Error, path: impl Into<String>) -> Self {
    Self {
      source,
      path: Some(path.into()),
    }
  }
}

#[derive(Debug)]
pub struct SerdeYamlError {
  source: serde_yaml::Error,
  path: Option<String>,
}

impl std::fmt::Display for SerdeYamlError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.path {
      Some(path) => write!(
        f,
        "Failed to process YAML file '{}': {}.",
        path, self.source
      ),
      None => write!(f, "Failed to process YAML data: {}.", self.source),
    }
  }
}

impl std::error::Error for SerdeYamlError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(&self.source)
  }
}

impl AppError for SerdeYamlError {
  fn error_type(&self) -> String {
    ErrorType::InternalServer.to_string()
  }

  fn code(&self) -> String {
    "serde_yaml_error".to_string()
  }

  fn args(&self) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    map.insert("source".to_string(), self.source.to_string());
    if let Some(path) = &self.path {
      map.insert("path".to_string(), path.clone());
    }
    map
  }
}

impl From<serde_yaml::Error> for SerdeYamlError {
  fn from(source: serde_yaml::Error) -> Self {
    Self { source, path: None }
  }
}

impl SerdeYamlError {
  pub fn new(source: serde_yaml::Error) -> Self {
    Self { source, path: None }
  }

  pub fn with_path(source: serde_yaml::Error, path: impl Into<String>) -> Self {
    Self {
      source,
      path: Some(path.into()),
    }
  }
}

#[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
#[error("Network error: {error}.")]
#[error_meta(trait_to_impl = AppError,
  error_type = ErrorType::InternalServer,
)]
pub struct ReqwestError {
  error: String,
}

impl From<reqwest::Error> for ReqwestError {
  fn from(source: reqwest::Error) -> Self {
    Self {
      error: source.to_string(),
    }
  }
}

