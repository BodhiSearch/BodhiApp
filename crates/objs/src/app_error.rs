use crate::{FluentLocalizationService, JsonRejectionError};
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
  use crate::test_utils::setup_l10n_objs;
  use axum::{body::Body, extract::Path, http::Request, routing::get, Router};
  use http_body_util::BodyExt;
  use rstest::{fixture, rstest};
  use serde_json::{json, Value};
  use std::sync::Arc;
  use tower::ServiceExt;

  #[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
  #[error_meta(trait_to_impl = AppError, error_type = "test_even_error", status = 418, code = "test_even_code")]
  #[error("even_error_obj")]
  pub struct EvenErrorObj {
    reason: String,
  }

  #[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
  #[error_meta(trait_to_impl = AppError, error_type = "test_odd_error", status = 418, code = "test_odd_code")]
  #[error("odd_error_obj")]
  pub struct OddErrorObj {
    reason: String,
  }

  async fn handler_return_different_error_objs(
    Path(input): Path<String>,
  ) -> Result<Response, ApiError> {
    if input.parse::<i32>().unwrap() % 2 == 0 {
      Err(EvenErrorObj::new("even".to_string()))?
    } else {
      Err(OddErrorObj::new("odd".to_string()))?
    }
  }

  #[fixture]
  fn setup_l10n_objs_local(
    setup_l10n_objs: Arc<FluentLocalizationService>,
  ) -> Arc<FluentLocalizationService> {
    let test_l10n_resource = concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/tests/resources-api-error/en-US/test.ftl"
    );
    let api_errors = std::fs::read_to_string(test_l10n_resource).unwrap();
    setup_l10n_objs
      .load_locale(EN_US.clone(), vec![api_errors])
      .unwrap();
    setup_l10n_objs
  }

  #[rstest]
  #[case("2", ErrorBody {
    message: "even_error from l10n file: \u{2068}even\u{2069}".to_string(),
    r#type: "test_even_error".to_string(),
    code: Some("test_even_code".to_string()),
    param: None
  })]
  #[case("3", ErrorBody {
    message: "odd_error from l10n file: \u{2068}odd\u{2069}".to_string(),
    r#type: "test_odd_error".to_string(),
    code: Some("test_odd_code".to_string()),
    param: None
  })]
  #[tokio::test]
  #[serial_test::serial(localization)]
  async fn test_app_error_into_response(
    _setup_l10n_objs_local: Arc<FluentLocalizationService>,
    #[case] input: &str,
    #[case] error: ErrorBody,
  ) -> anyhow::Result<()> {
    let router = Router::new().route("/:input", get(handler_return_different_error_objs));
    let req = Request::get(format!("/{}", input))
      .body(Body::empty())
      .unwrap();
    let response = router.oneshot(req).await?;
    assert_eq!(418, response.status());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let str = String::from_utf8_lossy(&bytes);
    let response_json = serde_json::from_str::<OpenAIApiError>(&str)?;
    assert_eq!(response_json, OpenAIApiError { error });
    Ok(())
  }

  #[derive(Debug, thiserror::Error, errmeta_derive::ErrorMeta, derive_new::new)]
  #[error_meta(trait_to_impl = AppError, error_type = "test_error_response", status = 418)]
  #[error("error_response_obj")]
  pub struct ErrorResponseObj {
    reason: String,
  }

  async fn handler_response_error() -> Result<Response, ApiError> {
    Err(ErrorResponseObj::new("error message".to_string()))?
  }

  #[rstest]
  #[tokio::test]
  #[serial_test::serial(localization)]
  async fn test_app_error_custom_into_response(
    _setup_l10n_objs_local: Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let router = Router::new().route("/", get(handler_response_error));
    let req = Request::get("/").body(Body::empty()).unwrap();
    let response = router.oneshot(req).await?;
    assert_eq!(418, response.status());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let str = String::from_utf8_lossy(&bytes);
    let response_json = serde_json::from_str::<Value>(&str)?;
    assert_eq!(
      response_json,
      json! {{
        "error": {
          "message": "from localization file: \u{2068}error message\u{2069}",
          "type": "test_error_response",
          "code": "error_response_obj",
        }
      }}
    );
    Ok(())
  }

  #[allow(unused)]
  async fn handler_auto_into_response() -> Result<Response, ApiError> {
    Err(ErrorResponseObj::new("error message".to_string()))?
  }

  #[rstest]
  #[tokio::test]
  #[serial_test::serial(localization)]
  async fn test_error_auto_into_response(
    _setup_l10n_objs_local: Arc<FluentLocalizationService>,
  ) -> anyhow::Result<()> {
    let req = Request::get("/").body(Body::empty()).unwrap();
    let response = Router::new()
      .route("/", get(handler_auto_into_response))
      .oneshot(req)
      .await?;
    assert_eq!(418, response.status());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let str = String::from_utf8_lossy(&bytes);
    let response_json = serde_json::from_str::<Value>(&str)?;
    assert_eq!(
      response_json,
      json! {{
        "error": {
          "message": "from localization file: \u{2068}error message\u{2069}",
          "type": "test_error_response",
          "code": "error_response_obj",
        },
      }}
    );
    Ok(())
  }
}
