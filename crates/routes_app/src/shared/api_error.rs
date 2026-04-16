use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use services::AppError;
use std::borrow::Borrow;
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "message": "Validation failed: name is required",
    "type": "invalid_request_error",
    "code": "validation_error",
    "params": {"field": "name", "value": "invalid"},
    "param": "{\"field\":\"name\",\"value\":\"invalid\"}"
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
  pub params: Option<HashMap<String, String>>,

  /// JSON-encoded form of `params`. Superset field so clients that speak the
  /// OpenAI `Error` shape (where `param` is a String) can still read it.
  /// Populated automatically from `params` by `BodhiError::new`.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  #[schema(example = "{\"field\":\"name\",\"value\":\"invalid\"}")]
  pub param: Option<String>,
}

impl BodhiError {
  pub fn new(
    message: String,
    r#type: String,
    code: Option<String>,
    params: Option<HashMap<String, String>>,
  ) -> Self {
    let param = params.as_ref().and_then(|p| serde_json::to_string(p).ok());
    Self {
      message,
      r#type,
      code,
      params,
      param,
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new, ToSchema, thiserror::Error)]
#[schema(example = json!({
    "error": {
        "message": "Validation failed: name is required",
        "type": "invalid_request_error",
        "code": "validation_error",
        "params": {"field": "name"},
        "param": "{\"field\":\"name\"}"
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
    let params = if args.is_empty() { None } else { Some(args) };
    BodhiErrorResponse {
      error: BodhiError::new(
        value.to_string(),
        value.error_type(),
        Some(value.code()),
        params,
      ),
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
