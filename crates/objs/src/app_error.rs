use axum::{
  body::Body,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new)]
pub struct ErrorBody {
  pub message: String,
  pub r#type: String,
  pub code: Option<String>,
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

impl IntoResponse for ApiError {
  fn into_response(self) -> Response {
    let message = format!("l10n: {:?}", self.args);
    Response::builder()
      .status(self.status)
      .body(Body::from(
        serde_json::to_string(&OpenAIApiError {
          error: ErrorBody {
            message,
            r#type: self.error_type,
            code: Some(self.code),
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
  use axum::{body::Body, extract::Path, http::Request, routing::get, Router};
  use http_body_util::BodyExt;
  use rstest::rstest;
  use serde_json::{json, Value};
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

  #[rstest]
  #[case("2", ErrorBody {
    message: r#"l10n: {"reason": "even"}"#.to_string(),
    r#type: "test_even_error".to_string(),
    code: Some("test_even_code".to_string()),
    param: None
  })]
  #[case("3", ErrorBody {
    message: r#"l10n: {"reason": "odd"}"#.to_string(),
    r#type: "test_odd_error".to_string(),
    code: Some("test_odd_code".to_string()),
    param: None
  })]
  #[tokio::test]
  async fn test_app_error_into_response(
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
  #[error_meta(trait_to_impl = AppError, error_type = "test_error_response", status = 418, code = "error_response_code")]
  #[error("error_response_obj")]
  pub struct ErrorResponseObj {
    reason: String,
  }

  async fn handler_response_error() -> Result<Response, ApiError> {
    Err(ErrorResponseObj::new("error message".to_string()))?
  }

  #[rstest]
  #[tokio::test]
  async fn test_app_error_custom_into_response() -> anyhow::Result<()> {
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
          "message": r#"l10n: {"reason": "error message"}"#,
          "type": "test_error_response",
          "code": "error_response_code",
          "param": null,
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
  async fn test_error_auto_into_response() -> anyhow::Result<()> {
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
          "message": r#"l10n: {"reason": "error message"}"#,
          "type": "test_error_response",
          "code": "error_response_code",
          "param": null
        },
      }}
    );
    Ok(())
  }
}
