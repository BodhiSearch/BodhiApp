use crate::{ApiError, AppError, FluentLocalizationService, LocalizationService, EN_US};
use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new, ToSchema)]
pub struct ErrorBody {
  pub message: String,
  pub r#type: String,
  pub code: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub param: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new, ToSchema)]
pub struct OpenAIApiError {
  pub error: ErrorBody,
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

const DEFAULT_ERR_MSG: &str = "something went wrong, try again later";

impl From<ApiError> for OpenAIApiError {
  fn from(value: ApiError) -> Self {
    let ApiError {
      error_type,
      status,
      code,
      args,
      ..
    } = value;
    let instance = FluentLocalizationService::get_instance();
    let message = instance
      .get_message(&EN_US, &code, Some(args))
      .unwrap_or_else(|err| {
        tracing::warn!(
          "failed to get message: err: {}, code={}, args={:?}",
          err,
          err.code(),
          err.args()
        );
        DEFAULT_ERR_MSG.to_string()
      });
    OpenAIApiError {
      error: ErrorBody {
        message,
        r#type: error_type,
        code: Some(code),
        param: None,
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
