use super::*;
use axum::http::StatusCode;

#[test]
fn test_invalid_request_has_correct_shape() {
  let err = GeminiApiError::invalid_request("bad input");
  assert_eq!(400, err.status);
  assert_eq!(400, err.body.error.code);
  assert_eq!("INVALID_ARGUMENT", err.body.error.status);
  assert_eq!("bad input", err.body.error.message);
}

#[test]
fn test_not_found_has_correct_shape() {
  let err = GeminiApiError::not_found("model x not found");
  assert_eq!(404, err.status);
  assert_eq!(404, err.body.error.code);
  assert_eq!("NOT_FOUND", err.body.error.status);
  assert_eq!("model x not found", err.body.error.message);
}

#[test]
fn test_missing_model_has_correct_shape() {
  let err = GeminiApiError::missing_model();
  assert_eq!(400, err.status);
  assert_eq!("INVALID_ARGUMENT", err.body.error.status);
}

#[test]
fn test_5xx_message_is_generic_not_internal_detail() {
  let api_err = crate::ApiError {
    name: "database connection pool exhausted: timed out after 5s".to_string(),
    error_type: "internal_server_error".to_string(),
    status: 500,
    code: String::new(),
    args: std::collections::HashMap::new(),
  };
  let gemini_err: GeminiApiError = api_err.into();
  assert_eq!(500, gemini_err.status);
  assert_eq!("INTERNAL", gemini_err.body.error.status);
  assert_eq!("internal server error", gemini_err.body.error.message);
}

#[test]
fn test_4xx_message_is_preserved() {
  let api_err = crate::ApiError {
    name: "alias 'foo' not found".to_string(),
    error_type: "not_found_error".to_string(),
    status: 404,
    code: String::new(),
    args: std::collections::HashMap::new(),
  };
  let gemini_err: GeminiApiError = api_err.into();
  assert_eq!(404, gemini_err.status);
  assert_eq!("alias 'foo' not found", gemini_err.body.error.message);
}

#[tokio::test]
async fn test_into_response_serializes_correctly() {
  use axum::response::IntoResponse;
  let err = GeminiApiError::invalid_request("test error");
  let response = err.into_response();
  assert_eq!(StatusCode::BAD_REQUEST, response.status());
  let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
  assert_eq!(400, body["error"]["code"].as_i64().unwrap());
  assert_eq!(
    "INVALID_ARGUMENT",
    body["error"]["status"].as_str().unwrap()
  );
  assert_eq!("test error", body["error"]["message"].as_str().unwrap());
}
