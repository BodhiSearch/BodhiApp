use crate::{AppError, ErrorBody, ErrorMessage, JsonRejectionError, OpenAIApiError};
use axum::{
  body::Body,
  extract::rejection::JsonRejection,
  response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashMap;

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
    let msg = serde_json::to_string(self).unwrap_or_else(|err| format!("{:?}", err));
    write!(f, "{msg}")
  }
}

impl From<JsonRejection> for ApiError {
  fn from(value: JsonRejection) -> Self {
    JsonRejectionError::new(value).into()
  }
}

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

impl From<ApiError> for ErrorMessage {
  fn from(value: ApiError) -> Self {
    let ApiError {
      name,
      error_type,
      code,
      ..
    } = value;
    Self::new(code, error_type, name)
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
mod tests {
  use crate::{ApiError, BadRequestError, InternalServerError};
  use axum::{body::Body, extract::Path, http::Request, response::Response, routing::get, Router};
  use http_body_util::BodyExt;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::Value;
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
    "message": "Invalid request: even.",
    "type": "invalid_request_error",
    "code": "bad_request_error",
    "param": {"reason": "even"}
    }}
  })]
  #[case("3", 500, serde_json::json! {{
    "error": {
      "message": "Internal error: odd.",
      "type": "internal_server_error",
      "code": "internal_server_error",
      "param": {"reason": "odd"}
    }
  }})]
  #[tokio::test]
  async fn test_app_error_into_response(
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
