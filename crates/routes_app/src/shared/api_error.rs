use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use services::AppError;
use std::borrow::Borrow;
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new, ToSchema)]
#[schema(example = json!({
    "message": "Validation failed: name is required",
    "type": "invalid_request_error",
    "code": "validation_error",
    "param": {"field": "name", "value": "invalid"}
}))]
pub struct BodhiError {
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

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new, ToSchema, thiserror::Error)]
#[schema(example = json!({
    "error": {
        "message": "Validation failed: name is required",
        "type": "invalid_request_error",
        "code": "validation_error",
        "param": {"field": "name"}
    }
}))]
pub struct BodhiErrorResponse {
  /// Error details following Bodhi API error format
  pub error: BodhiError,

  /// HTTP status code (not serialized in response)
  #[serde(skip)]
  #[schema(ignore = true)]
  pub status: u16,
}

impl std::fmt::Display for BodhiErrorResponse {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "status: {}, {}",
      self.status,
      serde_json::to_string(self).unwrap_or_else(|err| format!("{:?}", err))
    )
  }
}

impl<T: AppError + 'static> From<T> for BodhiErrorResponse {
  fn from(value: T) -> Self {
    let value = value.borrow();
    let args = value.args();
    let param = if args.is_empty() { None } else { Some(args) };
    BodhiErrorResponse {
      error: BodhiError {
        message: value.to_string(),
        r#type: value.error_type(),
        code: Some(value.code()),
        param,
      },
      status: value.status(),
    }
  }
}

impl IntoResponse for BodhiErrorResponse {
  fn into_response(self) -> Response {
    Response::builder()
      .status(self.status)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&self).unwrap()))
      .unwrap()
  }
}

#[cfg(test)]
#[path = "test_api_error.rs"]
mod test_api_error;
