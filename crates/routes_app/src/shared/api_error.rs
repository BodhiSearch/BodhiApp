use crate::shared::error_oai::{ErrorBody, OpenAIApiError};
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

impl From<ApiError> for OpenAIApiError {
  fn from(value: ApiError) -> Self {
    let ApiError {
      name,
      error_type,
      status,
      code,
      args,
    } = value;
    let param = if args.is_empty() { None } else { Some(args) };
    OpenAIApiError {
      error: ErrorBody {
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
    let openai_error: OpenAIApiError = self.into();
    Response::builder()
      .status(openai_error.status)
      .header("Content-Type", "application/json")
      .body(Body::from(serde_json::to_string(&openai_error).unwrap()))
      .unwrap()
  }
}

#[cfg(test)]
#[path = "test_api_error.rs"]
mod test_api_error;
