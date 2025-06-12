use crate::ErrorMessage;
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

impl From<OpenAIApiError> for ErrorMessage {
  fn from(value: OpenAIApiError) -> Self {
    Self::new(
      value.error.code.unwrap_or("unknown".to_string()),
      value.error.r#type,
      value.error.message,
    )
  }
}
