use async_openai::error::{ApiError, WrappedError};
use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use services::AppError;

/// Wire-format error type for OpenAI- and Ollama-compatible endpoints.
///
/// Serializes to OpenAI's native error envelope (async-openai's
/// `WrappedError { error: ApiError }`) on the wire, so OpenAI SDK clients can
/// parse 4xx/5xx responses correctly. Use this as the return type for any
/// handler exposed under `src/oai/`.
///
/// Unlike [`crate::BodhiErrorResponse`], the `param` field is a single
/// `Option<String>` (joining any key=value pairs from the underlying args map)
/// to match OpenAI's wire format exactly.
#[derive(Debug)]
pub struct OaiApiError {
  pub message: String,
  pub error_type: String,
  pub status: u16,
  pub code: String,
  pub param: Option<String>,
}

impl<T: AppError + 'static> From<T> for OaiApiError {
  fn from(value: T) -> Self {
    let args = value.args();
    let param = if args.is_empty() {
      None
    } else {
      Some(
        args
          .iter()
          .map(|(k, v)| format!("{}={}", k, v))
          .collect::<Vec<_>>()
          .join(", "),
      )
    };
    OaiApiError {
      message: value.to_string(),
      error_type: value.error_type(),
      status: value.status(),
      code: value.code(),
      param,
    }
  }
}

impl From<OaiApiError> for WrappedError {
  fn from(value: OaiApiError) -> Self {
    WrappedError {
      error: ApiError {
        message: value.message,
        r#type: Some(value.error_type),
        param: value.param,
        code: Some(value.code),
      },
    }
  }
}

impl IntoResponse for OaiApiError {
  fn into_response(self) -> Response {
    let status = self.status;
    let wrapped: WrappedError = self.into();
    Response::builder()
      .status(status)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&wrapped).unwrap()))
      .unwrap()
  }
}

#[cfg(test)]
#[path = "test_api_error.rs"]
mod test_api_error;
