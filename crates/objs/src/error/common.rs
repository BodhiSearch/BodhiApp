use axum::http::StatusCode;
use std::{collections::HashMap, str::FromStr};

// https://help.openai.com/en/articles/6897213-openai-library-error-types-guidance
#[derive(Debug, strum::Display, strum::AsRefStr, strum::EnumString, Default)]
#[strum(serialize_all = "snake_case")]
pub enum ErrorType {
  #[strum(serialize = "invalid_request_error")]
  BadRequest,
  #[strum(serialize = "invalid_app_state")]
  InvalidAppState,
  #[strum(serialize = "internal_server_error")]
  InternalServer,
  #[strum(serialize = "authentication_error")]
  Authentication,
  #[strum(serialize = "forbidden_error")]
  Forbidden,
  #[strum(serialize = "not_found_error")]
  NotFound,
  #[strum(serialize = "conflict_error")]
  Conflict,
  #[strum(serialize = "unprocessable_entity_error")]
  UnprocessableEntity,
  #[default]
  #[strum(serialize = "unknown_error")]
  Unknown,
  #[strum(serialize = "service_unavailable")]
  ServiceUnavailable,
}

impl ErrorType {
  pub fn status(&self) -> u16 {
    match self {
      ErrorType::InternalServer => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
      ErrorType::BadRequest => StatusCode::BAD_REQUEST.as_u16(),
      ErrorType::InvalidAppState => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
      ErrorType::Authentication => StatusCode::UNAUTHORIZED.as_u16(),
      ErrorType::NotFound => StatusCode::NOT_FOUND.as_u16(),
      ErrorType::Conflict => StatusCode::CONFLICT.as_u16(),
      ErrorType::UnprocessableEntity => StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
      ErrorType::Unknown => StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
      ErrorType::Forbidden => StatusCode::FORBIDDEN.as_u16(),
      ErrorType::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE.as_u16(),
    }
  }
}

pub trait AppError: std::error::Error + Send + Sync + 'static {
  fn error_type(&self) -> String;

  fn status(&self) -> u16 {
    let error_type: Result<ErrorType, _> = FromStr::from_str(self.error_type().as_str());
    error_type.unwrap_or_default().status()
  }

  fn code(&self) -> String;

  fn args(&self) -> HashMap<String, String>;
}

impl<T: AppError + 'static> From<T> for Box<dyn AppError> {
  fn from(error: T) -> Self {
    Box::new(error)
  }
}

// Manual implementation to make Box<dyn AppError> work with std::error::Error
impl std::error::Error for Box<dyn AppError> {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    (**self).source()
  }
}
