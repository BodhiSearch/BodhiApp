use crate::{objs::BuilderError, shared_rw::ContextError};
use axum::{extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse, Json};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenAIApiError {
  #[error("{0}")]
  ModelNotFound(String),
  #[error("{0}")]
  InternalServer(String),
  #[error(transparent)]
  ContextError(#[from] ContextError),
  #[error(transparent)]
  JsonRejection(#[from] JsonRejection),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Builder)]
#[builder(setter(strip_option), build_fn(error = BuilderError))]
pub struct ApiError {
  pub message: String,
  pub r#type: String,
  #[builder(default)]
  pub param: Option<String>,
  #[builder(default)]
  pub code: Option<String>,
}

impl ApiError {
  pub fn internal_server(message: String) -> ApiError {
    ApiError {
      message,
      r#type: "internal_server_error".to_string(),
      param: None,
      code: Some("internal_server_error".to_string()),
    }
  }

  pub fn bad_request(err: &JsonRejection) -> ApiError {
    ApiError {
      message: err.to_string(),
      r#type: "invalid_request_error".to_string(),
      param: None,
      code: Some("invalid_value".to_string()),
    }
  }
}

impl From<&OpenAIApiError> for ApiError {
  fn from(value: &OpenAIApiError) -> Self {
    match value {
      OpenAIApiError::ModelNotFound(model) => ApiError {
        message: format!("The model '{}' does not exist", model),
        r#type: "invalid_request_error".to_string(),
        param: Some("model".to_string()),
        code: Some("model_not_found".to_string()),
      },
      OpenAIApiError::ContextError(err) => ApiError::internal_server(err.to_string()),
      OpenAIApiError::InternalServer(err) => ApiError::internal_server(err.to_string()),
      OpenAIApiError::JsonRejection(err) => ApiError::bad_request(err),
    }
  }
}

impl From<&OpenAIApiError> for StatusCode {
  fn from(value: &OpenAIApiError) -> Self {
    match value {
      OpenAIApiError::ModelNotFound(_) => StatusCode::NOT_FOUND,
      OpenAIApiError::ContextError(_) | OpenAIApiError::InternalServer(_) => {
        StatusCode::INTERNAL_SERVER_ERROR
      }
      OpenAIApiError::JsonRejection(_) => StatusCode::BAD_REQUEST,
    }
  }
}

impl IntoResponse for OpenAIApiError {
  fn into_response(self) -> axum::response::Response {
    (StatusCode::from(&self), Json(ApiError::from(&self))).into_response()
  }
}

pub type Result<T> = std::result::Result<T, OpenAIApiError>;
