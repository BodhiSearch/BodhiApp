use crate::ErrorMessage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new, ToSchema)]
#[schema(example = json!({
    "message": "Validation failed: name is required",
    "type": "invalid_request_error",
    "code": "validation_error",
    "param": {"field": "name", "value": "invalid"}
}))]
pub struct ErrorBody {
  /// Human-readable error message describing what went wrong
  #[schema(example = "Validation failed: name is required")]
  pub message: String,

  /// Error type categorizing the kind of error that occurred
  #[schema(example = "invalid_request_error")]
  pub r#type: String,

  /// Specific error code for programmatic error handling
  #[schema(example = "validation_error")]
  pub code: Option<String>,

  /// Additional error parameters as key-value pairs (for validation errors)
  #[serde(skip_serializing_if = "Option::is_none")]
  #[schema(example = json!({"field": "name", "value": "invalid"}))]
  pub param: Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new, ToSchema)]
#[schema(example = json!({
    "error": {
        "message": "Validation failed: name is required",
        "type": "invalid_request_error",
        "code": "validation_error",
        "param": "name"
    }
}))]
pub struct OpenAIApiError {
  /// Error details following OpenAI API error format
  pub error: ErrorBody,

  /// HTTP status code (not serialized in response)
  #[serde(skip)]
  #[schema(ignore = true)]
  pub status: u16,
}

impl std::fmt::Display for OpenAIApiError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "status: {}, {}",
      self.status,
      serde_json::to_string(self).unwrap()
    )
  }
}

impl From<OpenAIApiError> for ErrorMessage {
  fn from(value: OpenAIApiError) -> Self {
    Self::new(
      value.error.code.unwrap_or("unknown".to_string()),
      value.error.r#type,
      value.error.message,
    )
  }
}
