use crate::shared::api_error::ApiError;
use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use serde::Serialize;
use services::AppError;

/// Anthropic error envelope. Local mirror of `anthropic_api_types::ErrorResponse` —
/// defined in-repo because that crate's fields are private and cannot be constructed.
#[derive(Debug, Serialize)]
pub struct AnthropicErrorResponse {
  #[serde(rename = "type")]
  pub envelope_type: &'static str,
  pub error: AnthropicErrorBody,
}

#[derive(Debug, Serialize)]
pub struct AnthropicErrorBody {
  /// One of: `invalid_request_error`, `authentication_error`, `permission_error`,
  /// `not_found_error`, `rate_limit_error`, `api_error`, `overloaded_error`,
  /// `billing_error`, `timeout_error`.
  #[serde(rename = "type")]
  pub error_type: &'static str,
  pub message: String,
}

/// Wraps BodhiApp local errors into Anthropic's error envelope. Upstream proxy
/// errors pass through verbatim — only local errors (validation, alias-not-found) are wrapped.
#[derive(Debug)]
pub struct AnthropicApiError {
  pub status: u16,
  pub body: AnthropicErrorResponse,
}

impl AnthropicApiError {
  fn new(status: u16, error_type: &'static str, message: impl Into<String>) -> Self {
    Self {
      status,
      body: AnthropicErrorResponse {
        envelope_type: "error",
        error: AnthropicErrorBody {
          error_type,
          message: message.into(),
        },
      },
    }
  }

  pub fn missing_model() -> Self {
    Self::new(
      400,
      "invalid_request_error",
      "Field 'model' is required and must be a string.",
    )
  }

  pub fn invalid_request(message: impl Into<String>) -> Self {
    Self::new(400, "invalid_request_error", message)
  }

  pub fn not_found(message: impl Into<String>) -> Self {
    Self::new(404, "not_found_error", message)
  }
}

fn map_error_type(bodhi_error_type: &str) -> &'static str {
  match bodhi_error_type {
    "invalid_request_error" => "invalid_request_error",
    "authentication_error" => "authentication_error",
    "forbidden_error" => "permission_error",
    "not_found_error" => "not_found_error",
    "internal_server_error" => "api_error",
    "service_unavailable" => "overloaded_error",
    "unprocessable_entity_error" => "invalid_request_error",
    _ => "api_error",
  }
}

impl From<ApiError> for AnthropicApiError {
  fn from(value: ApiError) -> Self {
    Self {
      status: value.status,
      body: AnthropicErrorResponse {
        envelope_type: "error",
        error: AnthropicErrorBody {
          error_type: map_error_type(&value.error_type),
          message: value.name,
        },
      },
    }
  }
}

impl<T: AppError + 'static> From<T> for AnthropicApiError {
  fn from(value: T) -> Self {
    Self::from(ApiError::from(value))
  }
}

impl IntoResponse for AnthropicApiError {
  fn into_response(self) -> Response {
    Response::builder()
      .status(self.status)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&self.body).unwrap()))
      .unwrap()
  }
}

#[cfg(test)]
#[path = "test_anthropic_error.rs"]
mod test_anthropic_error;
