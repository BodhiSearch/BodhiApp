use crate::BodhiErrorResponse;
use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use serde::Serialize;
use services::AppError;

/// Google Gemini error envelope. Wraps BodhiApp local errors into Google's
/// `{error: {code, message, status}}` shape. Upstream proxy errors pass
/// through verbatim — only local errors (validation, alias-not-found) are wrapped.
#[derive(Debug, Serialize)]
pub struct GeminiErrorResponse {
  pub error: GeminiErrorBody,
}

#[derive(Debug, Serialize)]
pub struct GeminiErrorBody {
  /// HTTP status code as integer (e.g. 400, 404).
  pub code: u16,
  pub message: String,
  /// gRPC-style status string (e.g. "INVALID_ARGUMENT", "NOT_FOUND").
  pub status: &'static str,
}

/// Wraps BodhiApp local errors into Gemini's error envelope.
#[derive(Debug)]
pub struct GeminiApiError {
  pub status: u16,
  pub body: GeminiErrorResponse,
}

impl GeminiApiError {
  fn new(http_status: u16, grpc_status: &'static str, message: impl Into<String>) -> Self {
    Self {
      status: http_status,
      body: GeminiErrorResponse {
        error: GeminiErrorBody {
          code: http_status,
          message: message.into(),
          status: grpc_status,
        },
      },
    }
  }

  pub fn invalid_request(message: impl Into<String>) -> Self {
    Self::new(400, "INVALID_ARGUMENT", message)
  }

  pub fn missing_model() -> Self {
    Self::invalid_request("Model ID is required.")
  }

  pub fn not_found(message: impl Into<String>) -> Self {
    Self::new(404, "NOT_FOUND", message)
  }
}

fn map_error_to_grpc_status(bodhi_error_type: &str) -> (u16, &'static str) {
  match bodhi_error_type {
    "invalid_request_error" => (400, "INVALID_ARGUMENT"),
    "authentication_error" => (401, "UNAUTHENTICATED"),
    "forbidden_error" => (403, "PERMISSION_DENIED"),
    "not_found_error" => (404, "NOT_FOUND"),
    "internal_server_error" => (500, "INTERNAL"),
    "service_unavailable" => (503, "UNAVAILABLE"),
    "unprocessable_entity_error" => (400, "INVALID_ARGUMENT"),
    _ => (500, "INTERNAL"),
  }
}

impl From<BodhiErrorResponse> for GeminiApiError {
  fn from(value: BodhiErrorResponse) -> Self {
    let (http_status, grpc_status) = map_error_to_grpc_status(&value.error.r#type);
    // 5xx error messages may include internal service/DB details — substitute a generic
    // message so implementation details don't leak to Gemini SDK callers.
    let message = if http_status >= 500 {
      "internal server error".to_string()
    } else {
      value.error.message
    };
    Self {
      status: http_status,
      body: GeminiErrorResponse {
        error: GeminiErrorBody {
          code: http_status,
          message,
          status: grpc_status,
        },
      },
    }
  }
}

impl<T: AppError + 'static> From<T> for GeminiApiError {
  fn from(value: T) -> Self {
    Self::from(BodhiErrorResponse::from(value))
  }
}

impl IntoResponse for GeminiApiError {
  fn into_response(self) -> Response {
    Response::builder()
      .status(self.status)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&self.body).unwrap()))
      .unwrap()
  }
}

#[cfg(test)]
#[path = "test_gemini_api_schemas.rs"]
mod test_gemini_api_schemas;
