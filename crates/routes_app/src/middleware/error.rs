use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use services::AppError;
use std::collections::HashMap;

pub struct MiddlewareError {
  name: String,
  error_type: String,
  status: u16,
  code: String,
  args: HashMap<String, String>,
}

impl<T: AppError + 'static> From<T> for MiddlewareError {
  fn from(value: T) -> Self {
    MiddlewareError {
      name: value.to_string(),
      error_type: value.error_type(),
      status: value.status(),
      code: value.code(),
      args: value.args(),
    }
  }
}

impl IntoResponse for MiddlewareError {
  fn into_response(self) -> Response {
    let mut error_obj = serde_json::json!({
      "message": self.name,
      "type": self.error_type,
      "code": self.code,
    });
    if !self.args.is_empty() {
      error_obj["param"] = serde_json::to_value(self.args).unwrap_or(serde_json::Value::Null);
    }
    let body = serde_json::json!({ "error": error_obj });
    let body_str = serde_json::to_string(&body).unwrap_or_else(|e| format!("{:?}", e));
    Response::builder()
      .status(self.status)
      .header("Content-Type", "application/json")
      .body(Body::from(body_str))
      .unwrap_or_else(|_| {
        Response::builder()
          .status(500)
          .header("Content-Type", "application/json")
          .body(Body::from(r#"{"error":{"message":"Internal server error","type":"internal_server_error","code":"middleware_error-response_build_failed"}}"#))
          .expect("fallback response must be valid")
      })
  }
}
