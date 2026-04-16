use crate::anthropic::anthropic_api_schemas::AnthropicApiError;
use crate::{BodhiError, BodhiErrorResponse};
use axum::body::to_bytes;
use axum::response::IntoResponse;
use errmeta_derive::ErrorMeta;
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::{json, Value};
use services::{AppError, ErrorType};

// =============================================================================
// Synthetic AppError variants — one per ErrorType we care to map. The macro
// derives `error_type()` from the `error_meta(error_type = ...)` attribute,
// which feeds the `From<T: AppError> for BodhiErrorResponse` blanket impl, which feeds
// the `From<BodhiErrorResponse> for AnthropicApiError` impl under test.
// =============================================================================

#[derive(Debug, thiserror::Error, ErrorMeta)]
#[error_meta(trait_to_impl = AppError)]
enum SyntheticError {
  #[error("bad input")]
  #[error_meta(error_type = ErrorType::BadRequest)]
  Bad,

  #[error("missing alias")]
  #[error_meta(error_type = ErrorType::NotFound)]
  Missing,

  #[error("forbidden")]
  #[error_meta(error_type = ErrorType::Forbidden)]
  Forbidden,

  #[error("auth failed")]
  #[error_meta(error_type = ErrorType::Authentication)]
  Auth,

  #[error("oh no")]
  #[error_meta(error_type = ErrorType::InternalServer)]
  Internal,

  #[error("upstream down")]
  #[error_meta(error_type = ErrorType::ServiceUnavailable)]
  Unavailable,

  #[error("validation failed")]
  #[error_meta(error_type = ErrorType::UnprocessableEntity)]
  Unprocessable,

  #[error("conflict")]
  #[error_meta(error_type = ErrorType::Conflict)]
  Conflict,
}

#[rstest]
#[case::bad_request(SyntheticError::Bad, 400, "invalid_request_error")]
#[case::not_found(SyntheticError::Missing, 404, "not_found_error")]
#[case::forbidden(SyntheticError::Forbidden, 403, "permission_error")]
#[case::authentication(SyntheticError::Auth, 401, "authentication_error")]
#[case::internal(SyntheticError::Internal, 500, "api_error")]
#[case::service_unavailable(SyntheticError::Unavailable, 503, "overloaded_error")]
#[case::unprocessable(SyntheticError::Unprocessable, 422, "invalid_request_error")]
#[case::conflict_falls_back(SyntheticError::Conflict, 409, "api_error")]
fn test_app_error_to_anthropic_envelope(
  #[case] err: SyntheticError,
  #[case] expected_status: u16,
  #[case] expected_anthropic_type: &str,
) {
  let api_error: BodhiErrorResponse = err.into();
  let anthropic: AnthropicApiError = api_error.into();
  assert_eq!(expected_status, anthropic.status);
  assert_eq!(expected_anthropic_type, anthropic.body.error.error_type);
  assert_eq!("error", anthropic.body.envelope_type);
}

#[test]
fn test_5xx_message_is_generic_not_internal_detail() {
  let api_err = BodhiErrorResponse {
    error: BodhiError {
      message: "database connection pool exhausted: timed out after 5s".to_string(),
      r#type: "internal_server_error".to_string(),
      code: None,
      param: None,
    },
    status: 500,
  };
  let anthropic: AnthropicApiError = api_err.into();
  assert_eq!(500, anthropic.status);
  assert_eq!("internal server error", anthropic.body.error.message);
}

#[test]
fn test_4xx_message_is_preserved() {
  let api_err = BodhiErrorResponse {
    error: BodhiError {
      message: "alias 'foo' not found".to_string(),
      r#type: "not_found_error".to_string(),
      code: None,
      param: None,
    },
    status: 404,
  };
  let anthropic: AnthropicApiError = api_err.into();
  assert_eq!(404, anthropic.status);
  assert_eq!("alias 'foo' not found", anthropic.body.error.message);
}

#[test]
fn test_missing_model_constructor() {
  let err = AnthropicApiError::missing_model();
  assert_eq!(400, err.status);
  assert_eq!("invalid_request_error", err.body.error.error_type);
  assert!(err.body.error.message.contains("model"));
}

#[test]
fn test_invalid_request_constructor_uses_provided_message() {
  let err = AnthropicApiError::invalid_request("custom invalid request");
  assert_eq!(400, err.status);
  assert_eq!("invalid_request_error", err.body.error.error_type);
  assert_eq!("custom invalid request", err.body.error.message);
}

#[test]
fn test_not_found_constructor_uses_provided_message() {
  let err = AnthropicApiError::not_found("alias 'foo' not found");
  assert_eq!(404, err.status);
  assert_eq!("not_found_error", err.body.error.error_type);
  assert_eq!("alias 'foo' not found", err.body.error.message);
}

#[tokio::test]
async fn test_into_response_envelope_shape() {
  let err = AnthropicApiError::not_found("alias 'foo' not found");
  let response = err.into_response();
  assert_eq!(404, response.status().as_u16());
  assert_eq!(
    "application/json",
    response.headers().get("Content-Type").unwrap()
  );
  let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  let body: Value = serde_json::from_slice(&bytes).unwrap();
  assert_eq!(
    json!({
      "type": "error",
      "error": {
        "type": "not_found_error",
        "message": "alias 'foo' not found",
      }
    }),
    body
  );
}

#[tokio::test]
async fn test_into_response_serializes_static_strings_as_strings() {
  // Regression: the `error_type` and `envelope_type` fields are `&'static str`
  // — make sure they serialize to JSON strings (not numbers, not omitted).
  let err = AnthropicApiError::missing_model();
  let response = err.into_response();
  let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
  let body: Value = serde_json::from_slice(&bytes).unwrap();
  assert!(body["type"].is_string());
  assert!(body["error"]["type"].is_string());
  assert!(body["error"]["message"].is_string());
}
