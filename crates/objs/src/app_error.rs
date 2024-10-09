use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new)]
struct ErrorBody {
  pub message: String,
  pub r#type: String,
  pub code: Option<String>,
  pub param: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_new::new)]
struct OpenAIApiError {
  pub error: ErrorBody,
}

impl IntoResponse for Box<dyn AppError> {
  fn into_response(self) -> Response {
    let error = OpenAIApiError {
      error: ErrorBody {
        message: self.to_string(),
        r#type: self.error_type(),
        code: Some(self.code()),
        param: None,
      },
    };
    (
      StatusCode::try_from(self.status_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
      Json(error),
    )
      .into_response()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use axum::{body::Body, extract::Path, http::Request, routing::get, Router};
  use http_body_util::BodyExt;
  use rstest::rstest;
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

  async fn handler_return_error(Path(input): Path<String>) -> Result<Response, Box<dyn AppError>> {
    if input.parse::<i32>().unwrap() % 2 == 0 {
      Err(EvenErrorObj::new("even".to_string()))?
    } else {
      Err(OddErrorObj::new("odd".to_string()))?
    }
  }

  #[rstest]
  #[case("2", ErrorBody {
    message: "even_error_obj".to_string(),
    r#type: "test_even_error".to_string(),
    code: Some("test_even_code".to_string()),
    param: None
  })]
  #[case("3", ErrorBody {
    message: "odd_error_obj".to_string(),
    r#type: "test_odd_error".to_string(),
    code: Some("test_odd_code".to_string()),
    param: None
  })]
  #[tokio::test]
  async fn test_app_error_into_response(
    #[case] input: &str,
    #[case] error: ErrorBody,
  ) -> anyhow::Result<()> {
    let router = Router::new().route("/:input", get(handler_return_error));
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

  impl IntoResponse for ErrorResponseObj {
    fn into_response(self) -> Response {
      let api_error = OpenAIApiError {
        error: ErrorBody {
          message: self.to_string(),
          r#type: self.error_type(),
          code: Some(self.code()),
          param: None,
        },
      };
      Response::builder()
        .status(self.status_u16())
        .body(Body::from(serde_json::to_string(&api_error).unwrap()))
        .unwrap()
    }
  }

  async fn handler_response_error() -> Result<Response, ErrorResponseObj> {
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
    let response_json = serde_json::from_str::<OpenAIApiError>(&str)?;
    assert_eq!(
      response_json,
      OpenAIApiError {
        error: ErrorBody {
          message: "error_response_obj".to_string(),
          r#type: "test_error_response".to_string(),
          code: Some("error_response_code".to_string()),
          param: None
        }
      }
    );
    Ok(())
  }
}
