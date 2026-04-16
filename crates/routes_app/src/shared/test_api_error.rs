use crate::shared::api_error::BodhiErrorResponse;
use axum::{
  body::{to_bytes, Body},
  extract::Path,
  http::Request,
  response::Response,
  routing::get,
  Router,
};
use errmeta_derive::ErrorMeta;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::Value;
use services::{AppError, ErrorType};
use tower::ServiceExt;

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
enum TestError {
  #[error("Bad input: {0}.")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  BadInput(String),

  #[error("Something went wrong: {0}.")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Internal(String),
}

async fn handler_return_different_error_objs(
  Path(input): Path<String>,
) -> Result<Response, BodhiErrorResponse> {
  if input.parse::<i32>().unwrap() % 2 == 0 {
    Err(TestError::BadInput("even".to_string()))?
  } else {
    Err(TestError::Internal("odd".to_string()))?
  }
}

#[rstest]
#[case("2", 400, serde_json::json! {{
  "error": {
  "message": "Bad input: even.",
  "type": "invalid_request_error",
  "code": "test_error-bad_input",
  "params": {"var_0": "even"},
  "param": "{\"var_0\":\"even\"}"
  }
}})]
#[case("3", 500, serde_json::json! {{
  "error": {
    "message": "Something went wrong: odd.",
    "type": "internal_server_error",
    "code": "test_error-internal",
    "params": {"var_0": "odd"},
    "param": "{\"var_0\":\"odd\"}"
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
  let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  let str = String::from_utf8_lossy(&bytes);
  let response_json = serde_json::from_str::<Value>(&str)?;
  assert_eq!(expected, response_json);
  Ok(())
}
