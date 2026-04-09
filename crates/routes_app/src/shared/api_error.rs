use crate::shared::error_oai::{BodhiApiError, BodhiErrorBody};
use async_openai::error::{ApiError as OaiErrorBody, WrappedError as OaiWrappedError};
use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use services::AppError;
use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub struct ApiError {
  pub name: String,
  pub error_type: String,
  pub status: u16,
  pub code: String,
  pub args: HashMap<String, String>,
}

impl std::fmt::Display for ApiError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let msg = serde_json::to_string(self).unwrap_or_else(|err| format!("{:?}", err));
    write!(f, "{msg}")
  }
}

impl<T: AppError + 'static> From<T> for ApiError {
  fn from(value: T) -> Self {
    let value = value.borrow();
    ApiError {
      name: value.to_string(),
      error_type: value.error_type(),
      status: value.status(),
      code: value.code(),
      args: value.args(),
    }
  }
}

impl From<ApiError> for BodhiApiError {
  fn from(value: ApiError) -> Self {
    let ApiError {
      name,
      error_type,
      status,
      code,
      args,
    } = value;
    let param = if args.is_empty() { None } else { Some(args) };
    BodhiApiError {
      error: BodhiErrorBody {
        message: name,
        r#type: error_type,
        code: Some(code),
        param,
      },
      status,
    }
  }
}

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let openai_error: BodhiApiError = self.into();
    Response::builder()
      .status(openai_error.status)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&openai_error).unwrap()))
      .unwrap()
  }
}

/// Wire-format error type for OpenAI- and Ollama-compatible endpoints.
///
/// Wraps the internal [`ApiError`] but serializes to OpenAI's native error
/// envelope (async-openai's `WrappedError { error: ApiError }`) on the wire,
/// so OpenAI SDK clients can parse 4xx/5xx responses correctly. Use this as
/// the return type for any handler exposed by the OAI spec:
///
/// ```ignore
/// pub async fn chat_completions_handler(...) -> Result<Json<Response>, OaiApiError> { ... }
/// ```
///
/// Unlike [`BodhiApiError`], the `param` field is a single `Option<String>`
/// (joining any key=value pairs from the internal args map) to match OpenAI's
/// wire format exactly.
#[derive(Debug)]
pub struct OaiApiError(pub ApiError);

impl From<ApiError> for OaiApiError {
  fn from(value: ApiError) -> Self {
    Self(value)
  }
}

impl<T: AppError + 'static> From<T> for OaiApiError {
  fn from(value: T) -> Self {
    Self(ApiError::from(value))
  }
}

impl From<OaiApiError> for OaiWrappedError {
  fn from(value: OaiApiError) -> Self {
    let ApiError {
      name,
      error_type,
      status: _,
      code,
      args,
    } = value.0;
    // OpenAI's native `param` is Option<String>. Join multi-arg validation
    // details as "key=value, key2=value2" so callers still get the context.
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
    OaiWrappedError {
      error: OaiErrorBody {
        message: name,
        r#type: Some(error_type),
        param,
        code: Some(code),
      },
    }
  }
}

impl IntoResponse for OaiApiError {
  fn into_response(self) -> Response {
    let status = self.0.status;
    let wrapped: OaiWrappedError = self.into();
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
