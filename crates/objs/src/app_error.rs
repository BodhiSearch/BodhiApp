use crate::{FluentLocalizationService, JsonRejectionError, LocalizationService};
use axum::{
  body::Body,
  extract::rejection::JsonRejection,
  response::{IntoResponse, Response},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{borrow::Borrow, str::FromStr};
use unic_langid::LanguageIdentifier;

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new)]
pub struct ErrorBody {
  pub message: String,
  pub r#type: String,
  pub code: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub param: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new)]
pub struct OpenAIApiError {
  pub error: ErrorBody,
}

pub trait AppError: std::error::Error {
  fn error_type(&self) -> String;

  fn status(&self) -> i32;

  fn status_u16(&self) -> u16;

  fn code(&self) -> String;

  fn args(&self) -> HashMap<String, String>;
}

impl<T: AppError + 'static> From<T> for Box<dyn AppError> {
  fn from(error: T) -> Self {
    Box::new(error)
  }
}

#[derive(Debug, thiserror::Error)]
#[error("api_error")]
pub struct ApiError {
  pub name: String,
  pub error_type: String,
  pub status: u16,
  pub code: String,
  pub args: HashMap<String, String>,
}

impl From<JsonRejection> for ApiError {
  fn from(value: JsonRejection) -> Self {
    JsonRejectionError::new(value).into()
  }
}

// TODO: limiting to EN_US locale, need to refactor and move creating ApiError to route_layer to
// use the request locale
pub static EN_US: Lazy<LanguageIdentifier> =
  Lazy::new(|| LanguageIdentifier::from_str("en-US").unwrap());
const DEFAULT_ERR_MSG: &str = "something went wrong, try again later";

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let ApiError {
      error_type,
      status,
      code,
      args,
      ..
    } = self;
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
    Response::builder()
      .status(status)
      .body(Body::from(
        serde_json::to_string(&OpenAIApiError {
          error: ErrorBody {
            message,
            r#type: error_type,
            code: Some(code),
            param: None,
          },
        })
        .unwrap(),
      ))
      .unwrap()
  }
}

impl<T: AppError + 'static> From<T> for ApiError {
  fn from(value: T) -> Self {
    let value = value.borrow();
    ApiError {
      name: value.to_string(),
      error_type: value.error_type(),
      status: value.status_u16(),
      code: value.code(),
      args: value.args(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{test_utils::setup_l10n, BadRequestError, InternalServerError};
  use axum::{body::Body, extract::Path, http::Request, routing::get, Router};
  use http_body_util::BodyExt;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::Value;
  use std::sync::Arc;
  use tower::ServiceExt;

  async fn handler_return_different_error_objs(
    Path(input): Path<String>,
  ) -> Result<Response, ApiError> {
    if input.parse::<i32>().unwrap() % 2 == 0 {
      Err(BadRequestError::new("even".to_string()))?
    } else {
      Err(InternalServerError::new("odd".to_string()))?
    }
  }

  #[rstest]
  #[case("2", 400, serde_json::json! {{
    "error": {
    "message": "invalid request, reason: \u{2068}even\u{2069}",
    "type": "invalid_request_error",
    "code": "bad_request_error",
    }}
  })]
  #[case("3", 500, serde_json::json! {{
    "error": {
      "message": "internal_server_error: \u{2068}odd\u{2069}",
      "type": "internal_server_error",
      "code": "internal_server_error",
    }
  }})]
  #[tokio::test]
  async fn test_app_error_into_response(
    #[from(setup_l10n)] _localization_service: &Arc<FluentLocalizationService>,
    #[case] input: &str,
    #[case] status: u16,
    #[case] expected: Value,
  ) -> anyhow::Result<()> {
    let router = Router::new().route("/:input", get(handler_return_different_error_objs));
    let req = Request::get(format!("/{}", input))
      .body(Body::empty())
      .unwrap();
    let response = router.oneshot(req).await?;
    assert_eq!(status, response.status());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let str = String::from_utf8_lossy(&bytes);
    let response_json = serde_json::from_str::<Value>(&str)?;
    assert_eq!(response_json, expected);
    Ok(())
  }
}
