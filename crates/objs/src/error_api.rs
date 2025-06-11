use crate::{AppError, JsonRejectionError};
use axum::extract::rejection::JsonRejection;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{borrow::Borrow, str::FromStr};
use unic_langid::LanguageIdentifier;

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub struct ApiError {
  pub name: String,
  pub error_type: String,
  pub status: u16,
  pub code: String,
  pub args: HashMap<String, String>,
}

impl std::fmt::Display for ApiError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", serde_json::to_string(self).unwrap())
  }
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

impl From<Box<dyn AppError>> for ApiError {
  fn from(value: Box<dyn AppError>) -> Self {
    ApiError {
      name: value.to_string(),
      error_type: value.error_type(),
      status: value.status(),
      code: value.code(),
      args: value.args(),
    }
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

#[cfg(test)]
mod tests {
  use crate::{
    test_utils::setup_l10n, ApiError, BadRequestError, FluentLocalizationService,
    InternalServerError,
  };
  use axum::{body::Body, extract::Path, http::Request, response::Response, routing::get, Router};
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
    let router = Router::new().route("/{input}", get(handler_return_different_error_objs));
    let req = Request::get(format!("/{}", input))
      .body(Body::empty())
      .unwrap();
    let response = router.oneshot(req).await?;
    assert_eq!(status, response.status());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let str = String::from_utf8_lossy(&bytes);
    let response_json = serde_json::from_str::<Value>(&str)?;
    assert_eq!(expected, response_json);
    Ok(())
  }
}
